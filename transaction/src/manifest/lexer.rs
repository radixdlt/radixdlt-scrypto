use sbor::rust::str::FromStr;

/// The span of tokens. The `start` and `end` are Unicode code points / UTF-32 - as opposed to a
/// byte-based / UTF-8 index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The start of the span, exclusive
    pub start: usize,
    /// The end of the span, inclusive
    pub end: usize,
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

    // ==============
    // SBOR basic types
    // ==============
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
    Enum,
    Array,
    Tuple,
    Map,

    // ==============
    // SBOR aliases
    // ==============
    Some,
    None,
    Ok,
    Err,
    Bytes,
    NonFungibleGlobalId,

    // ==============
    // SBOR custom types
    // ==============
    Address,
    Bucket,
    Proof,
    Expression,
    Blob,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,

    /* Punctuations */
    OpenParenthesis,
    CloseParenthesis,
    LessThan,
    GreaterThan,
    Comma,
    Semicolon,

    /* Instructions */
    TakeFromWorktop,
    TakeFromWorktopByAmount,
    TakeFromWorktopByIds,
    ReturnToWorktop,
    AssertWorktopContains,
    AssertWorktopContainsByAmount,
    AssertWorktopContainsByIds,
    PopFromAuthZone,
    PushToAuthZone,
    ClearAuthZone,
    CreateProofFromAuthZone,
    CreateProofFromAuthZoneByAmount,
    CreateProofFromAuthZoneByIds,
    CreateProofFromBucket,
    CloneProof,
    DropProof,
    DropAllProofs,
    ClearSignatureProofs,
    CallFunction,
    CallMethod,
    PublishPackage,
    PublishPackageAdvanced,
    BurnResource,
    RecallResource,
    SetMetadata,
    RemoveMetadata,
    SetPackageRoyaltyConfig,
    SetComponentRoyaltyConfig,
    ClaimPackageRoyalty,
    ClaimComponentRoyalty,
    SetMethodAccessRule,
    MintFungible,
    MintNonFungible,
    MintUuidNonFungible,
    CreateFungibleResource,
    CreateFungibleResourceWithInitialSupply,
    CreateNonFungibleResource,
    CreateNonFungibleResourceWithInitialSupply,
    CreateValidator,
    CreateAccessController,
    CreateIdentity,
    CreateIdentityAdvanced,
    CreateAccount,
    CreateAccountAdvanced,
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
    InvalidInteger(String),
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
            '{' | '}' | '(' | ')' | '<' | '>' | ',' | ';' | '&' => self.tokenize_punctuation(),
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
        .map(|kind| self.new_token(kind, start, self.current))
    }

    fn parse_int<T: FromStr>(
        int: &str,
        ty: &str,
        map: fn(T) -> TokenKind,
    ) -> Result<TokenKind, LexerError> {
        int.parse::<T>()
            .map(map)
            .map_err(|_| LexerError::InvalidInteger(format!("{}{}", int, ty)))
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

        Ok(self.new_token(TokenKind::StringLiteral(s), start, self.current))
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
            "Enum" => Ok(TokenKind::Enum),
            "Array" => Ok(TokenKind::Array),
            "Tuple" => Ok(TokenKind::Tuple),
            "Map" => Ok(TokenKind::Map),

            "Some" => Ok(TokenKind::Some),
            "None" => Ok(TokenKind::None),
            "Ok" => Ok(TokenKind::Ok),
            "Err" => Ok(TokenKind::Err),
            "Bytes" => Ok(TokenKind::Bytes),
            "NonFungibleGlobalId" => Ok(TokenKind::NonFungibleGlobalId),

            "Address" => Ok(TokenKind::Address),
            "Bucket" => Ok(TokenKind::Bucket),
            "Proof" => Ok(TokenKind::Proof),
            "Expression" => Ok(TokenKind::Expression),
            "Blob" => Ok(TokenKind::Blob),
            "Decimal" => Ok(TokenKind::Decimal),
            "PreciseDecimal" => Ok(TokenKind::PreciseDecimal),
            "NonFungibleLocalId" => Ok(TokenKind::NonFungibleLocalId),

            "TAKE_FROM_WORKTOP" => Ok(TokenKind::TakeFromWorktop),
            "TAKE_FROM_WORKTOP_BY_AMOUNT" => Ok(TokenKind::TakeFromWorktopByAmount),
            "TAKE_FROM_WORKTOP_BY_IDS" => Ok(TokenKind::TakeFromWorktopByIds),
            "RETURN_TO_WORKTOP" => Ok(TokenKind::ReturnToWorktop),
            "ASSERT_WORKTOP_CONTAINS" => Ok(TokenKind::AssertWorktopContains),
            "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT" => Ok(TokenKind::AssertWorktopContainsByAmount),
            "ASSERT_WORKTOP_CONTAINS_BY_IDS" => Ok(TokenKind::AssertWorktopContainsByIds),
            "POP_FROM_AUTH_ZONE" => Ok(TokenKind::PopFromAuthZone),
            "PUSH_TO_AUTH_ZONE" => Ok(TokenKind::PushToAuthZone),
            "CLEAR_AUTH_ZONE" => Ok(TokenKind::ClearAuthZone),
            "CREATE_PROOF_FROM_AUTH_ZONE" => Ok(TokenKind::CreateProofFromAuthZone),
            "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT" => {
                Ok(TokenKind::CreateProofFromAuthZoneByAmount)
            }
            "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS" => Ok(TokenKind::CreateProofFromAuthZoneByIds),
            "CREATE_PROOF_FROM_BUCKET" => Ok(TokenKind::CreateProofFromBucket),
            "CLONE_PROOF" => Ok(TokenKind::CloneProof),
            "DROP_PROOF" => Ok(TokenKind::DropProof),
            "DROP_ALL_PROOFS" => Ok(TokenKind::DropAllProofs),
            "CLEAR_SIGNATURE_PROOFS" => Ok(TokenKind::ClearSignatureProofs),
            "CALL_FUNCTION" => Ok(TokenKind::CallFunction),
            "CALL_METHOD" => Ok(TokenKind::CallMethod),
            "PUBLISH_PACKAGE" => Ok(TokenKind::PublishPackage),
            "PUBLISH_PACKAGE_ADVANCED" => Ok(TokenKind::PublishPackageAdvanced),
            "BURN_RESOURCE" => Ok(TokenKind::BurnResource),
            "RECALL_RESOURCE" => Ok(TokenKind::RecallResource),
            "SET_METADATA" => Ok(TokenKind::SetMetadata),
            "REMOVE_METADATA" => Ok(TokenKind::RemoveMetadata),
            "SET_PACKAGE_ROYALTY_CONFIG" => Ok(TokenKind::SetPackageRoyaltyConfig),
            "SET_COMPONENT_ROYALTY_CONFIG" => Ok(TokenKind::SetComponentRoyaltyConfig),
            "CLAIM_PACKAGE_ROYALTY" => Ok(TokenKind::ClaimPackageRoyalty),
            "CLAIM_COMPONENT_ROYALTY" => Ok(TokenKind::ClaimComponentRoyalty),
            "SET_METHOD_ACCESS_RULE" => Ok(TokenKind::SetMethodAccessRule),
            "MINT_FUNGIBLE" => Ok(TokenKind::MintFungible),
            "MINT_NON_FUNGIBLE" => Ok(TokenKind::MintNonFungible),
            "MINT_UUID_NON_FUNGIBLE" => Ok(TokenKind::MintUuidNonFungible),
            "CREATE_FUNGIBLE_RESOURCE" => Ok(TokenKind::CreateFungibleResource),
            "CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY" => {
                Ok(TokenKind::CreateFungibleResourceWithInitialSupply)
            }
            "CREATE_NON_FUNGIBLE_RESOURCE" => Ok(TokenKind::CreateNonFungibleResource),
            "CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY" => {
                Ok(TokenKind::CreateNonFungibleResourceWithInitialSupply)
            }
            "CREATE_VALIDATOR" => Ok(TokenKind::CreateValidator),
            "CREATE_IDENTITY" => Ok(TokenKind::CreateIdentity),
            "CREATE_IDENTITY_ADVANCED" => Ok(TokenKind::CreateIdentityAdvanced),
            "CREATE_ACCOUNT" => Ok(TokenKind::CreateAccount),
            "CREATE_ACCOUNT_ADVANCED" => Ok(TokenKind::CreateAccountAdvanced),
            "CREATE_ACCESS_CONTROLLER" => Ok(TokenKind::CreateAccessController),

            s @ _ => Err(LexerError::UnknownIdentifier(s.into())),
        }
        .map(|kind| self.new_token(kind, start, self.current))
    }

    fn tokenize_punctuation(&mut self) -> Result<Token, LexerError> {
        let start = self.current;

        let token_kind = match self.advance()? {
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

        Ok(self.new_token(token_kind, start, self.current))
    }

    fn new_token(&self, kind: TokenKind, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: Span { start, end },
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
    fn test_comment() {
        lex_ok!("# 1u8", Vec::<TokenKind>::new());
        lex_ok!("1u8 # comment", vec![TokenKind::U8Literal(1),]);
        lex_ok!(
            "# multiple\n# line\nCALL_FUNCTION",
            vec![TokenKind::CallFunction,]
        );
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
            r#"CALL_FUNCTION Map<String, Array>("test", Array<String>("abc"));"#,
            vec![
                TokenKind::CallFunction,
                TokenKind::Map,
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

    #[test]
    fn test_precise_decimal() {
        lex_ok!(
            "PreciseDecimal(\"12\")",
            vec![
                TokenKind::PreciseDecimal,
                TokenKind::OpenParenthesis,
                TokenKind::StringLiteral("12".into()),
                TokenKind::CloseParenthesis,
            ]
        );
    }

    #[test]
    fn test_precise_decimal_colletion() {
        lex_ok!(
            "Array<PreciseDecimal>(PreciseDecimal(\"12\"), PreciseDecimal(\"212\"), PreciseDecimal(\"1984\"))",
            vec![
                TokenKind::Array,
                TokenKind::LessThan,
                TokenKind::PreciseDecimal,
                TokenKind::GreaterThan,
                TokenKind::OpenParenthesis,
                TokenKind::PreciseDecimal,
                TokenKind::OpenParenthesis,
                TokenKind::StringLiteral("12".into()),
                TokenKind::CloseParenthesis,
                TokenKind::Comma,
                TokenKind::PreciseDecimal,
                TokenKind::OpenParenthesis,
                TokenKind::StringLiteral("212".into()),
                TokenKind::CloseParenthesis,
                TokenKind::Comma,
                TokenKind::PreciseDecimal,
                TokenKind::OpenParenthesis,
                TokenKind::StringLiteral("1984".into()),
                TokenKind::CloseParenthesis,
                TokenKind::CloseParenthesis,
            ]
        );
    }
}
