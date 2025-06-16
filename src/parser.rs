// src/parser.rs
use crate::lexer::Lexer;
use crate::token::{KEYWORDS, Token, TokenType, is_modifier, is_setting};
use anyhow::{Error, Result, anyhow};
use regex::Regex;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:2}:{:<2} │ {}",
            self.token.line, self.token.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandOption {
    Rate(Duration),
    Scale(u32),
    Format(String),
    TypingSpeed(u16),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandArg {
    // For repeat counts (like Up 5, Down 3, etc.)
    Repititions(u32),

    // For text content in Type commands
    Text(String),

    // For file paths in Output, Screenshot, Require commands
    FilePath(String),

    // For wait modes ("Line" or "Screen")
    WaitMode(String),

    // For regex patterns in Wait commands
    RegexPattern(String),

    // For control/alt/shift key combinations
    KeyCombination(String),

    // For environment variable names
    EnvVarName(String),

    // For setting values
    Height,
    FontSize(u32),
    Padding(u32),
    LoopOffset(String),
    WaitPattern(String),
    CursorBlink(bool),

    // For boolean settings
    Yes(bool),
}

impl fmt::Display for CommandArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandArg::Repititions(count) => write!(f, "{}", count),
            CommandArg::Text(text) => write!(f, "\"{}\"", text),
            CommandArg::FilePath(path) => write!(f, "{}", path),
            CommandArg::WaitMode(mode) => write!(f, "{}", mode),
            CommandArg::RegexPattern(pattern) => write!(f, "/{}/", pattern),
            CommandArg::KeyCombination(combo) => write!(f, "{}", combo),
            CommandArg::EnvVarName(var) => write!(f, "${}", var),
            CommandArg::Height => write!(f, "height"),
            CommandArg::FontSize(size) => write!(f, "{}pt", size),
            CommandArg::Padding(pad) => write!(f, "{}px", pad),
            CommandArg::LoopOffset(offset) => write!(f, "{}", offset),
            CommandArg::WaitPattern(pattern) => write!(f, "{}", pattern),
            CommandArg::CursorBlink(blink) => write!(f, "{}", blink),
            CommandArg::Yes(val) => write!(f, "{}", val),
        }
    }
}

// Helper method to join multiple CommandArgs for display implementation
impl CommandArg {
    pub fn join_args(args: &[CommandArg]) -> String {
        args.iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for CommandOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandOption::Rate(duration) => write!(f, "{}ms", duration.as_millis()),
            CommandOption::Scale(scale) => write!(f, "{}scale", scale),
            CommandOption::Format(format) => write!(f, "{}", format),
            CommandOption::TypingSpeed(speed) => write!(f, "{}wpm", speed),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeCommand {
    pub rate: Option<Duration>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SleepCommand {
    pub duration: Duration,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct OutputCommand {
    pub path: PathBuf,
    pub format: String, // "gif", "mp4", "webm"
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct KeyCommand {
    pub key: TokenType,
    pub rate: Option<Duration>,
    pub repeat_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CtrlCommand {
    pub keys: Vec<String>, // e.g., ["c"] for Ctrl+C, ["alt", "tab"] for Ctrl+Alt+Tab
    pub rate: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetCommand {
    pub setting: Setting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Setting {
    Shell(String),
    FontSize(u32),
    FontFamily(String),
    Width(u32),
    Height(u32),
    LetterSpacing(f32),
    LineHeight(f32),
    LoopOffset(f32),
    Theme(String),
    Padding(u32),
    Framerate(u32),
    PlaybackSpeed(f32),
    MarginFill(String),
    Margin(u32),
    BorderRadius(u32),
    WindowBar(String),
    WindowBarSize(u32),
    TypingSpeed(Duration),
    WaitTimeout(Duration),
    WaitPattern(String),
    CursorBlink(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequireCommand {
    pub program: String,
}

#[derive(Debug, Default, Clone)]
pub struct WaitCommand {
    pub mode: WaitMode,
    pub pattern: Option<Regex>, // regex pattern
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaitMode {
    Line,
    Screen,
}

impl FromStr for WaitMode {
    type Err = Error;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        // Normalize the string
        let normalized = input.to_lowercase();

        match normalized.as_str() {
            "line" => Ok(WaitMode::Line),
            "screen" => Ok(WaitMode::Screen),
            _ => Err(anyhow!(
                "Wait mode '{}' not recognized. Valid options: line, screen",
                input
            )),
        }
    }
}

impl Default for WaitMode {
    fn default() -> Self {
        WaitMode::Line
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenshotCommand {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopyCommand {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvCommand {
    pub variable: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum Commands {
    Type(TypeCommand),
    Sleep(SleepCommand),
    Output(OutputCommand),
    Key(KeyCommand),
    Ctrl(CtrlCommand),
    Alt(CtrlCommand),
    Shift(CtrlCommand),
    Set(SetCommand),
    Require(RequireCommand),
    Wait(WaitCommand),
    Screenshot(ScreenshotCommand),
    Copy(CopyCommand),
    Paste, // No additional data needed
    Env(EnvCommand),
    Hide, // No additional data needed
    Show, // No additional data needed
}

impl From<TypeCommand> for Commands {
    fn from(cmd: TypeCommand) -> Self {
        Commands::Type(cmd)
    }
}

impl From<SleepCommand> for Commands {
    fn from(cmd: SleepCommand) -> Self {
        Commands::Sleep(cmd)
    }
}

impl From<OutputCommand> for Commands {
    fn from(cmd: OutputCommand) -> Self {
        Commands::Output(cmd)
    }
}

impl From<KeyCommand> for Commands {
    fn from(cmd: KeyCommand) -> Self {
        Commands::Key(cmd)
    }
}

impl From<CtrlCommand> for Commands {
    fn from(cmd: CtrlCommand) -> Self {
        Commands::Ctrl(cmd)
    }
}

impl From<SetCommand> for Commands {
    fn from(cmd: SetCommand) -> Self {
        Commands::Set(cmd)
    }
}

impl From<RequireCommand> for Commands {
    fn from(cmd: RequireCommand) -> Self {
        Commands::Require(cmd)
    }
}

impl From<WaitCommand> for Commands {
    fn from(cmd: WaitCommand) -> Self {
        Commands::Wait(cmd)
    }
}

impl From<ScreenshotCommand> for Commands {
    fn from(cmd: ScreenshotCommand) -> Self {
        Commands::Screenshot(cmd)
    }
}

impl From<CopyCommand> for Commands {
    fn from(cmd: CopyCommand) -> Self {
        Commands::Copy(cmd)
    }
}

impl From<EnvCommand> for Commands {
    fn from(cmd: EnvCommand) -> Self {
        Commands::Env(cmd)
    }
}

impl From<()> for Commands {
    fn from(_: ()) -> Self {
        Commands::Paste
    }
}

pub struct Parser<'source> {
    lexer: &'source mut Lexer<'source>,
    errors: Vec<ParseError>,
    current_token: Token,
    peek_token: Token,
}

impl<'source> Parser<'source> {
    pub fn new(lexer: &'source mut Lexer<'source>) -> Self {
        let mut parser = Parser {
            lexer,
            errors: Vec::new(),
            current_token: Token::default(),
            peek_token: Token::default(),
        };

        // Read at least two tokens so current_token and peek_token are both set
        parser.next_token();
        parser.next_token();

        parser
    }

    pub fn parse(&mut self) -> Vec<Commands> {
        let mut commands = Vec::new();

        while self.current_token.token_type != TokenType::Eof {
            // Skipping comments
            if self.current_token.token_type == TokenType::Comment {
                self.next_token();
                continue;
            }

            match self.get_current_command() {
                Ok(cmds) => commands.push(cmds),
                Err(e) => {
                    self.errors.push(ParseError {
                        token: self.current_token.clone(),
                        message: e.to_string(),
                    });
                }
            }
            self.next_token();
        }

        commands
    }

    /// Get an array of the current errors.
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    fn get_current_command(&mut self) -> Result<Commands> {
        match self.current_token.token_type {
            TokenType::Space
            | TokenType::Backspace
            | TokenType::Delete
            | TokenType::Insert
            | TokenType::Enter
            | TokenType::Escape
            | TokenType::Tab
            | TokenType::Down
            | TokenType::Left
            | TokenType::Right
            | TokenType::Up
            | TokenType::PageUp
            | TokenType::PageDown => Ok(self
                .parse_keypress(self.current_token.token_type.clone())
                .into()),
            TokenType::Set => Ok(self.parse_set()?),
            TokenType::Output => Ok(self.parse_output()?.into()),
            TokenType::Sleep => Ok(self.parse_sleep()?),
            TokenType::Type => Ok(self.parse_type()?),
            TokenType::Ctrl => Ok(self.parse_ctrl()?.into()),
            TokenType::Alt => Ok(self.parse_alt()?.into()),
            TokenType::Shift => Ok(self.parse_shift()?),
            TokenType::Hide => Ok(self.parse_hide()?),
            TokenType::Require => Ok(self.parse_require()?),
            TokenType::Show => Ok(self.parse_show()?),
            TokenType::Wait => Ok(self.parse_wait()?.into()),
            TokenType::Screenshot => Ok(self.parse_screenshot()?),
            TokenType::Copy => Ok(self.parse_copy()?),
            TokenType::Paste => Ok(self.parse_paste()?),
            TokenType::Env => Ok(self.parse_env()?),
            _ => Err(anyhow!("Invalid command: {}", self.current_token.literal)),
        }
    }

    fn parse_wait(&mut self) -> Result<WaitCommand> {
        let mut cmd = WaitCommand::default();

        if self.peek_token.token_type == TokenType::Plus {
            self.next_token();
            if self.peek_token.token_type != TokenType::String
                || (self.peek_token.literal != "Line" && self.peek_token.literal != "Screen")
            {
                return Err(anyhow!("Wait+ expects Line or Screen"));
            }
            cmd.mode = self.peek_token.literal.clone().parse()?;
            self.next_token();
        } else {
            cmd.mode = WaitMode::Line;
        }

        let speed = self.parse_speed();
        if speed != Duration::default() {
            cmd.timeout = Some(speed);
        }

        // Handle wait regex
        if self.peek_token.token_type == TokenType::Regex {
            self.next_token();
            // Make sure it's valid
            if let Err(_) = Regex::new(&self.current_token.literal) {
                return Err(anyhow!(
                    "Invalid regular expression '{}': invalid regex",
                    self.current_token.literal
                ));
            }

            // Assign the built regex
            cmd.pattern = Some(self.current_token.literal.clone().parse()?);
        }

        Ok(cmd)
    }

    fn parse_speed(&mut self) -> Duration {
        if self.peek_token.token_type == TokenType::At {
            self.next_token(); // consume the '@'
            self.parse_time()
        } else {
            Duration::default()
        }
    }

    fn parse_repeat(&mut self) -> u32 {
        if self.peek_token.token_type == TokenType::Number {
            let count: u32 = self.peek_token.literal.parse().unwrap_or(1);
            self.next_token();
            count
        } else {
            1
        }
    }

    /// Helper function that gets the corresponding duration from a time
    fn parse_time(&mut self) -> Duration {
        // get the user provided integer value for the time
        let provided_time: f64 = if self.peek_token.token_type == TokenType::Number {
            let base = self.peek_token.literal.clone();
            self.next_token(); // consume the number
            base.parse().unwrap()
        } else {
            // If the next token is not a number, this is invalid.
            self.errors.push(ParseError {
                token: self.current_token.clone(),
                message: format!("Expected time after {}", self.current_token.literal),
            });
            return Duration::default();
        };

        // Check for time unit and create Duration accordingly
        if matches!(
            self.peek_token.token_type,
            TokenType::Milliseconds | TokenType::Seconds | TokenType::Minutes
        ) {
            let duration = match self.peek_token.token_type {
                TokenType::Milliseconds => Duration::from_millis(provided_time as u64),
                TokenType::Seconds => Duration::from_secs(provided_time as u64),
                TokenType::Minutes => Duration::from_secs((provided_time * 60.0) as u64),
                _ => unreachable!(), // We should have already matched above
            };
            self.next_token(); // Advance past the time unit token
            duration
        } else {
            // Default to seconds if no marker is denoted
            Duration::from_secs(provided_time as u64)
        }
    }

    fn parse_ctrl(&mut self) -> Result<CtrlCommand> {
        // optional @<time>
        let dur = self.parse_speed();
        let rate = if dur != Duration::default() {
            Some(dur)
        } else {
            None
        };

        let mut keys = Vec::new();
        let mut in_modifier_chain = true;

        // expect a series of "+X" tokens
        while self.peek_token.token_type == TokenType::Plus {
            self.next_token(); // consume the '+'
            let peek = &self.peek_token;

            // is this a named modifier? (Alt or Shift)
            if let Some(kw) = KEYWORDS.get(&*peek.literal) {
                if is_modifier(kw) {
                    if !in_modifier_chain {
                        return Err(anyhow!("Modifiers must come before other keys"));
                    }
                    keys.push(peek.literal.clone());
                    self.next_token();
                    continue;
                }
            }

            // once we hit a non-modifier, no more modifiers allowed
            in_modifier_chain = false;

            // must be a single‐char string or one of the special keys
            let lit = peek.literal.clone();
            match peek.token_type {
                TokenType::Enter
                | TokenType::Space
                | TokenType::Backspace
                | TokenType::Delete
                | TokenType::Insert
                | TokenType::Tab
                | TokenType::Escape
                | TokenType::Minus
                | TokenType::At
                | TokenType::LeftBracket
                | TokenType::RightBracket
                | TokenType::Caret
                | TokenType::Backslash => keys.push(lit),
                TokenType::String if lit.len() == 1 => keys.push(lit),
                _ => return Err(anyhow!("Invalid Ctrl key: {}", lit)),
            }

            self.next_token(); // consume the actual key
        }

        if keys.is_empty() {
            return Err(anyhow!("Expected at least one key after Ctrl"));
        }

        Ok(CtrlCommand { keys, rate })
    }

    fn parse_alt(&mut self) -> Result<CtrlCommand> {
        // optional @<time> prefix
        let dur = self.parse_speed();
        let rate = if dur != Duration::default() {
            Some(dur)
        } else {
            None
        };

        // must be "+<key>"
        if self.peek_token.token_type != TokenType::Plus {
            return Err(anyhow!(
                "Expected '+' after Alt, got {}",
                self.peek_token.literal
            ));
        }
        self.next_token(); // consume '+'

        // validate the one key
        let peek = &self.peek_token;
        let ok = matches!(
            peek.token_type,
            TokenType::String
                | TokenType::Enter
                | TokenType::LeftBracket
                | TokenType::RightBracket
                | TokenType::Tab
        );
        if !ok {
            return Err(anyhow!("Invalid Alt key: {}", peek.literal));
        }

        let key = peek.literal.clone();
        self.next_token(); // consume the key

        Ok(CtrlCommand {
            keys: vec![key],
            rate,
        })
    }

    fn parse_shift(&mut self) -> Result<CtrlCommand> {
        // optional @<time> prefix
        let dur = self.parse_speed();
        let rate = if dur != Duration::default() {
            Some(dur)
        } else {
            None
        };

        // must be "+<key>"
        if self.peek_token.token_type != TokenType::Plus {
            return Err(anyhow!(
                "Expected '+' after Shift, got {}",
                self.peek_token.literal
            ));
        }
        self.next_token(); // consume '+'

        // validate the one key
        let peek = &self.peek_token;
        let ok = matches!(
            peek.token_type,
            TokenType::String
                | TokenType::Enter
                | TokenType::LeftBracket
                | TokenType::RightBracket
                | TokenType::Tab
        );
        if !ok {
            return Err(anyhow!("Invalid Shift key: {}", peek.literal));
        }

        let key = peek.literal.clone();
        self.next_token(); // consume the key

        Ok(CtrlCommand {
            keys: vec![key],
            rate,
        })
    }

    fn parse_keypress(&mut self, command_type: TokenType) -> KeyCommand {
        let mut cmd = KeyCommand::default();

        let speed = self.parse_speed();
        if speed != Duration::default() {
            cmd.rate = Some(speed);
        } // Otherwise this stays None

        cmd.repeat_count = self.parse_repeat();

        cmd.key = command_type; // Set the key
        cmd
    }

    fn parse_output(&mut self) -> Result<OutputCommand> {
        let mut cmd = OutputCommand::default();

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("Expected file path after output"));
        }

        let path = Path::new(&self.peek_token.literal);
        if let Some(ext) = path.extension() {
            // TODO update the enum of supported formats and have a FromStr impl on it
            cmd.format = format!(".{}", ext.to_string_lossy());
        } else {
            cmd.format = String::from(".png");
            if !self.peek_token.literal.ends_with('/') {
                return Err(anyhow!("Expected folder with trailing slash"));
            }
        }

        // Parse the path from the next token (Should be the path)
        cmd.path = PathBuf::from(self.peek_token.literal.clone());
        self.next_token();
        Ok(cmd)
    }

    fn parse_set(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Set,
            option: None,
            args: None,
        };

        if is_setting(&self.peek_token.token_type) {
            cmd.args = Some(vec![CommandArg::Text(self.peek_token.literal.clone())]);
        } else {
            return Err(anyhow!("Unknown setting: {}", self.peek_token.literal));
        }
        self.next_token();

        match self.current_token.token_type {
            TokenType::WaitTimeout => {
                let duration = self.parse_speed();
                if duration != Duration::default() {
                    cmd.option = Some(CommandOption::Rate(duration));
                }
            }
            TokenType::WaitPattern => {
                cmd.option = Some(CommandOption::Format(self.peek_token.literal.clone()));
                if let Err(_) = Regex::new(&self.peek_token.literal) {
                    return Err(anyhow!(
                        "Invalid regexp pattern: {}",
                        self.peek_token.literal
                    ));
                }
                self.next_token();
            }
            TokenType::LoopOffset => {
                let mut offset = self.peek_token.literal.clone();
                self.next_token();
                offset.push('%');
                if self.peek_token.token_type == TokenType::Percent {
                    self.next_token();
                }
                cmd.option = Some(CommandOption::Format(offset));
            }
            TokenType::TypingSpeed => {
                let speed_str = self.peek_token.literal.clone();
                self.next_token();

                // Handle time units
                if matches!(
                    self.peek_token.token_type,
                    TokenType::Milliseconds | TokenType::Seconds
                ) {
                    let format_str = format!("{}{}", speed_str, self.peek_token.literal);
                    cmd.option = Some(CommandOption::Format(format_str));
                    self.next_token();
                } else {
                    // Parse as typing speed (words per minute)
                    if let Ok(speed) = speed_str.parse::<u16>() {
                        cmd.option = Some(CommandOption::TypingSpeed(speed));
                    } else {
                        return Err(anyhow!("Invalid typing speed: {}", speed_str));
                    }
                }
            }
            TokenType::FontSize => {
                cmd.option = Some(CommandOption::Scale(
                    self.peek_token.literal.clone().parse()?,
                ));
                self.next_token();
            }
            TokenType::Padding => {
                cmd.option = Some(CommandOption::Scale(
                    self.peek_token.literal.clone().parse()?,
                ));
                self.next_token();
            }
            TokenType::Height => {
                cmd.option = Some(CommandOption::Scale(
                    self.peek_token.literal.clone().parse()?,
                ));
                cmd.args = Some(vec![CommandArg::Height]);
                self.next_token();
            }
            TokenType::CursorBlink => {
                cmd.option = Some(CommandOption::Format(self.peek_token.literal.clone()));
                self.next_token();
                if self.current_token.token_type != TokenType::Boolean {
                    return Err(anyhow!("expected boolean value."));
                }
            }
            _ => {
                cmd.option = Some(CommandOption::Format(self.peek_token.literal.clone()));
                self.next_token();
            }
        }

        Ok(cmd)
    }

    fn parse_sleep(&mut self) -> Result<Command> {
        let duration = if self.peek_token.token_type == TokenType::Number {
            self.parse_time()
        } else {
            Duration::default()
        };

        let mut cmd = Command {
            command_type: TokenType::Sleep,
            option: None,
            args: None,
        };

        if duration != Duration::default() {
            cmd.option = Some(CommandOption::Rate(duration));
        }

        Ok(cmd)
    }

    fn parse_hide(&mut self) -> Result<Command> {
        Ok(Command {
            command_type: TokenType::Hide,
            option: None,
            args: None,
        })
    }

    fn parse_require(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Require,
            option: None,
            args: None,
        };

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects one string", self.current_token.literal));
        }

        cmd.args = Some(vec![CommandArg::FilePath(self.peek_token.literal.clone())]);
        self.next_token();
        Ok(cmd)
    }

    fn parse_show(&mut self) -> Result<Command> {
        Ok(Command {
            command_type: TokenType::Show,
            option: None,
            args: None,
        })
    }

    fn parse_type(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Type,
            option: None,
            args: None,
        };

        let speed = self.parse_speed();
        if speed != Duration::default() {
            cmd.option = Some(CommandOption::Rate(speed));
        }

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects string", self.current_token.literal));
        }

        let mut args = Vec::new();
        while self.peek_token.token_type == TokenType::String {
            self.next_token();
            args.push(CommandArg::Text(self.current_token.literal.clone()));
        }

        cmd.args = Some(args);
        Ok(cmd)
    }

    fn parse_copy(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Copy,
            option: None,
            args: None,
        };

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects string", self.current_token.literal));
        }

        let mut args = Vec::new();
        while self.peek_token.token_type == TokenType::String {
            self.next_token();
            args.push(CommandArg::Text(self.current_token.literal.clone()));
        }

        cmd.args = Some(args);
        Ok(cmd)
    }

    fn parse_paste(&mut self) -> Result<Command> {
        Ok(Command {
            command_type: TokenType::Paste,
            option: None,
            args: None,
        })
    }

    fn parse_env(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Env,
            option: Some(CommandOption::Format(self.peek_token.literal.clone())),
            args: None,
        };

        self.next_token();

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects string", self.current_token.literal));
        }

        cmd.args = Some(vec![CommandArg::EnvVarName(
            self.peek_token.literal.clone(),
        )]);
        self.next_token();
        Ok(cmd)
    }

    fn parse_screenshot(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Screenshot,
            option: None,
            args: None,
        };

        if self.peek_token.token_type != TokenType::String {
            self.next_token();
            return Err(anyhow!("Expected path after Screenshot"));
        }

        let path = Path::new(&self.peek_token.literal);
        if path.extension().map_or(true, |ext| ext != "png") {
            self.next_token();
            return Err(anyhow!("Expected file with .png extension"));
        }

        cmd.args = Some(vec![CommandArg::FilePath(self.peek_token.literal.clone())]);
        self.next_token();
        Ok(cmd)
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
}
