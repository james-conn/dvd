// src/parser.rs
use crate::lexer::Lexer;
use crate::token::{KEYWORDS, Token, TokenType, is_modifier, is_setting};
use anyhow::{Result, anyhow};
use regex::Regex;
use std::fmt;
use std::path::Path;
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
    Immediate,
    Format(String),
    TypingSpeed(u16),
}

impl fmt::Display for CommandOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandOption::Rate(duration) => write!(f, "{}ms", duration.as_millis()),
            CommandOption::Scale(scale) => write!(f, "{}scale", scale),
            CommandOption::Immediate => write!(f, "immediate"),
            CommandOption::Format(format) => write!(f, "{}", format),
            CommandOption::TypingSpeed(speed) => write!(f, "{}wpm", speed),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub command_type: TokenType,
    pub options: Option<CommandOption>,
    pub args: String,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(options) = &self.options {
            write!(f, "{} {} {}", self.command_type, options, self.args)
        } else {
            write!(f, "{} {}", self.command_type, self.args)
        }
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

    pub fn parse(&mut self) -> Vec<Command> {
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

    fn get_current_command(&mut self) -> Result<Command> {
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
            | TokenType::PageDown => {
                Ok(self.parse_keypress(self.current_token.token_type.clone())?)
            }
            TokenType::Set => Ok(self.parse_set()?),
            TokenType::Output => Ok(self.parse_output()?),
            TokenType::Sleep => Ok(self.parse_sleep()?),
            TokenType::Type => Ok(self.parse_type()?),
            TokenType::Ctrl => Ok(self.parse_ctrl()?),
            TokenType::Alt => Ok(self.parse_alt()?),
            TokenType::Shift => Ok(self.parse_shift()?),
            TokenType::Hide => Ok(self.parse_hide()?),
            TokenType::Require => Ok(self.parse_require()?),
            TokenType::Show => Ok(self.parse_show()?),
            TokenType::Wait => Ok(self.parse_wait()?),
            TokenType::Screenshot => Ok(self.parse_screenshot()?),
            TokenType::Copy => Ok(self.parse_copy()?),
            TokenType::Paste => Ok(self.parse_paste()?),
            TokenType::Env => Ok(self.parse_env()?),
            _ => Err(anyhow!("Invalid command: {}", self.current_token.literal)),
        }
    }

    fn parse_wait(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Wait,
            options: None,
            args: String::new(),
        };

        if self.peek_token.token_type == TokenType::Plus {
            self.next_token();
            if self.peek_token.token_type != TokenType::String
                || (self.peek_token.literal != "Line" && self.peek_token.literal != "Screen")
            {
                return Err(anyhow!("Wait+ expects Line or Screen"));
            }
            cmd.args = self.peek_token.literal.clone();
            self.next_token();
        } else {
            cmd.args = "Line".to_string();
        }

        let speed = self.parse_speed();
        if speed != Duration::default() {
            cmd.options = Some(CommandOption::Rate(speed));
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
            cmd.args = format!("{} {}", cmd.args, self.current_token.literal);
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

    fn parse_repeat(&mut self) -> String {
        if self.peek_token.token_type == TokenType::Number {
            let count = self.peek_token.literal.clone();
            self.next_token();
            count
        } else {
            "1".to_string()
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

    fn parse_ctrl(&mut self) -> Result<Command> {
        let mut args = Vec::new();
        let mut in_modifier_chain = true;

        while self.peek_token.token_type == TokenType::Plus {
            self.next_token();
            let peek = self.peek_token.clone();

            if let Some(keyword_type) = KEYWORDS.get(&*peek.literal) {
                if is_modifier(keyword_type) {
                    if !in_modifier_chain {
                        return Err(anyhow!("Modifiers must come before other characters"));
                    }
                    args.push(peek.literal);
                    self.next_token();
                    continue;
                }
            }

            in_modifier_chain = false;

            match peek.token_type {
                TokenType::Enter
                | TokenType::Space
                | TokenType::Backspace
                | TokenType::Minus
                | TokenType::At
                | TokenType::LeftBracket
                | TokenType::RightBracket
                | TokenType::Caret
                | TokenType::Backslash => {
                    args.push(peek.literal);
                }
                TokenType::String if peek.literal.len() == 1 => {
                    args.push(peek.literal);
                }
                _ => {
                    return Err(anyhow!(
                        "Invalid control argument: {}",
                        self.current_token.literal
                    ));
                }
            }

            self.next_token();
        }

        if args.is_empty() {
            return Err(anyhow!(
                "Expected control character with args, got {}",
                self.current_token.literal
            ));
        }

        Ok(Command {
            command_type: TokenType::Ctrl,
            options: None,
            args: args.join(" "),
        })
    }

    fn parse_alt(&mut self) -> Result<Command> {
        if self.peek_token.token_type == TokenType::Plus {
            self.next_token();
            if matches!(
                self.peek_token.token_type,
                TokenType::String
                    | TokenType::Enter
                    | TokenType::LeftBracket
                    | TokenType::RightBracket
                    | TokenType::Tab
            ) {
                let c = self.peek_token.literal.clone();
                self.next_token();
                return Ok(Command {
                    command_type: TokenType::Alt,
                    options: None,
                    args: c,
                });
            }
        }

        Err(anyhow!(
            "Expected alt character, got {}",
            self.current_token.literal
        ))
    }

    fn parse_shift(&mut self) -> Result<Command> {
        if self.peek_token.token_type == TokenType::Plus {
            self.next_token();
            if matches!(
                self.peek_token.token_type,
                TokenType::String
                    | TokenType::Enter
                    | TokenType::LeftBracket
                    | TokenType::RightBracket
                    | TokenType::Tab
            ) {
                let c = self.peek_token.literal.clone();
                self.next_token();
                return Ok(Command {
                    command_type: TokenType::Shift,
                    options: None,
                    args: c,
                });
            }
        }

        Err(anyhow!(
            "Expected shift character, got {}",
            self.current_token.literal
        ))
    }

    fn parse_keypress(&mut self, command_type: TokenType) -> Result<Command> {
        let mut cmd = Command {
            command_type,
            options: None,
            args: String::new(),
        };

        let speed = self.parse_speed();
        if speed != Duration::default() {
            cmd.options = Some(CommandOption::Rate(speed));
        }

        cmd.args = self.parse_repeat();
        Ok(cmd)
    }

    fn parse_output(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Output,
            options: None,
            args: String::new(),
        };

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("Expected file path after output"));
        }

        let path = Path::new(&self.peek_token.literal);
        if let Some(ext) = path.extension() {
            cmd.options = Some(CommandOption::Format(format!(".{}", ext.to_string_lossy())));
        } else {
            cmd.options = Some(CommandOption::Format(".png".to_string()));
            if !self.peek_token.literal.ends_with('/') {
                return Err(anyhow!("Expected folder with trailing slash"));
            }
        }

        cmd.args = self.peek_token.literal.clone();
        self.next_token();
        Ok(cmd)
    }

    fn parse_set(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Set,
            options: None,
            args: String::new(),
        };

        if is_setting(&self.peek_token.token_type) {
            cmd.args = self.peek_token.literal.clone();
        } else {
            return Err(anyhow!("Unknown setting: {}", self.peek_token.literal));
        }
        self.next_token();

        match self.current_token.token_type {
            TokenType::WaitTimeout => {
                let duration = self.parse_speed();
                if duration != Duration::default() {
                    cmd.options = Some(CommandOption::Rate(duration));
                }
            }
            TokenType::WaitPattern => {
                cmd.options = Some(CommandOption::Format(self.peek_token.literal.clone()));
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
                cmd.options = Some(CommandOption::Format(offset));
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
                    cmd.options = Some(CommandOption::Format(format_str));
                    self.next_token();
                } else {
                    // Parse as typing speed (words per minute)
                    if let Ok(speed) = speed_str.parse::<u16>() {
                        cmd.options = Some(CommandOption::TypingSpeed(speed));
                    } else {
                        return Err(anyhow!("Invalid typing speed: {}", speed_str));
                    }
                }
            }
            TokenType::FontSize => {
                cmd.options = Some(CommandOption::Scale(
                    self.peek_token.literal.clone().parse()?,
                ));
                self.next_token();
            }
            TokenType::Padding => {
                cmd.options = Some(CommandOption::Scale(
                    self.peek_token.literal.clone().parse()?,
                ));
                self.next_token();
            }
            TokenType::Height => {
                cmd.options = Some(CommandOption::Scale(
                    self.peek_token.literal.clone().parse()?,
                ));
                self.next_token();
            }
            TokenType::CursorBlink => {
                cmd.options = Some(CommandOption::Format(self.peek_token.literal.clone()));
                self.next_token();
                if self.current_token.token_type != TokenType::Boolean {
                    return Err(anyhow!("expected boolean value."));
                }
            }
            _ => {
                cmd.options = Some(CommandOption::Format(self.peek_token.literal.clone()));
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
            options: None,
            args: String::new(),
        };

        if duration != Duration::default() {
            cmd.options = Some(CommandOption::Rate(duration));
        }

        Ok(cmd)
    }

    fn parse_hide(&mut self) -> Result<Command> {
        Ok(Command {
            command_type: TokenType::Hide,
            options: None,
            args: String::new(),
        })
    }

    fn parse_require(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Require,
            options: None,
            args: String::new(),
        };

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects one string", self.current_token.literal));
        }

        cmd.args = self.peek_token.literal.clone();
        self.next_token();
        Ok(cmd)
    }

    fn parse_show(&mut self) -> Result<Command> {
        Ok(Command {
            command_type: TokenType::Show,
            options: None,
            args: String::new(),
        })
    }

    fn parse_type(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Type,
            options: None,
            args: String::new(),
        };

        let speed = self.parse_speed();
        if speed != Duration::default() {
            cmd.options = Some(CommandOption::Rate(speed));
        }

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects string", self.current_token.literal));
        }

        while self.peek_token.token_type == TokenType::String {
            self.next_token();
            cmd.args.push_str(&self.current_token.literal);

            if self.peek_token.token_type == TokenType::String {
                cmd.args.push(' ');
            }
        }

        Ok(cmd)
    }

    fn parse_copy(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Copy,
            options: None,
            args: String::new(),
        };

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects string", self.current_token.literal));
        }

        while self.peek_token.token_type == TokenType::String {
            self.next_token();
            cmd.args.push_str(&self.current_token.literal);

            if self.peek_token.token_type == TokenType::String {
                cmd.args.push(' ');
            }
        }

        Ok(cmd)
    }

    fn parse_paste(&mut self) -> Result<Command> {
        Ok(Command {
            command_type: TokenType::Paste,
            options: None,
            args: String::new(),
        })
    }

    fn parse_env(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Env,
            options: Some(CommandOption::Format(self.peek_token.literal.clone())),
            args: String::new(),
        };

        self.next_token();

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects string", self.current_token.literal));
        }

        cmd.args = self.peek_token.literal.clone();
        self.next_token();
        Ok(cmd)
    }

    fn parse_screenshot(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Screenshot,
            options: None,
            args: String::new(),
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

        cmd.args = self.peek_token.literal.clone();
        self.next_token();
        Ok(cmd)
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
}
