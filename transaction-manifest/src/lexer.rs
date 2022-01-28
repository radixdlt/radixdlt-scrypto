use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The start of the span, inclusive
    pub start: usize,
    /// The end of the span, exclusive
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /* Literals */
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

    /* Types */
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,
    Struct,
    Enum,
    Option,
    Box,
    Array,
    Tuple,
    Result,
    Vec,
    TreeSet,
    TreeMap,
    HashSet,
    HashMap,
    Decimal,
    BigDecimal,
    Address,
    Hash,
    Bucket,
    BucketRef,
    LazyMap,
    Vault,

    /* Sub-types */
    Some,
    None,
    Ok,
    Err,

    /* Punctuations */
    OpenCurlyBrace,
    CloseCurlyBrace,
    OpenParenthesis,
    CloseParenthesis,
    LessThan,
    GreaterThan,
    Comma,
    Semicolon,

    /* Instructions */
    CreateTempBucket,
    CreateTempBucketRef,
    CloneTempBucketRef,
    DropTempBucketRef,
    CallFunction,
    CallMethod,
    CallMethodWithAllResources,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    UnexpectedEof,
    UnexpectedChar(char, usize),
    InvalidNumber(String),
    InvalidUnicode(u32),
    UnknownIdentifier(String),
}

#[derive(Debug, Clone)]
pub struct Lexer {
    /// The input text chars
    text: Vec<char>,
    /// A 0-indexed cursor indicating the next char
    current: usize,
}

pub fn tokenize(s: &str) -> Result<Vec<Token>, LexerError> {
    let mut lexer = Lexer::new(s);
    let mut tokens = Vec::new();
    loop {
        if let Some(token) = lexer.next_token()? {
            tokens.push(token);
        } else {
            break;
        }
    }
    Ok(tokens)
}

impl Lexer {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.chars().collect(),
            current: 0,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.current == self.text.len()
    }

    fn peek(&self) -> Result<char, LexerError> {
        self.text
            .get(self.current)
            .cloned()
            .ok_or(LexerError::UnexpectedEof)
    }

    fn advance(&mut self) -> Result<char, LexerError> {
        let c = self.peek()?;
        self.current += 1;
        Ok(c)
    }

    fn is_whitespace(c: char) -> bool {
        // slightly different from the original specs, we skip `\n`
        // rather than consider it as a terminal
        c == ' ' || c == '\t' || c == '\r' || c == '\n'
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, LexerError> {
        // skip whitespace
        while !self.is_eof() && Self::is_whitespace(self.peek()?) {
            self.advance()?;
        }

        // check if it's the end of file
        if self.is_eof() {
            return Ok(None);
        }

        // match next token
        match self.peek()? {
            '-' | '0'..='9' => self.tokenize_number(),
            '"' => self.tokenize_string(),
            'a'..='z' | 'A'..='Z' => self.tokenize_identifier(),
            '{' | '}' | '(' | ')' | '<' | '>' | ',' | ';' => self.tokenize_punctuation(),
            _ => Err(LexerError::UnexpectedChar(
                self.text[self.current],
                self.current,
            )),
        }
        .map(Option::from)
    }

    // TODO: consider using DFA
    fn tokenize_number(&mut self) -> Result<Token, LexerError> {
        let start = self.current;
        let mut s = String::new();

        // negative sign
        if self.peek()? == '-' {
            s.push(self.advance()?);
        }

        // integer
        match self.advance()? {
            c @ '0' => s.push(c),
            c @ '1'..='9' => {
                s.push(c);
                while self.peek()?.is_ascii_digit() {
                    s.push(self.advance()?);
                }
            }
            _ => {
                return Err(self.unexpected_char());
            }
        }

        // type
        match self.advance()? {
            'i' => match self.advance()? {
                '1' => match self.advance()? {
                    '2' => match self.advance()? {
                        '8' => Self::parse_int(&s, "i128", TokenKind::I128Literal),
                        _ => Err(self.unexpected_char()),
                    },
                    '6' => Self::parse_int(&s, "i16", TokenKind::I16Literal),
                    _ => Err(self.unexpected_char()),
                },
                '3' => match self.advance()? {
                    '2' => Self::parse_int(&s, "i32", TokenKind::I32Literal),
                    _ => Err(self.unexpected_char()),
                },
                '6' => match self.advance()? {
                    '4' => Self::parse_int(&s, "i64", TokenKind::I64Literal),
                    _ => Err(self.unexpected_char()),
                },
                '8' => Self::parse_int(&s, "i8", TokenKind::I8Literal),
                _ => Err(self.unexpected_char()),
            },
            'u' => match self.advance()? {
                '1' => match self.advance()? {
                    '2' => match self.advance()? {
                        '8' => Self::parse_int(&s, "u128", TokenKind::U128Literal),
                        _ => Err(self.unexpected_char()),
                    },
                    '6' => Self::parse_int(&s, "u16", TokenKind::U16Literal),
                    _ => Err(self.unexpected_char()),
                },
                '3' => match self.advance()? {
                    '2' => Self::parse_int(&s, "u32", TokenKind::U32Literal),
                    _ => Err(self.unexpected_char()),
                },
                '6' => match self.advance()? {
                    '4' => Self::parse_int(&s, "u64", TokenKind::U64Literal),
                    _ => Err(self.unexpected_char()),
                },
                '8' => Self::parse_int(&s, "u8", TokenKind::U8Literal),
                _ => Err(self.unexpected_char()),
            },
            _ => Err(self.unexpected_char()),
        }
        .map(|kind| self.new_token(kind, start))
    }

    fn parse_int<T: FromStr>(
        int: &str,
        ty: &str,
        map: fn(T) -> TokenKind,
    ) -> Result<TokenKind, LexerError> {
        int.parse::<T>()
            .map(map)
            .map_err(|_| LexerError::InvalidNumber(format!("{}{}", int, ty)))
    }

    fn tokenize_string(&mut self) -> Result<Token, LexerError> {
        let start = self.current;
        assert_eq!(self.advance()?, '"');

        let mut s = String::new();
        while self.peek()? != '"' {
            let c = self.advance()?;
            if c == '\\' {
                // See the JSON string specifications
                match self.advance()? {
                    '"' => s.push('\"'),
                    '\\' => s.push('\\'),
                    '/' => s.push('/'),
                    'b' => s.push('\x08'),
                    'f' => s.push('\x0c'),
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'u' => {
                        let mut unicode = self.read_utf16_unit()?;
                        if unicode >= 0xD800 && unicode <= 0xDFFF {
                            if self.advance()? == '\\' && self.advance()? == 'u' {
                                unicode = 0x10000
                                    + ((unicode - 0xD800) << 10)
                                    + (self.read_utf16_unit()? - 0xDC00);
                            } else {
                                return Err(self.unexpected_char());
                            }
                        }
                        s.push(char::from_u32(unicode).ok_or(LexerError::InvalidUnicode(unicode))?);
                    }
                    _ => {
                        return Err(self.unexpected_char());
                    }
                }
            } else {
                s.push(c);
            }
        }
        self.advance()?;

        Ok(self.new_token(TokenKind::StringLiteral(s), start))
    }

    fn read_utf16_unit(&mut self) -> Result<u32, LexerError> {
        let mut code: u32 = 0;

        for _ in 0..4 {
            let c = self.advance()?;
            if c.is_ascii_hexdigit() {
                code = code * 16 + c.to_digit(16).unwrap();
            } else {
                return Err(self.unexpected_char());
            }
        }

        Ok(code)
    }

    fn tokenize_identifier(&mut self) -> Result<Token, LexerError> {
        let start = self.current;

        let mut id = String::from(self.advance()?);
        while !self.is_eof() && (self.peek()?.is_ascii_alphanumeric() || self.peek()? == '_') {
            id.push(self.advance()?);
        }

        match id.as_str() {
            "true" => Ok(TokenKind::BoolLiteral(true)),
            "false" => Ok(TokenKind::BoolLiteral(false)),

            "Unit" => Ok(TokenKind::Unit),
            "Bool" => Ok(TokenKind::Bool),
            "I8" => Ok(TokenKind::I8),
            "I16" => Ok(TokenKind::I16),
            "I32" => Ok(TokenKind::I32),
            "I64" => Ok(TokenKind::I64),
            "I128" => Ok(TokenKind::I128),
            "U8" => Ok(TokenKind::U8),
            "U16" => Ok(TokenKind::U16),
            "U32" => Ok(TokenKind::U32),
            "U64" => Ok(TokenKind::U64),
            "U128" => Ok(TokenKind::U128),
            "String" => Ok(TokenKind::String),
            "Struct" => Ok(TokenKind::Struct),
            "Enum" => Ok(TokenKind::Enum),
            "Option" => Ok(TokenKind::Option),
            "Box" => Ok(TokenKind::Box),
            "Array" => Ok(TokenKind::Array),
            "Tuple" => Ok(TokenKind::Tuple),
            "Result" => Ok(TokenKind::Result),
            "Vec" => Ok(TokenKind::Vec),
            "TreeSet" => Ok(TokenKind::TreeSet),
            "TreeMap" => Ok(TokenKind::TreeMap),
            "HashSet" => Ok(TokenKind::HashSet),
            "HashMap" => Ok(TokenKind::HashMap),
            "Decimal" => Ok(TokenKind::Decimal),
            "BigDecimal" => Ok(TokenKind::BigDecimal),
            "Address" => Ok(TokenKind::Address),
            "Hash" => Ok(TokenKind::Hash),
            "Bucket" => Ok(TokenKind::Bucket),
            "BucketRef" => Ok(TokenKind::BucketRef),
            "LazyMap" => Ok(TokenKind::LazyMap),
            "Vault" => Ok(TokenKind::Vault),

            "Some" => Ok(TokenKind::Some),
            "None" => Ok(TokenKind::None),
            "Ok" => Ok(TokenKind::Ok),
            "Err" => Ok(TokenKind::Err),

            "CREATE_TEMP_BUCKET" => Ok(TokenKind::CreateTempBucket),
            "CREATE_TEMP_BUCKET_REF" => Ok(TokenKind::CreateTempBucketRef),
            "CLONE_TEMP_BUCKET_REF" => Ok(TokenKind::CloneTempBucketRef),
            "DROP_TEMP_BUCKET_REF" => Ok(TokenKind::DropTempBucketRef),
            "CALL_FUNCTION" => Ok(TokenKind::CallFunction),
            "CALL_METHOD" => Ok(TokenKind::CallMethod),
            "CALL_METHOD_WITH_ALL_RESOURCES" => Ok(TokenKind::CallMethodWithAllResources),

            s @ _ => Err(LexerError::UnknownIdentifier(s.into())),
        }
        .map(|kind| self.new_token(kind, start))
    }

    fn tokenize_punctuation(&mut self) -> Result<Token, LexerError> {
        let start = self.current;

        let token_kind = match self.advance()? {
            '{' => TokenKind::OpenCurlyBrace,
            '}' => TokenKind::CloseCurlyBrace,
            '(' => TokenKind::OpenParenthesis,
            ')' => TokenKind::CloseParenthesis,
            '<' => TokenKind::LessThan,
            '>' => TokenKind::GreaterThan,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            _ => {
                return Err(self.unexpected_char());
            }
        };

        Ok(self.new_token(token_kind, start))
    }

    fn new_token(&self, kind: TokenKind, start: usize) -> Token {
        Token {
            kind,
            span: Span {
                start,
                end: self.current,
            },
        }
    }

    fn unexpected_char(&self) -> LexerError {
        LexerError::UnexpectedChar(self.text[self.current - 1], self.current - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! lex_ok {
        ( $s:expr, $expected:expr ) => {{
            let mut lexer = Lexer::new($s);
            for i in 0..$expected.len() {
                assert_eq!(
                    lexer.next_token().map(|opt| opt.map(|t| t.kind)),
                    Ok(Some($expected[i].clone()))
                );
            }
            assert_eq!(lexer.next_token(), Ok(None));
        }};
    }

    #[macro_export]
    macro_rules! lex_error {
        ( $s:expr, $expected:expr ) => {{
            let mut lexer = Lexer::new($s);
            loop {
                match lexer.next_token() {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        panic!("Expected {:?} but no error is thrown", $expected);
                    }
                    Err(e) => {
                        assert_eq!(e, $expected);
                        return;
                    }
                }
            }
        }};
    }

    #[test]
    fn test_empty_strings() {
        lex_ok!("", Vec::<TokenKind>::new());
        lex_ok!("  ", Vec::<TokenKind>::new());
        lex_ok!("\r\n\t", Vec::<TokenKind>::new());
    }

    #[test]
    fn test_bool() {
        lex_ok!("true", vec![TokenKind::BoolLiteral(true)]);
        lex_ok!("false", vec![TokenKind::BoolLiteral(false)]);
        lex_error!(
            "false123u8",
            LexerError::UnknownIdentifier("false123u8".into())
        );
    }

    #[test]
    fn test_int() {
        lex_ok!(
            "1u82u1283i84i128",
            vec![
                TokenKind::U8Literal(1),
                TokenKind::U128Literal(2),
                TokenKind::I8Literal(3),
                TokenKind::I128Literal(4),
            ]
        );
        lex_ok!(
            "1u8 2u32",
            vec![TokenKind::U8Literal(1), TokenKind::U32Literal(2)]
        );
        lex_error!("123", LexerError::UnexpectedEof);
    }

    #[test]
    fn test_string() {
        lex_ok!(
            r#"  "" "abc" "abc\r\n\"def\uD83C\uDF0D"  "#,
            vec![
                TokenKind::StringLiteral("".into()),
                TokenKind::StringLiteral("abc".into()),
                TokenKind::StringLiteral("abc\r\n\"def🌍".into()),
            ]
        );
        lex_error!("\"", LexerError::UnexpectedEof);
    }

    #[test]
    fn test_mixed() {
        lex_ok!(
            r#"CALL_FUNCTION HashMap<String, Array>("test", Array<String>("abc"));"#,
            vec![
                TokenKind::CallFunction,
                TokenKind::HashMap,
                TokenKind::LessThan,
                TokenKind::String,
                TokenKind::Comma,
                TokenKind::Array,
                TokenKind::GreaterThan,
                TokenKind::OpenParenthesis,
                TokenKind::StringLiteral("test".into()),
                TokenKind::Comma,
                TokenKind::Array,
                TokenKind::LessThan,
                TokenKind::String,
                TokenKind::GreaterThan,
                TokenKind::OpenParenthesis,
                TokenKind::StringLiteral("abc".into()),
                TokenKind::CloseParenthesis,
                TokenKind::CloseParenthesis,
                TokenKind::Semicolon,
            ]
        );
    }
}
