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

use std::sync::LazyLock;
pub static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("em", TokenType::Em);
    m.insert("px", TokenType::Px);
    m.insert("ms", TokenType::Milliseconds);
    m.insert("s", TokenType::Seconds);
    m.insert("m", TokenType::Minutes);
    m.insert("Set", TokenType::Set);
    m.insert("Sleep", TokenType::Sleep);
    m.insert("Type", TokenType::Type);
    m.insert("Enter", TokenType::Enter);
    m.insert("Space", TokenType::Space);
    m.insert("Backspace", TokenType::Backspace);
    m.insert("Delete", TokenType::Delete);
    m.insert("Insert", TokenType::Insert);
    m.insert("Ctrl", TokenType::Ctrl);
    m.insert("Alt", TokenType::Alt);
    m.insert("Shift", TokenType::Shift);
    m.insert("Down", TokenType::Down);
    m.insert("Left", TokenType::Left);
    m.insert("Right", TokenType::Right);
    m.insert("Up", TokenType::Up);
    m.insert("PageUp", TokenType::PageUp);
    m.insert("PageDown", TokenType::PageDown);
    m.insert("Tab", TokenType::Tab);
    m.insert("Escape", TokenType::Escape);
    m.insert("End", TokenType::End);
    m.insert("Hide", TokenType::Hide);
    m.insert("Require", TokenType::Require);
    m.insert("Show", TokenType::Show);
    m.insert("Output", TokenType::Output);
    m.insert("Shell", TokenType::Shell);
    m.insert("FontFamily", TokenType::FontFamily);
    m.insert("MarginFill", TokenType::MarginFill);
    m.insert("Margin", TokenType::Margin);
    m.insert("WindowBar", TokenType::WindowBar);
    m.insert("WindowBarSize", TokenType::WindowBarSize);
    m.insert("BorderRadius", TokenType::BorderRadius);
    m.insert("FontSize", TokenType::FontSize);
    m.insert("Framerate", TokenType::Framerate);
    m.insert("Height", TokenType::Height);
    m.insert("LetterSpacing", TokenType::LetterSpacing);
    m.insert("LineHeight", TokenType::LineHeight);
    m.insert("PlaybackSpeed", TokenType::PlaybackSpeed);
    m.insert("TypingSpeed", TokenType::TypingSpeed);
    m.insert("Padding", TokenType::Padding);
    m.insert("Theme", TokenType::Theme);
    m.insert("Width", TokenType::Width);
    m.insert("LoopOffset", TokenType::LoopOffset);
    m.insert("WaitTimeout", TokenType::WaitTimeout);
    m.insert("WaitPattern", TokenType::WaitPattern);
    m.insert("Wait", TokenType::Wait);
    m.insert("Source", TokenType::Source);
    m.insert("CursorBlink", TokenType::CursorBlink);
    m.insert("true", TokenType::Boolean);
    m.insert("false", TokenType::Boolean);
    m.insert("Screenshot", TokenType::Screenshot);
    m.insert("Copy", TokenType::Copy);
    m.insert("Paste", TokenType::Paste);
    m.insert("Env", TokenType::Env);
    m
});

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
