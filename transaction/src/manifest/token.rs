use sbor::rust::fmt;
use sbor::rust::fmt::Debug;

/// The span of tokens. The `start` and `end` are Unicode code points / UTF-32 - as opposed to a
/// byte-based / UTF-8 index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The start of the span, exclusive
    pub start: Position,
    /// The end of the span, inclusive
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// A 0-indexed cursor indicating the next unicode char from the start
    pub full_index: usize,
    /// A 1-indexed cursor indicating the line number (assuming \n is a line break)
    pub line_number: usize,
    /// A 0-indexed cursor indicating the character offset in the line
    pub line_char_index: usize,
}

#[macro_export]
macro_rules! position {
    ($full_index:expr, $line_number:expr, $line_char_index:expr) => {
        Position {
            full_index: $full_index,
            line_number: $line_number,
            line_char_index: $line_char_index,
        }
    };
}

#[macro_export]
macro_rules! span {
    (start = ($st_full_index:expr, $st_line_number:expr, $st_line_char_index:expr),
         end = ($end_full_index:expr, $end_line_number:expr, $end_line_char_index:expr)) => {
        Span {
            start: position!($st_full_index, $st_line_number, $st_line_char_index),
            end: position!($end_full_index, $end_line_number, $end_line_char_index),
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // ==============
    // Literals
    // ==============
    BoolLiteral(bool),
    I8Literal(i8),
    I16Literal(i16),
    I32Literal(i32),
    I64Literal(i64),
    I128Literal(i128),
    U8Literal(u8),
    U16Literal(u16),
    U32Literal(u32),
    U64Literal(u64),
    U128Literal(u128),
    StringLiteral(String),

    Ident(String),

    /* Punctuations */
    OpenParenthesis,
    CloseParenthesis,
    LessThan,
    GreaterThan,
    Comma,
    Semicolon,
    FatArrow,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::BoolLiteral(bool) => write!(f, "`{:?}`", bool),
            TokenKind::I8Literal(i8) => write!(f, "`i8: {:?}`", i8),
            TokenKind::I16Literal(i16) => write!(f, "i16: `{:?}`", i16),
            TokenKind::I32Literal(i32) => write!(f, "i32: `{:?}`", i32),
            TokenKind::I64Literal(i64) => write!(f, "i64: `{:?}`", i64),
            TokenKind::I128Literal(i128) => write!(f, "i128: `{:?}`", i128),
            TokenKind::U8Literal(u8) => write!(f, "u8: `{:?}`", u8),
            TokenKind::U16Literal(u16) => write!(f, "u16: `{:?}`", u16),
            TokenKind::U32Literal(u32) => write!(f, "u32: `{:?}`", u32),
            TokenKind::U64Literal(u64) => write!(f, "u64: `{:?}`", u64),
            TokenKind::U128Literal(u128) => write!(f, "u128: `{:?}`", u128),
            TokenKind::StringLiteral(string) => write!(f, "String: `{}`", string),
            TokenKind::Ident(string) => write!(f, "Ident: `{}`", string),
            TokenKind::OpenParenthesis => write!(f, "token `(`"),
            TokenKind::CloseParenthesis => write!(f, "token `)`",),
            TokenKind::LessThan => write!(f, "token `<`"),
            TokenKind::GreaterThan => write!(f, "token `>`",),
            TokenKind::Comma => write!(f, "token `,`"),
            TokenKind::Semicolon => write!(f, "token `;`",),
            TokenKind::FatArrow => write!(f, "token `=>`"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
