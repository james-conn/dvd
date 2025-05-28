// src/token.rs
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Operators
    At,
    Equal,
    Plus,
    Percent,
    Slash,
    Backslash,
    Dot,
    Dash,
    Minus,
    RightBracket,
    LeftBracket,
    Caret,

    // Time units
    Em,
    Milliseconds,
    Minutes,
    Px,
    Seconds,

    // Special
    Eof,
    Illegal,

    // Keys
    Alt,
    Backspace,
    Ctrl,
    Delete,
    End,
    Enter,
    Escape,
    Home,
    Insert,
    PageDown,
    PageUp,
    Sleep,
    Space,
    Tab,
    Shift,

    // Literals
    Comment,
    Number,
    String,
    Json,
    Regex,
    Boolean,

    // Movement
    Down,
    Left,
    Right,
    Up,

    // Commands
    Hide,
    Output,
    Require,
    Set,
    Show,
    Source,
    Type,
    Screenshot,
    Copy,
    Paste,
    Shell,
    Env,

    // Settings
    FontFamily,
    FontSize,
    Framerate,
    PlaybackSpeed,
    Height,
    Width,
    LetterSpacing,
    LineHeight,
    TypingSpeed,
    Padding,
    Theme,
    LoopOffset,
    MarginFill,
    Margin,
    WindowBar,
    WindowBarSize,
    BorderRadius,
    Wait,
    WaitTimeout,
    WaitPattern,
    CursorBlink,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TokenType::At => "@",
            TokenType::Equal => "=",
            TokenType::Plus => "+",
            TokenType::Percent => "%",
            TokenType::Slash => "/",
            TokenType::Backslash => "\\",
            TokenType::Dot => ".",
            TokenType::Dash => "-",
            TokenType::Minus => "-",
            TokenType::RightBracket => "]",
            TokenType::LeftBracket => "[",
            TokenType::Caret => "^",
            _ => return write!(f, "{}", to_camel(&format!("{:?}", self))),
        };
        write!(f, "{}", s)
    }
}

lazy_static::lazy_static! {
    pub static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("em".to_string(), TokenType::Em);
        m.insert("px".to_string(), TokenType::Px);
        m.insert("ms".to_string(), TokenType::Milliseconds);
        m.insert("s".to_string(), TokenType::Seconds);
        m.insert("m".to_string(), TokenType::Minutes);
        m.insert("Set".to_string(), TokenType::Set);
        m.insert("Sleep".to_string(), TokenType::Sleep);
        m.insert("Type".to_string(), TokenType::Type);
        m.insert("Enter".to_string(), TokenType::Enter);
        m.insert("Space".to_string(), TokenType::Space);
        m.insert("Backspace".to_string(), TokenType::Backspace);
        m.insert("Delete".to_string(), TokenType::Delete);
        m.insert("Insert".to_string(), TokenType::Insert);
        m.insert("Ctrl".to_string(), TokenType::Ctrl);
        m.insert("Alt".to_string(), TokenType::Alt);
        m.insert("Shift".to_string(), TokenType::Shift);
        m.insert("Down".to_string(), TokenType::Down);
        m.insert("Left".to_string(), TokenType::Left);
        m.insert("Right".to_string(), TokenType::Right);
        m.insert("Up".to_string(), TokenType::Up);
        m.insert("PageUp".to_string(), TokenType::PageUp);
        m.insert("PageDown".to_string(), TokenType::PageDown);
        m.insert("Tab".to_string(), TokenType::Tab);
        m.insert("Escape".to_string(), TokenType::Escape);
        m.insert("End".to_string(), TokenType::End);
        m.insert("Hide".to_string(), TokenType::Hide);
        m.insert("Require".to_string(), TokenType::Require);
        m.insert("Show".to_string(), TokenType::Show);
        m.insert("Output".to_string(), TokenType::Output);
        m.insert("Shell".to_string(), TokenType::Shell);
        m.insert("FontFamily".to_string(), TokenType::FontFamily);
        m.insert("MarginFill".to_string(), TokenType::MarginFill);
        m.insert("Margin".to_string(), TokenType::Margin);
        m.insert("WindowBar".to_string(), TokenType::WindowBar);
        m.insert("WindowBarSize".to_string(), TokenType::WindowBarSize);
        m.insert("BorderRadius".to_string(), TokenType::BorderRadius);
        m.insert("FontSize".to_string(), TokenType::FontSize);
        m.insert("Framerate".to_string(), TokenType::Framerate);
        m.insert("Height".to_string(), TokenType::Height);
        m.insert("LetterSpacing".to_string(), TokenType::LetterSpacing);
        m.insert("LineHeight".to_string(), TokenType::LineHeight);
        m.insert("PlaybackSpeed".to_string(), TokenType::PlaybackSpeed);
        m.insert("TypingSpeed".to_string(), TokenType::TypingSpeed);
        m.insert("Padding".to_string(), TokenType::Padding);
        m.insert("Theme".to_string(), TokenType::Theme);
        m.insert("Width".to_string(), TokenType::Width);
        m.insert("LoopOffset".to_string(), TokenType::LoopOffset);
        m.insert("WaitTimeout".to_string(), TokenType::WaitTimeout);
        m.insert("WaitPattern".to_string(), TokenType::WaitPattern);
        m.insert("Wait".to_string(), TokenType::Wait);
        m.insert("Source".to_string(), TokenType::Source);
        m.insert("CursorBlink".to_string(), TokenType::CursorBlink);
        m.insert("true".to_string(), TokenType::Boolean);
        m.insert("false".to_string(), TokenType::Boolean);
        m.insert("Screenshot".to_string(), TokenType::Screenshot);
        m.insert("Copy".to_string(), TokenType::Copy);
        m.insert("Paste".to_string(), TokenType::Paste);
        m.insert("Env".to_string(), TokenType::Env);
        m
    };
}

pub fn is_setting(token_type: &TokenType) -> bool {
    matches!(
        token_type,
        TokenType::Shell
            | TokenType::FontFamily
            | TokenType::FontSize
            | TokenType::LetterSpacing
            | TokenType::LineHeight
            | TokenType::Framerate
            | TokenType::TypingSpeed
            | TokenType::Theme
            | TokenType::PlaybackSpeed
            | TokenType::Height
            | TokenType::Width
            | TokenType::Padding
            | TokenType::LoopOffset
            | TokenType::MarginFill
            | TokenType::Margin
            | TokenType::WindowBar
            | TokenType::WindowBarSize
            | TokenType::BorderRadius
            | TokenType::CursorBlink
            | TokenType::WaitTimeout
            | TokenType::WaitPattern
    )
}

pub fn is_command(token_type: &TokenType) -> bool {
    matches!(
        token_type,
        TokenType::Type
            | TokenType::Sleep
            | TokenType::Up
            | TokenType::Down
            | TokenType::Right
            | TokenType::Left
            | TokenType::PageUp
            | TokenType::PageDown
            | TokenType::Enter
            | TokenType::Backspace
            | TokenType::Delete
            | TokenType::Tab
            | TokenType::Escape
            | TokenType::Home
            | TokenType::Insert
            | TokenType::End
            | TokenType::Ctrl
            | TokenType::Source
            | TokenType::Screenshot
            | TokenType::Copy
            | TokenType::Paste
            | TokenType::Wait
    )
}

pub fn is_modifier(token_type: &TokenType) -> bool {
    matches!(token_type, TokenType::Alt | TokenType::Shift)
}

pub fn to_camel(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    parts
        .iter()
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

pub fn lookup_identifier(ident: &str) -> TokenType {
    KEYWORDS.get(ident).cloned().unwrap_or(TokenType::String)
}
