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
    /// Create a new lexer using the source input
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

    /// Consume and increment
    fn read_char(&mut self) {
        self.column += 1;
        self.current_char = self.chars.next();
        if self.current_char.is_some() {
            self.position += 1;
        }
    }

    /// Peek one char ahead
    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Skip to the next token
    pub fn next_token(&mut self) -> Token {
        // We can ignore whitespace...
        self.skip_whitespace();

        // Initialize a default token at the current line/column
        let mut token = Token::default();

        match self.current_char {
            // No token, we've reached the end
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
                // TODO: Make this much more robust. Currently doesn't even try to handle JSON escaping
                token.literal = "{".to_string() + &self.read_string('}') + "}";
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
            // The fallback case when it's not a semantic token and instead arbitrary
            Some(ch) => {
                // Stand up and pay attention if it's either a straight-up number, or some kind of fraction.
                if ch.is_ascii_digit()
                    || (ch == '.' && self.peek_char().is_some_and(|c| c.is_ascii_digit()))
                {
                    token.literal = self.read_number();
                    token.token_type = TokenType::Number;
                } else if ch.is_alphabetic() {
                    // Okay, it's probably a string, look ahead and see if this has an ID we know of.
                    token.literal = self.read_identifier();
                    token.token_type = lookup_identifier(&token.literal);
                } else {
                    // We can't find anything, this is an illegal token.
                    token = self.new_token(TokenType::Illegal, ch);
                    self.read_char();
                }
            }
        }

        token
    }

    /// Helper function for creation of a new token cheaply
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
                .is_none_or(|ch| ch == '\n' || ch == '\r')
            // Read until some kind of carrige return and then break.
            {
                break;
            }
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    /// Read a string until an end char. Useful for text within some kind of braces.
    fn read_string(&mut self, end_char: char) -> String {
        let start_pos = self.position;
        loop {
            self.read_char();
            if self
                .current_char
                .is_none_or(|ch| ch == end_char || ch == '\n' || ch == '\r')
            {
                break;
            }
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn read_number(&mut self) -> String {
        let start_pos = self.position - 1;
        while self
            .current_char // TODO: Recognize an invalid sequence and throw an error or an optional here. For a case like (0.0.0.0) -- which seems valid in this parsing logic so far
            .is_some_and(|ch| ch.is_ascii_digit() || ch == '.')
        {
            self.read_char();
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn read_identifier(&mut self) -> String {
        let start_pos = self.position - 1;
        while self.current_char.is_some_and(|ch| {
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
