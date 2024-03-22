use crate::manifest::compiler::CompileErrorDiagnosticsStyle;
use crate::manifest::diagnostic_snippets::create_snippet;
use crate::manifest::token::{Position, Span, Token, TokenWithSpan};
use sbor::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedChar {
    Exact(char),
    OneOf(Vec<char>),
    HexDigit,
    DigitLetterQuotePunctuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerErrorKind {
    UnexpectedEof,
    UnexpectedChar(char, ExpectedChar),
    InvalidIntegerLiteral(String),
    InvalidIntegerType(String),
    InvalidInteger(String),
    InvalidUnicode(u32),
    MissingUnicodeSurrogate(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerError {
    pub error_kind: LexerErrorKind,
    pub span: Span,
}

impl LexerError {
    fn unexpected_char(position: Position, c: char, expected: ExpectedChar) -> Self {
        Self {
            error_kind: LexerErrorKind::UnexpectedChar(c, expected),
            span: Span {
                start: position,
                end: position.advance(c),
            },
        }
    }

    fn invalid_integer_type(ty: String, start: Position, end: Position) -> Self {
        Self {
            error_kind: LexerErrorKind::InvalidIntegerType(ty),
            span: Span { start, end },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    /// The input text chars
    text: Vec<char>,
    /// The current position in the text (in case of end of file it equals to text length)
    current: Position,
}

pub fn tokenize(s: &str) -> Result<Vec<TokenWithSpan>, LexerError> {
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
            current: Position {
                full_index: 0,
                line_idx: 0,
                line_char_index: 0,
            },
        }
    }

    pub fn is_eof(&self) -> bool {
        self.current.full_index == self.text.len()
    }

    fn peek(&self) -> Result<char, LexerError> {
        if self.is_eof() {
            Err(LexerError {
                error_kind: LexerErrorKind::UnexpectedEof,
                span: Span {
                    start: self.current,
                    end: self.current,
                },
            })
        } else {
            Ok(self.text[self.current.full_index])
        }
    }

    fn advance(&mut self) -> Result<char, LexerError> {
        let c = self.peek()?;
        self.current = self.current.advance(c);
        Ok(c)
    }

    fn advance_expected(&mut self, expected: char) -> Result<char, LexerError> {
        self.advance_matching(|c| c == expected, ExpectedChar::Exact(expected))
    }

    fn advance_matching(
        &mut self,
        matcher: impl Fn(char) -> bool,
        expected: ExpectedChar,
    ) -> Result<char, LexerError> {
        let previous = self.current;
        let c = self.advance()?;
        if !matcher(c) {
            Err(LexerError::unexpected_char(previous, c, expected))
        } else {
            Ok(c)
        }
    }

    fn advance_and_append(&mut self, s: &mut String) -> Result<char, LexerError> {
        let c = self.advance()?;
        s.push(c);
        Ok(c)
    }

    fn is_whitespace(c: char) -> bool {
        // slightly different from the original specs, we skip `\n`
        // rather than consider it as a terminal
        c == ' ' || c == '\t' || c == '\r' || c == '\n'
    }

    pub fn next_token(&mut self) -> Result<Option<TokenWithSpan>, LexerError> {
        // skip comment and whitespace
        let mut in_comment = false;
        while !self.is_eof() {
            if in_comment {
                if self.advance()? == '\n' {
                    in_comment = false;
                }
            } else if self.peek()? == '#' {
                in_comment = true;
            } else if Self::is_whitespace(self.peek()?) {
                self.advance()?;
            } else {
                break;
            }
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
            '{' | '}' | '(' | ')' | '<' | '>' | ',' | ';' | '&' | '=' => {
                self.tokenize_punctuation()
            }
            c => Err(LexerError::unexpected_char(
                self.current,
                c,
                ExpectedChar::DigitLetterQuotePunctuation,
            )),
        }
        .map(Option::from)
    }

    // TODO: consider using DFA
    fn tokenize_number(&mut self) -> Result<TokenWithSpan, LexerError> {
        let literal_start = self.current;
        let mut s = String::new();

        // negative sign
        if self.peek()? == '-' {
            s.push(self.advance()?);
        }

        // integer
        match self.advance_and_append(&mut s)? {
            '0' => {}
            '1'..='9' => {
                while self.peek()?.is_ascii_digit() {
                    s.push(self.advance()?);
                }
            }
            _ => {
                return Err(LexerError {
                    error_kind: LexerErrorKind::InvalidIntegerLiteral(s),
                    span: Span {
                        start: literal_start,
                        end: self.current,
                    },
                });
            }
        }

        // type
        let ty_start = self.current;
        let mut t = String::new();
        match self.advance_and_append(&mut t)? {
            'i' => match self.advance_and_append(&mut t)? {
                '1' => match self.advance_and_append(&mut t)? {
                    '2' => match self.advance_and_append(&mut t)? {
                        '8' => self.parse_int(&s, "i128", Token::I128Literal, literal_start),
                        _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                    },
                    '6' => self.parse_int(&s, "i16", Token::I16Literal, literal_start),
                    _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                },
                '3' => match self.advance_and_append(&mut t)? {
                    '2' => self.parse_int(&s, "i32", Token::I32Literal, literal_start),
                    _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                },
                '6' => match self.advance_and_append(&mut t)? {
                    '4' => self.parse_int(&s, "i64", Token::I64Literal, literal_start),
                    _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                },
                '8' => self.parse_int(&s, "i8", Token::I8Literal, literal_start),
                _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
            },
            'u' => match self.advance_and_append(&mut t)? {
                '1' => match self.advance_and_append(&mut t)? {
                    '2' => match self.advance_and_append(&mut t)? {
                        '8' => self.parse_int(&s, "u128", Token::U128Literal, literal_start),
                        _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                    },
                    '6' => self.parse_int(&s, "u16", Token::U16Literal, literal_start),
                    _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                },
                '3' => match self.advance_and_append(&mut t)? {
                    '2' => self.parse_int(&s, "u32", Token::U32Literal, literal_start),
                    _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                },
                '6' => match self.advance_and_append(&mut t)? {
                    '4' => self.parse_int(&s, "u64", Token::U64Literal, literal_start),
                    _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
                },
                '8' => self.parse_int(&s, "u8", Token::U8Literal, literal_start),
                _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
            },
            _ => Err(LexerError::invalid_integer_type(t, ty_start, self.current)),
        }
        .map(|token| self.new_token(token, literal_start, self.current))
    }

    fn parse_int<T>(
        &self,
        int: &str,
        ty: &str,
        map: fn(T) -> Token,
        token_start: Position,
    ) -> Result<Token, LexerError>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        int.parse::<T>().map(map).map_err(|err| LexerError {
            error_kind: LexerErrorKind::InvalidInteger(format!(
                "'{}{}' - {}",
                int,
                ty,
                err.to_string()
            )),
            span: Span {
                start: token_start,
                end: self.current,
            },
        })
    }

    fn tokenize_string(&mut self) -> Result<TokenWithSpan, LexerError> {
        let start = self.current;
        assert_eq!(self.advance()?, '"');

        let mut s = String::new();
        while self.peek()? != '"' {
            let c = self.advance()?;
            if c == '\\' {
                // Remember '\\' position
                let token_start = self.current;

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
                        // Check unicode surrogate pair
                        // (see https://unicodebook.readthedocs.io/unicode_encodings.html#surrogates)
                        if (0xD800..=0xDFFF).contains(&unicode) {
                            let position = self.current;
                            if self.advance()? == '\\' && self.advance()? == 'u' {
                                unicode = 0x10000
                                    + ((unicode - 0xD800) << 10)
                                    + self.read_utf16_unit()?
                                    - 0xDC00;
                            } else {
                                return Err(LexerError {
                                    error_kind: LexerErrorKind::MissingUnicodeSurrogate(unicode),
                                    span: Span {
                                        start: token_start,
                                        end: position,
                                    },
                                });
                            }
                        }
                        s.push(char::from_u32(unicode).ok_or(LexerError {
                            error_kind: LexerErrorKind::InvalidUnicode(unicode),
                            span: Span {
                                start: token_start,
                                end: self.current,
                            },
                        })?);
                    }
                    c => {
                        return Err(LexerError::unexpected_char(
                            token_start,
                            c,
                            ExpectedChar::OneOf(vec!['"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u']),
                        ));
                    }
                }
            } else {
                s.push(c);
            }
        }
        self.advance()?;

        Ok(self.new_token(Token::StringLiteral(s), start, self.current))
    }

    fn read_utf16_unit(&mut self) -> Result<u32, LexerError> {
        let mut code: u32 = 0;

        for _ in 0..4 {
            let c = self.advance_matching(|c| c.is_ascii_hexdigit(), ExpectedChar::HexDigit)?;
            code = code * 16 + c.to_digit(16).unwrap();
        }

        Ok(code)
    }

    fn tokenize_identifier(&mut self) -> Result<TokenWithSpan, LexerError> {
        let start = self.current;

        let mut id = String::from(self.advance()?);
        while !self.is_eof() {
            let next_char = self.peek()?;
            let next_char_can_be_part_of_ident =
                next_char.is_ascii_alphanumeric() || next_char == '_' || next_char == ':';
            if !next_char_can_be_part_of_ident {
                break;
            }
            id.push(self.advance()?);
        }

        let token = match id.as_str() {
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            other => Token::Ident(other.to_string()),
        };
        Ok(self.new_token(token, start, self.current))
    }

    fn tokenize_punctuation(&mut self) -> Result<TokenWithSpan, LexerError> {
        let token_start = self.current;

        let token = match self.advance()? {
            '(' => Token::OpenParenthesis,
            ')' => Token::CloseParenthesis,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '=' => {
                self.advance_expected('>')?;
                Token::FatArrow
            }
            c => {
                return Err(LexerError::unexpected_char(
                    token_start,
                    c,
                    ExpectedChar::OneOf(vec!['(', ')', '<', '>', ',', ';', '=']),
                ))
            }
        };

        Ok(self.new_token(token, token_start, self.current))
    }

    fn new_token(&self, token: Token, start: Position, end: Position) -> TokenWithSpan {
        TokenWithSpan {
            token,
            span: Span { start, end },
        }
    }
}

pub fn lexer_error_diagnostics(
    s: &str,
    err: LexerError,
    style: CompileErrorDiagnosticsStyle,
) -> String {
    let (title, label) = match err.error_kind {
        LexerErrorKind::UnexpectedEof => (
            "unexpected end of file".to_string(),
            "unexpected end of file".to_string(),
        ),
        LexerErrorKind::UnexpectedChar(c, expected) => {
            let expected = match expected {
                ExpectedChar::Exact(exact) => format!("'{}'", exact),
                ExpectedChar::OneOf(one_of) => {
                    let v: Vec<String> = one_of.iter().map(|c| format!("'{}'", c)).collect();
                    if let Some((last, init)) =  v.split_last() {
                        format!("{} or {}", init.join(", "), last)
                    }
                    else {
                        "unknown".to_string()
                    }
                }
                ExpectedChar::HexDigit => "hex digit".to_string(),
                ExpectedChar::DigitLetterQuotePunctuation => "digit, letter, quotation mark or one of punctuation characters '(', ')', '<', '>', ',', ';', '='".to_string(),
            };
            (
                format!("unexpected character {:?}, expected {}", c, expected),
                "unexpected character".to_string(),
            )
        }
        LexerErrorKind::InvalidIntegerLiteral(string) => (
            format!("invalid integer literal '{}'", string),
            "invalid integer literal".to_string(),
        ),
        LexerErrorKind::InvalidIntegerType(string) => (
            format!("invalid integer type '{}'", string),
            "invalid integer type".to_string(),
        ),
        LexerErrorKind::InvalidInteger(string) => (
            format!("invalid integer value {}", string),
            "invalid integer value".to_string(),
        ),
        LexerErrorKind::InvalidUnicode(value) => (
            format!("invalid unicode code point {}", value),
            "invalid unicode code point".to_string(),
        ),
        LexerErrorKind::MissingUnicodeSurrogate(value) => (
            format!("missing unicode '{:X}' surrogate pair", value),
            "missing unicode surrogate pair".to_string(),
        ),
    };
    create_snippet(s, &err.span, &title, &label, style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position, span};

    #[macro_export]
    macro_rules! lex_ok {
        ( $s:expr, $expected:expr ) => {{
            let mut lexer = Lexer::new($s);
            for i in 0..$expected.len() {
                assert_eq!(
                    lexer.next_token().map(|opt| opt.map(|t| t.token)),
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
                        break;
                    }
                }
            }
        }};
    }

    #[test]
    fn test_empty_strings() {
        lex_ok!("", Vec::<Token>::new());
        lex_ok!("  ", Vec::<Token>::new());
        lex_ok!("\r\n\t", Vec::<Token>::new());
    }

    #[test]
    fn test_bool() {
        lex_ok!("true", vec![Token::BoolLiteral(true)]);
        lex_ok!("false", vec![Token::BoolLiteral(false)]);
        lex_ok!("false123u8", vec![Token::Ident("false123u8".into())]);
    }

    #[test]
    fn test_int() {
        lex_ok!(
            "1u82u1283i84i128",
            vec![
                Token::U8Literal(1),
                Token::U128Literal(2),
                Token::I8Literal(3),
                Token::I128Literal(4),
            ]
        );
        lex_ok!("1u8 2u32", vec![Token::U8Literal(1), Token::U32Literal(2)]);
        lex_error!(
            "123",
            LexerError {
                error_kind: LexerErrorKind::UnexpectedEof,
                span: span!(start = (3, 0, 3), end = (3, 0, 3))
            }
        );
    }

    #[test]
    fn test_comment() {
        lex_ok!("# 1u8", Vec::<Token>::new());
        lex_ok!("1u8 # comment", vec![Token::U8Literal(1),]);
        lex_ok!(
            "# multiple\n# line\nCALL_FUNCTION",
            vec![Token::Ident("CALL_FUNCTION".to_string()),]
        );
    }

    #[test]
    fn test_string() {
        lex_ok!(
            r#"  "" "abc" "abc\r\n\"def\uD83C\uDF0D"  "#,
            vec![
                Token::StringLiteral("".into()),
                Token::StringLiteral("abc".into()),
                Token::StringLiteral("abc\r\n\"def🌍".into()),
            ]
        );
        lex_error!(
            "\"",
            LexerError {
                error_kind: LexerErrorKind::UnexpectedEof,
                span: span!(start = (1, 0, 1), end = (1, 0, 1))
            }
        );
    }

    #[test]
    fn test_mixed() {
        lex_ok!(
            r#"CALL_FUNCTION Map<String, Array>("test", Array<String>("abc"));"#,
            vec![
                Token::Ident("CALL_FUNCTION".to_string()),
                Token::Ident("Map".to_string()),
                Token::LessThan,
                Token::Ident("String".to_string()),
                Token::Comma,
                Token::Ident("Array".to_string()),
                Token::GreaterThan,
                Token::OpenParenthesis,
                Token::StringLiteral("test".into()),
                Token::Comma,
                Token::Ident("Array".to_string()),
                Token::LessThan,
                Token::Ident("String".to_string()),
                Token::GreaterThan,
                Token::OpenParenthesis,
                Token::StringLiteral("abc".into()),
                Token::CloseParenthesis,
                Token::CloseParenthesis,
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn test_precise_decimal() {
        lex_ok!(
            "PreciseDecimal(\"12\")",
            vec![
                Token::Ident("PreciseDecimal".to_string()),
                Token::OpenParenthesis,
                Token::StringLiteral("12".into()),
                Token::CloseParenthesis,
            ]
        );
    }

    #[test]
    fn test_precise_decimal_collection() {
        lex_ok!(
            "Array<PreciseDecimal>(PreciseDecimal(\"12\"), PreciseDecimal(\"212\"), PreciseDecimal(\"1984\"))",
            vec![
                Token::Ident("Array".to_string()),
                Token::LessThan,
                Token::Ident("PreciseDecimal".to_string()),
                Token::GreaterThan,
                Token::OpenParenthesis,
                Token::Ident("PreciseDecimal".to_string()),
                Token::OpenParenthesis,
                Token::StringLiteral("12".into()),
                Token::CloseParenthesis,
                Token::Comma,
                Token::Ident("PreciseDecimal".to_string()),
                Token::OpenParenthesis,
                Token::StringLiteral("212".into()),
                Token::CloseParenthesis,
                Token::Comma,
                Token::Ident("PreciseDecimal".to_string()),
                Token::OpenParenthesis,
                Token::StringLiteral("1984".into()),
                Token::CloseParenthesis,
                Token::CloseParenthesis,
            ]
        );
    }

    #[test]
    fn test_invalid_integer() {
        lex_error!(
            "-_28u32",
            LexerError {
                error_kind: LexerErrorKind::InvalidIntegerLiteral("-_".to_string()),
                span: span!(start = (0, 0, 0), end = (2, 0, 2))
            }
        );

        lex_error!(
            "1i128\n 1u64 \n 1i37",
            LexerError {
                error_kind: LexerErrorKind::InvalidIntegerType("i37".to_string()),
                span: span!(start = (15, 2, 2), end = (18, 2, 5))
            }
        );

        lex_error!(
            "3_0i8",
            LexerError {
                error_kind: LexerErrorKind::InvalidIntegerType("_".to_string()),
                span: span!(start = (1, 0, 1), end = (2, 0, 2))
            }
        );
    }

    #[test]
    fn test_unexpected_char() {
        lex_error!(
            "1u8 +2u32",
            LexerError {
                error_kind: LexerErrorKind::UnexpectedChar(
                    '+',
                    ExpectedChar::DigitLetterQuotePunctuation
                ),
                span: span!(start = (4, 0, 4), end = (5, 0, 5))
            }
        );

        lex_error!(
            "x=7",
            LexerError {
                error_kind: LexerErrorKind::UnexpectedChar('7', ExpectedChar::Exact('>')),
                span: span!(start = (2, 0, 2), end = (3, 0, 3))
            }
        );
    }

    #[test]
    fn test_unicode() {
        lex_ok!(r#""\u2764""#, vec![Token::StringLiteral("❤".to_string())]);
        lex_ok!(r#""\uFA84""#, vec![Token::StringLiteral("彩".to_string())]);
        lex_ok!(
            r#""\uD83D\uDC69""#,
            vec![Token::StringLiteral("👩".to_string())]
        );
        lex_ok!(r#""👩""#, vec![Token::StringLiteral("👩".to_string())]);
        lex_error!(
            r#""\uDCAC\u1234""#,
            LexerError {
                error_kind: LexerErrorKind::InvalidUnicode(1238580),
                span: span!(start = (2, 0, 2), end = (13, 0, 13))
            }
        );
    }
}
