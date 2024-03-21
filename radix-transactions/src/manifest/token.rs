use sbor::rust::fmt;
use sbor::rust::fmt::Debug;

/// The span of tokens. The `start` and `end` are Unicode code points / UTF-32 - as opposed to a
/// byte-based / UTF-8 index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The start of the span, inclusive
    pub start: Position,
    /// The end of the span, exclusive
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// A 0-indexed cursor indicating the next unicode char from the start
    /// In case of end of file it equals to text length.
    pub full_index: usize,
    /// A 0-indexed cursor indicating the line number (assuming \n is a line break)
    pub line_idx: usize,
    /// A 0-indexed cursor indicating the character offset in the line
    pub line_char_index: usize,
}

impl Position {
    pub fn advance(mut self, next_char: char) -> Self {
        self.full_index += 1;
        if next_char == '\n' {
            self.line_idx += 1;
            self.line_char_index = 0;
        } else {
            self.line_char_index += 1;
        }
        self
    }

    pub fn line_number(self) -> usize {
        self.line_idx + 1
    }
}

#[macro_export]
macro_rules! position {
    ($full_index:expr, $line_idx:expr, $line_char_index:expr) => {
        Position {
            full_index: $full_index,
            line_idx: $line_idx,
            line_char_index: $line_char_index,
        }
    };
}

#[macro_export]
macro_rules! span {
    (start = ($start_full_index:expr, $start_line_idx:expr, $start_line_char_index:expr),
         end = ($end_full_index:expr, $end_line_idx:expr, $end_line_char_index:expr)) => {
        Span {
            start: position!($start_full_index, $start_line_idx, $start_line_char_index),
            end: position!($end_full_index, $end_line_idx, $end_line_char_index),
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::BoolLiteral(value) => write!(f, "'{:?}'", value),
            Token::I8Literal(value) => write!(f, "'{:?}i8'", value),
            Token::I16Literal(value) => write!(f, "'{:?}i16'", value),
            Token::I32Literal(value) => write!(f, "'{:?}i32'", value),
            Token::I64Literal(value) => write!(f, "'{:?}i64'", value),
            Token::I128Literal(value) => write!(f, "'{:?}i128'", value),
            Token::U8Literal(value) => write!(f, "'{:?}u8'", value),
            Token::U16Literal(value) => write!(f, "'{:?}u16'", value),
            Token::U32Literal(value) => write!(f, "'{:?}u32'", value),
            Token::U64Literal(value) => write!(f, "'{:?}u64'", value),
            Token::U128Literal(value) => write!(f, "'{:?}u128'", value),
            Token::StringLiteral(value) => write!(f, "{:?}", value),
            Token::Ident(value) => write!(f, "'{}'", value),
            Token::OpenParenthesis => write!(f, "'('"),
            Token::CloseParenthesis => write!(f, "')'",),
            Token::LessThan => write!(f, "'<'"),
            Token::GreaterThan => write!(f, "'>'",),
            Token::Comma => write!(f, "','"),
            Token::Semicolon => write!(f, "';'",),
            Token::FatArrow => write!(f, "'=>'"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}
