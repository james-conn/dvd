// src/parser.rs
use crate::lexer::Lexer;
use crate::token::{KEYWORDS, Token, TokenType, is_modifier, is_setting};
use anyhow::{Result, anyhow};
use regex::Regex;
use std::fmt;
use std::fs;
use std::path::Path;

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
pub struct Command {
    pub command_type: TokenType,
    pub options: String,
    pub args: String,
    pub source: String,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.options.is_empty() {
            write!(f, "{} {} {}", self.command_type, self.options, self.args)
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

        // Read two tokens so current_token and peek_token are both set
        parser.next_token();
        parser.next_token();

        parser
    }

    pub fn parse(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();

        while self.current_token.token_type != TokenType::Eof {
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
            options: String::new(),
            args: String::new(),
            source: String::new(),
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

        cmd.options = self.parse_speed();
        if !cmd.options.is_empty() {
            // In a real implementation, you'd parse the duration here
            // For now, just check it's not empty
        }

        if self.peek_token.token_type == TokenType::Regex {
            self.next_token();
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

    fn parse_speed(&mut self) -> String {
        if self.peek_token.token_type == TokenType::At {
            self.next_token();
            self.parse_time()
        } else {
            String::new()
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

    fn parse_time(&mut self) -> String {
        let time = if self.peek_token.token_type == TokenType::Number {
            let t = self.peek_token.literal.clone();
            self.next_token();
            t
        } else {
            self.errors.push(ParseError {
                token: self.current_token.clone(),
                message: format!("Expected time after {}", self.current_token.literal),
            });
            return String::new();
        };

        let mut result = time;
        if matches!(
            self.peek_token.token_type,
            TokenType::Milliseconds | TokenType::Seconds | TokenType::Minutes
        ) {
            result.push_str(&self.peek_token.literal);
            self.next_token();
        } else {
            result.push('s');
        }

        result
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
            options: String::new(),
            args: args.join(" "),
            source: String::new(),
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
                    options: String::new(),
                    args: c,
                    source: String::new(),
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
                    options: String::new(),
                    args: c,
                    source: String::new(),
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
            options: String::new(),
            args: String::new(),
            source: String::new(),
        };
        cmd.options = self.parse_speed();
        cmd.args = self.parse_repeat();
        Ok(cmd)
    }

    fn parse_output(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Output,
            options: String::new(),
            args: String::new(),
            source: String::new(),
        };

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("Expected file path after output"));
        }

        let path = Path::new(&self.peek_token.literal);
        if let Some(ext) = path.extension() {
            cmd.options = format!(".{}", ext.to_string_lossy());
        } else {
            cmd.options = ".png".to_string();
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
            options: String::new(),
            args: String::new(),
            source: String::new(),
        };

        if is_setting(&self.peek_token.token_type) {
            cmd.options = self.peek_token.literal.clone();
        } else {
            return Err(anyhow!("Unknown setting: {}", self.peek_token.literal));
        }
        self.next_token();

        match self.current_token.token_type {
            TokenType::WaitTimeout => {
                cmd.args = self.parse_time();
            }
            TokenType::WaitPattern => {
                cmd.args = self.peek_token.literal.clone();
                if let Err(_) = Regex::new(&self.peek_token.literal) {
                    return Err(anyhow!(
                        "Invalid regexp pattern: {}",
                        self.peek_token.literal
                    ));
                }
                self.next_token();
            }
            TokenType::LoopOffset => {
                cmd.args = self.peek_token.literal.clone();
                self.next_token();
                cmd.args.push('%');
                if self.peek_token.token_type == TokenType::Percent {
                    self.next_token();
                }
            }
            TokenType::TypingSpeed => {
                cmd.args = self.peek_token.literal.clone();
                self.next_token();
                if matches!(
                    self.peek_token.token_type,
                    TokenType::Milliseconds | TokenType::Seconds
                ) {
                    cmd.args.push_str(&self.peek_token.literal);
                    self.next_token();
                } else if cmd.options == "TypingSpeed" {
                    cmd.args.push('s');
                }
            }
            TokenType::CursorBlink => {
                cmd.args = self.peek_token.literal.clone();
                self.next_token();
                if self.current_token.token_type != TokenType::Boolean {
                    return Err(anyhow!("expected boolean value."));
                }
            }
            _ => {
                cmd.args = self.peek_token.literal.clone();
                self.next_token();
            }
        }

        Ok(cmd)
    }

    fn parse_sleep(&mut self) -> Result<Command> {
        let cmd = Command {
            command_type: TokenType::Sleep,
            options: String::new(),
            args: self.parse_time(),
            source: String::new(),
        };
        Ok(cmd)
    }

    fn parse_hide(&mut self) -> Result<Command> {
        Ok(Command {
            command_type: TokenType::Hide,
            options: String::new(),
            args: String::new(),
            source: String::new(),
        })
    }

    fn parse_require(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Require,
            options: String::new(),
            args: String::new(),
            source: String::new(),
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
            options: String::new(),
            args: String::new(),
            source: String::new(),
        })
    }

    fn parse_type(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Type,
            options: String::new(),
            args: String::new(),
            source: String::new(),
        };

        cmd.options = self.parse_speed();

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
            options: String::new(),
            args: String::new(),
            source: String::new(),
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
            options: String::new(),
            args: String::new(),
            source: String::new(),
        })
    }

    fn parse_env(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Env,
            options: String::new(),
            args: String::new(),
            source: String::new(),
        };

        cmd.options = self.peek_token.literal.clone();
        self.next_token();

        if self.peek_token.token_type != TokenType::String {
            return Err(anyhow!("{} expects string", self.current_token.literal));
        }

        cmd.args = self.peek_token.literal.clone();
        self.next_token();
        Ok(cmd)
    }

    // fn parse_source(&mut self) -> Result<Vec<Command>> {
    //     if self.peek_token.token_type != TokenType::String {
    //         self.next_token();
    //         return Err(anyhow!("Expected path after Source"));
    //     }

    //     // Clone the path to avoid borrowing issues
    //     let src_path = self.peek_token.literal.clone();

    //     // Check if path has .tape extension
    //     let path = Path::new(&src_path);
    //     if path.extension().map_or(true, |ext| ext != "tape") {
    //         self.next_token();
    //         return Err(anyhow!("Expected file with .tape extension"));
    //     }

    //     // Check if tape exists
    //     if !path.exists() {
    //         self.next_token();
    //         return Err(anyhow!("File {} not found", src_path));
    //     }

    //     // Read and parse source tape
    //     let src_content = fs::read_to_string(&src_path)
    //         .map_err(|_| anyhow!("Unable to read file: {}", src_path))?;

    //     if src_content.is_empty() {
    //         self.next_token();
    //         return Err(anyhow!("Source tape: {} is empty", src_path));
    //     }

    //     let mut src_lexer = Lexer::new(&src_content);
    //     let mut src_parser = Parser::new(&mut src_lexer);
    //     let src_commands = src_parser.parse();

    //     // Check for nested source commands
    //     for cmd in &src_commands {
    //         if cmd.command_type == TokenType::Source {
    //             self.next_token();
    //             return Err(anyhow!("Nested Source detected"));
    //         }
    //     }

    //     // Check for errors in source
    //     if !src_parser.errors().is_empty() {
    //         self.next_token();
    //         return Err(anyhow!(
    //             "{} has {} errors",
    //             src_path,
    //             src_parser.errors().len()
    //         ));
    //     }

    //     // Filter out Output and Source commands
    //     let filtered: Vec<Command> = src_commands
    //         .into_iter()
    //         .filter(|cmd| !matches!(cmd.command_type, TokenType::Source | TokenType::Output))
    //         .collect();

    //     self.next_token();
    //     Ok(filtered)
    // }

    fn parse_screenshot(&mut self) -> Result<Command> {
        let mut cmd = Command {
            command_type: TokenType::Screenshot,
            options: String::new(),
            args: String::new(),
            source: String::new(),
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
