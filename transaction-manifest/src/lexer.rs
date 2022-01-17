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
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    String(String),

    /* Keywords */
    Unit,
    True,
    False,
    Struct,
    Enum,
    Some,
    None,
    Box,
    Ok,
    Err,
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

    /* Punctuations */
    OpenParenthesis,
    CloseParenthesis,
    OpenBracket,
    CloseBracket,
    Comma,
    Semicolon,

    /* Instructions */
    DeclareTempBucket,
    DeclareTempBucketRef,
    TakeFromContext,
    BorrowFromContext,
    CallFunction,
    CallMethod,
    DropAllBucketRefs,
    DepositAllBuckets,
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
            '(' | ')' | '[' | ']' | ',' | ';' => self.tokenize_punctuation(),
            _ => Err(self.unexpected_char()),
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
                        '8' => Self::parse_int(&s, "i128", TokenKind::I128),
                        _ => Err(self.unexpected_char()),
                    },
                    '6' => Self::parse_int(&s, "i16", TokenKind::I16),
                    _ => Err(self.unexpected_char()),
                },
                '3' => match self.advance()? {
                    '2' => Self::parse_int(&s, "i32", TokenKind::I32),
                    _ => Err(self.unexpected_char()),
                },
                '6' => match self.advance()? {
                    '4' => Self::parse_int(&s, "i64", TokenKind::I64),
                    _ => Err(self.unexpected_char()),
                },
                '8' => Self::parse_int(&s, "i8", TokenKind::I8),
                _ => Err(self.unexpected_char()),
            },
            'u' => match self.advance()? {
                '1' => match self.advance()? {
                    '2' => match self.advance()? {
                        '8' => Self::parse_int(&s, "u128", TokenKind::U128),
                        _ => Err(self.unexpected_char()),
                    },
                    '6' => Self::parse_int(&s, "u16", TokenKind::U16),
                    _ => Err(self.unexpected_char()),
                },
                '3' => match self.advance()? {
                    '2' => Self::parse_int(&s, "u32", TokenKind::U32),
                    _ => Err(self.unexpected_char()),
                },
                '6' => match self.advance()? {
                    '4' => Self::parse_int(&s, "u64", TokenKind::U64),
                    _ => Err(self.unexpected_char()),
                },
                '8' => Self::parse_int(&s, "u8", TokenKind::U8),
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

        Ok(self.new_token(TokenKind::String(s), start))
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
            "unit" => Ok(TokenKind::Unit),
            "true" => Ok(TokenKind::True),
            "false" => Ok(TokenKind::False),
            "struct" => Ok(TokenKind::Struct),
            "enum" => Ok(TokenKind::Enum),
            "some" => Ok(TokenKind::Some),
            "none" => Ok(TokenKind::None),
            "box" => Ok(TokenKind::Box),
            "ok" => Ok(TokenKind::Ok),
            "err" => Ok(TokenKind::Err),
            "vec" => Ok(TokenKind::Vec),
            "tree_set" => Ok(TokenKind::TreeSet),
            "tree_map" => Ok(TokenKind::TreeMap),
            "hash_set" => Ok(TokenKind::HashSet),
            "hash_map" => Ok(TokenKind::HashMap),
            "decimal" => Ok(TokenKind::Decimal),
            "big_decimal" => Ok(TokenKind::BigDecimal),
            "address" => Ok(TokenKind::Address),
            "hash" => Ok(TokenKind::Hash),
            "bucket" => Ok(TokenKind::Bucket),
            "bucket_ref" => Ok(TokenKind::BucketRef),
            "lazy_map" => Ok(TokenKind::LazyMap),
            "vault" => Ok(TokenKind::Vault),
            "DECLARE_TEMP_BUCKET" => Ok(TokenKind::DeclareTempBucket),
            "DECLARE_TEMP_BUCKET_REF" => Ok(TokenKind::DeclareTempBucketRef),
            "TAKE_FROM_CONTEXT" => Ok(TokenKind::TakeFromContext),
            "BORROW_FROM_CONTEXT" => Ok(TokenKind::BorrowFromContext),
            "CALL_FUNCTION" => Ok(TokenKind::CallFunction),
            "CALL_METHOD" => Ok(TokenKind::CallMethod),
            "DROP_ALL_BUCKET_REFS" => Ok(TokenKind::DropAllBucketRefs),
            "DEPOSIT_ALL_BUCKETS" => Ok(TokenKind::DepositAllBuckets),
            s @ _ => Err(LexerError::UnknownIdentifier(s.to_string())),
        }
        .map(|kind| self.new_token(kind, start))
    }

    fn tokenize_punctuation(&mut self) -> Result<Token, LexerError> {
        let start = self.current;

        let token_kind = match self.advance()? {
            '(' => TokenKind::OpenParenthesis,
            ')' => TokenKind::CloseParenthesis,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            _ => panic!("Illegal state"),
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
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn parse_ok(s: &str, expected: Vec<TokenKind>) {
        let mut lexer = Lexer::new(s);
        for i in 0..expected.len() {
            assert_eq!(
                lexer.next_token().map(|opt| opt.map(|t| t.kind)),
                Ok(Some(expected[i].clone()))
            );
        }
        assert_eq!(lexer.next_token(), Ok(None));
    }

    fn parse_error(s: &str, expected: LexerError) {
        let mut lexer = Lexer::new(s);
        loop {
            match lexer.next_token() {
                Ok(Some(_)) => {}
                Ok(None) => {
                    panic!("Expected {:?} but no error is thrown", expected);
                }
                Err(e) => {
                    assert_eq!(e, expected);
                    return;
                }
            }
        }
    }

    #[test]
    fn test_empty_strings() {
        parse_ok("", vec![]);
        parse_ok("  ", vec![]);
        parse_ok("\r\n\t", vec![]);
    }

    #[test]
    fn test_unit_bool() {
        parse_ok("unit", vec![TokenKind::Unit]);
        parse_ok("true", vec![TokenKind::True]);
        parse_ok("false", vec![TokenKind::False]);
        parse_error(
            "false123u8",
            LexerError::UnknownIdentifier("false123u8".to_string()),
        );
    }

    #[test]
    fn test_int() {
        parse_ok(
            "1u82u1283i84i128",
            vec![
                TokenKind::U8(1),
                TokenKind::U128(2),
                TokenKind::I8(3),
                TokenKind::I128(4),
            ],
        );
        parse_ok("1u8 2u32", vec![TokenKind::U8(1), TokenKind::U32(2)]);
        parse_error("123", LexerError::UnexpectedEof);
    }

    #[test]
    fn test_string() {
        parse_ok(
            r#"  "" "abc" "abc\r\n\"def\uD83C\uDF0D"  "#,
            vec![
                TokenKind::String("".to_string()),
                TokenKind::String("abc".to_string()),
                TokenKind::String("abc\r\n\"def🌍".to_string()),
            ],
        );
        parse_error("\"", LexerError::UnexpectedEof);
    }

    #[test]
    fn test_keyword_and_punctuation() {
        parse_ok(
            r#"CALL_FUNCTION vec(hash_map(), ["abc"]);"#,
            vec![
                TokenKind::CallFunction,
                TokenKind::Vec,
                TokenKind::OpenParenthesis,
                TokenKind::HashMap,
                TokenKind::OpenParenthesis,
                TokenKind::CloseParenthesis,
                TokenKind::Comma,
                TokenKind::OpenBracket,
                TokenKind::String("abc".to_owned()),
                TokenKind::CloseBracket,
                TokenKind::CloseParenthesis,
                TokenKind::Semicolon,
            ],
        );
    }
}
