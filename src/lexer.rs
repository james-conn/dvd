// src/lexer.rs
use crate::token::{Token, TokenType, lookup_identifier};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'source> {
    input: &'source str,
    chars: Peekable<Chars<'source>>,
    current_char: Option<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            chars: input.chars().peekable(),
            current_char: None,
            position: 0,
            line: 1,
            column: 0,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        self.column += 1;
        self.current_char = self.chars.next();
        if self.current_char.is_some() {
            self.position += 1;
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let mut token = Token {
            token_type: TokenType::Eof,
            literal: String::new(),
            line: self.line,
            column: self.column,
        };

        match self.current_char {
            None => {
                token.token_type = TokenType::Eof;
                token.literal = "\0".to_string();
            }
            Some('@') => {
                token = self.new_token(TokenType::At, '@');
                self.read_char();
            }
            Some('=') => {
                token = self.new_token(TokenType::Equal, '=');
                self.read_char();
            }
            Some(']') => {
                token = self.new_token(TokenType::RightBracket, ']');
                self.read_char();
            }
            Some('[') => {
                token = self.new_token(TokenType::LeftBracket, '[');
                self.read_char();
            }
            Some('-') => {
                token = self.new_token(TokenType::Minus, '-');
                self.read_char();
            }
            Some('%') => {
                token = self.new_token(TokenType::Percent, '%');
                self.read_char();
            }
            Some('^') => {
                token = self.new_token(TokenType::Caret, '^');
                self.read_char();
            }
            Some('\\') => {
                token = self.new_token(TokenType::Backslash, '\\');
                self.read_char();
            }
            Some('#') => {
                token.token_type = TokenType::Comment;
                token.literal = self.read_comment();
            }
            Some('+') => {
                token = self.new_token(TokenType::Plus, '+');
                self.read_char();
            }
            Some('{') => {
                token.token_type = TokenType::Json;
                token.literal = "{".to_string() + &self.read_json() + "}";
                self.read_char();
            }
            Some('`') => {
                token.token_type = TokenType::String;
                token.literal = self.read_string('`');
                self.read_char();
            }
            Some('\'') => {
                token.token_type = TokenType::String;
                token.literal = self.read_string('\'');
                self.read_char();
            }
            Some('"') => {
                token.token_type = TokenType::String;
                token.literal = self.read_string('"');
                self.read_char();
            }
            Some('/') => {
                token.token_type = TokenType::Regex;
                token.literal = self.read_string('/');
                self.read_char();
            }
            Some(ch) => {
                if ch.is_ascii_digit()
                    || (ch == '.' && self.peek_char().map_or(false, |c| c.is_ascii_digit()))
                {
                    token.literal = self.read_number();
                    token.token_type = TokenType::Number;
                } else if ch.is_alphabetic() || ch == '.' {
                    token.literal = self.read_identifier();
                    token.token_type = lookup_identifier(&token.literal);
                } else {
                    token = self.new_token(TokenType::Illegal, ch);
                    self.read_char();
                }
            }
        }

        token
    }

    fn new_token(&self, token_type: TokenType, ch: char) -> Token {
        Token {
            token_type,
            literal: ch.to_string(),
            line: self.line,
            column: self.column,
        }
    }

    fn read_comment(&mut self) -> String {
        let start_pos = self.position;
        loop {
            self.read_char();
            if self
                .current_char
                .map_or(true, |ch| ch == '\n' || ch == '\r')
            {
                break;
            }
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn read_string(&mut self, end_char: char) -> String {
        let start_pos = self.position;
        loop {
            self.read_char();
            if self
                .current_char
                .map_or(true, |ch| ch == end_char || ch == '\n' || ch == '\r')
            {
                break;
            }
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn read_json(&mut self) -> String {
        let start_pos = self.position;
        loop {
            self.read_char();
            if self.current_char.map_or(true, |ch| ch == '}') {
                break;
            }
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn read_number(&mut self) -> String {
        let start_pos = self.position - 1;
        while self
            .current_char
            .map_or(false, |ch| ch.is_ascii_digit() || ch == '.')
        {
            self.read_char();
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn read_identifier(&mut self) -> String {
        let start_pos = self.position - 1;
        while self.current_char.map_or(false, |ch| {
            ch.is_alphanumeric() || ch == '.' || ch == '-' || ch == '_' || ch == '/' || ch == '%'
        }) {
            self.read_char();
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                self.read_char();
            } else {
                break;
            }
        }
    }
}
