use crate::ast::{Fields, Instruction, Transaction, Type, Value};
use crate::lexer::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedEof,
    UnexpectedToken(Token),
    InvalidNumberOfValues { actual: usize, expected: usize },
    InvalidNumberOfTypes { actual: usize, expected: usize },
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[macro_export]
macro_rules! advance_ok {
    ( $self:expr, $v:expr ) => {{
        $self.advance()?;
        Ok($v)
    }};
}

#[macro_export]
macro_rules! advance_match {
    ( $self:expr, $expected:expr ) => {{
        let token = $self.advance()?;
        if token.kind != $expected {
            return Err(ParserError::UnexpectedToken(token));
        }
    }};
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn is_eof(&self) -> bool {
        self.current == self.tokens.len()
    }

    pub fn peek(&mut self) -> Result<Token, ParserError> {
        self.tokens
            .get(self.current)
            .cloned()
            .ok_or(ParserError::UnexpectedEof)
    }

    pub fn advance(&mut self) -> Result<Token, ParserError> {
        let token = self.peek()?;
        self.current += 1;
        Ok(token)
    }

    pub fn parse_transaction(&mut self) -> Result<Transaction, ParserError> {
        let mut instructions = Vec::<Instruction>::new();

        while !self.is_eof() {
            let token = self.advance()?;
            match token.kind {
                TokenKind::DeclareTempBucket => {
                    instructions.push(Instruction::DeclareTempBucket {
                        name: self.parse_value()?,
                    });
                }
                TokenKind::DeclareTempBucketRef => {
                    instructions.push(Instruction::DeclareTempBucketRef {
                        name: self.parse_value()?,
                    });
                }
                TokenKind::TakeFromContext => {
                    instructions.push(Instruction::TakeFromContext {
                        amount: self.parse_value()?,
                        resource_address: self.parse_value()?,
                        to: self.parse_value()?,
                    });
                }
                TokenKind::BorrowFromContext => {
                    instructions.push(Instruction::BorrowFromContext {
                        amount: self.parse_value()?,
                        resource_address: self.parse_value()?,
                        to: self.parse_value()?,
                    });
                }
                TokenKind::CallFunction => {
                    instructions.push(Instruction::CallFunction {
                        package_address: self.parse_value()?,
                        blueprint_name: self.parse_value()?,
                        function: self.parse_value()?,
                        args: {
                            let mut values = vec![];
                            while self.peek()?.kind != TokenKind::Semicolon {
                                values.push(self.parse_value()?);
                            }
                            values
                        },
                    });
                }
                TokenKind::CallMethod => {
                    instructions.push(Instruction::CallMethod {
                        component_address: self.parse_value()?,
                        method: self.parse_value()?,
                        args: {
                            let mut values = vec![];
                            while self.peek()?.kind != TokenKind::Semicolon {
                                values.push(self.parse_value()?);
                            }
                            values
                        },
                    });
                }
                TokenKind::DropAllBucketRefs => {
                    instructions.push(Instruction::DropAllBucketRefs);
                }
                TokenKind::DepositAllBuckets => {
                    instructions.push(Instruction::DepositAllBuckets {
                        account: self.parse_value()?,
                    });
                }
                _ => {
                    return Err(ParserError::UnexpectedToken(token));
                }
            }
        }

        Ok(Transaction { instructions })
    }

    pub fn parse_value(&mut self) -> Result<Value, ParserError> {
        let token = self.peek()?;
        match token.kind {
            TokenKind::BoolLiteral(value) => advance_ok!(self, Value::Bool(value)),
            TokenKind::U8Literal(value) => advance_ok!(self, Value::U8(value)),
            TokenKind::U16Literal(value) => advance_ok!(self, Value::U16(value)),
            TokenKind::U32Literal(value) => advance_ok!(self, Value::U32(value)),
            TokenKind::U64Literal(value) => advance_ok!(self, Value::U64(value)),
            TokenKind::U128Literal(value) => advance_ok!(self, Value::U128(value)),
            TokenKind::I8Literal(value) => advance_ok!(self, Value::I8(value)),
            TokenKind::I16Literal(value) => advance_ok!(self, Value::I16(value)),
            TokenKind::I32Literal(value) => advance_ok!(self, Value::I32(value)),
            TokenKind::I64Literal(value) => advance_ok!(self, Value::I64(value)),
            TokenKind::I128Literal(value) => advance_ok!(self, Value::I128(value)),
            TokenKind::StringLiteral(value) => advance_ok!(self, Value::String(value)),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Some | TokenKind::None => self.parse_option(),
            TokenKind::Box => self.parse_box(),
            TokenKind::Array => self.parse_array(),
            TokenKind::Tuple => self.parse_tuple(),
            TokenKind::Ok | TokenKind::Err => self.parse_result(),
            TokenKind::Vec => self.parse_vec(),
            TokenKind::TreeSet => self.parse_tree_set(),
            TokenKind::TreeMap => self.parse_tree_map(),
            TokenKind::HashSet => self.parse_hash_set(),
            TokenKind::HashMap => self.parse_hash_map(),
            TokenKind::Decimal
            | TokenKind::BigDecimal
            | TokenKind::Address
            | TokenKind::Hash
            | TokenKind::Bucket
            | TokenKind::BucketRef
            | TokenKind::LazyMap
            | TokenKind::Vault => self.parse_scrypto_types(),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }
    pub fn parse_struct(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Struct);

        if self.is_eof() {
            return Ok(Value::Struct(Fields::Unit));
        }

        Ok(Value::Struct(match self.peek()?.kind {
            TokenKind::OpenCurlyBrace => Fields::Named(
                self.parse_values_any(TokenKind::OpenCurlyBrace, TokenKind::CloseCurlyBrace)?,
            ),
            TokenKind::OpenParenthesis => Fields::Unnamed(
                self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
            ),
            _ => Fields::Unit,
        }))
    }

    pub fn parse_enum(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Enum);
        advance_match!(self, TokenKind::OpenParenthesis);

        // parse the index
        let token = self.advance()?;
        let index = if let TokenKind::U8Literal(value) = token.kind {
            value
        } else {
            return Err(ParserError::UnexpectedToken(token));
        };

        // parse named/unnamed fields
        let fields = if let TokenKind::Comma = self.peek()?.kind {
            self.advance()?;
            let t = self.peek()?;
            match t.kind {
                TokenKind::OpenCurlyBrace => Fields::Named(
                    self.parse_values_any(TokenKind::OpenCurlyBrace, TokenKind::CloseCurlyBrace)?,
                ),
                TokenKind::OpenParenthesis => Fields::Unnamed(
                    self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
                ),
                _ => {
                    return Err(ParserError::UnexpectedToken(t));
                }
            }
        } else {
            Fields::Unit
        };

        advance_match!(self, TokenKind::CloseParenthesis);
        Ok(Value::Enum(index, fields))
    }

    pub fn parse_option(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Some => Ok(Value::Option(Some(self.parse_values_one()?).into())),
            TokenKind::None => Ok(Value::Option(None.into())),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }

    pub fn parse_box(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Box);
        Ok(Value::Box(self.parse_values_one()?.into()))
    }

    pub fn parse_array(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Array);
        Ok(Value::Array(self.parse_values_any(
            TokenKind::OpenBracket,
            TokenKind::CloseBracket,
        )?))
    }

    pub fn parse_tuple(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Tuple);
        Ok(Value::Tuple(self.parse_values_any(
            TokenKind::OpenParenthesis,
            TokenKind::CloseParenthesis,
        )?))
    }

    pub fn parse_result(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Ok => Ok(Value::Result(Ok(self.parse_values_one()?).into())),
            TokenKind::Err => Ok(Value::Result(Err(self.parse_values_one()?).into())),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }

    pub fn parse_vec(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Vec);
        let generics = self.parse_generics(1)?;
        Ok(Value::Vec(
            generics[0],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_tree_set(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::TreeSet);
        let generics = self.parse_generics(1)?;
        Ok(Value::Vec(
            generics[0],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_tree_map(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::TreeSet);
        let generics = self.parse_generics(2)?;
        Ok(Value::TreeMap(
            generics[0],
            generics[1],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_hash_set(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::HashSet);

        let generics = self.parse_generics(1)?;
        Ok(Value::Vec(
            generics[0],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_hash_map(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::HashSet);

        let generics = self.parse_generics(2)?;
        Ok(Value::HashMap(
            generics[0],
            generics[1],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_scrypto_types(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Address => Ok(Value::Address(self.parse_values_one()?.into())),
            TokenKind::Hash => Ok(Value::Hash(self.parse_values_one()?.into())),
            TokenKind::Bucket => Ok(Value::Bucket(self.parse_values_one()?.into())),
            TokenKind::BucketRef => Ok(Value::BucketRef(self.parse_values_one()?.into())),
            TokenKind::LazyMap => Ok(Value::LazyMap(self.parse_values_one()?.into())),
            TokenKind::Vault => Ok(Value::Vault(self.parse_values_one()?.into())),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }

    /// Parse a comma-separated value list, enclosed by a pair of marks.
    fn parse_values_any(
        &mut self,
        open: TokenKind,
        close: TokenKind,
    ) -> Result<Vec<Value>, ParserError> {
        advance_match!(self, open);
        let mut values = Vec::new();
        while self.peek()?.kind != close {
            values.push(self.parse_value()?);
            if self.peek()?.kind != close {
                advance_match!(self, TokenKind::Comma);
            }
        }
        advance_match!(self, close);
        Ok(values)
    }

    fn parse_values_one(&mut self) -> Result<Value, ParserError> {
        let values =
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?;
        if values.len() != 1 {
            Err(ParserError::InvalidNumberOfValues {
                actual: values.len(),
                expected: 1,
            })
        } else {
            Ok(values[0].clone())
        }
    }

    fn parse_generics(&mut self, _n: usize) -> Result<Vec<Type>, ParserError> {
        advance_match!(self, TokenKind::LessThan);
        let mut types = Vec::new();
        while self.peek()?.kind != TokenKind::GreaterThan {
            types.push(self.parse_type()?);
            if self.peek()?.kind != TokenKind::GreaterThan {
                advance_match!(self, TokenKind::Comma);
            }
        }
        advance_match!(self, TokenKind::GreaterThan);
        Ok(types)
    }

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        let token = self.advance()?;
        match &token.kind {
            TokenKind::Unit => Ok(Type::Unit),
            TokenKind::Bool => Ok(Type::Bool),
            TokenKind::I8 => Ok(Type::I8),
            TokenKind::I16 => Ok(Type::I16),
            TokenKind::I32 => Ok(Type::I32),
            TokenKind::I64 => Ok(Type::I64),
            TokenKind::I128 => Ok(Type::I128),
            TokenKind::U8 => Ok(Type::U8),
            TokenKind::U16 => Ok(Type::U16),
            TokenKind::U32 => Ok(Type::U32),
            TokenKind::U64 => Ok(Type::U64),
            TokenKind::U128 => Ok(Type::U128),
            TokenKind::String => Ok(Type::String),
            TokenKind::Struct => Ok(Type::Struct),
            TokenKind::Enum => Ok(Type::Enum),
            TokenKind::Option => Ok(Type::Option),
            TokenKind::Box => Ok(Type::Box),
            TokenKind::Array => Ok(Type::Array),
            TokenKind::Tuple => Ok(Type::Tuple),
            TokenKind::Result => Ok(Type::Result),
            TokenKind::Vec => Ok(Type::Vec),
            TokenKind::TreeSet => Ok(Type::TreeSet),
            TokenKind::TreeMap => Ok(Type::TreeMap),
            TokenKind::HashSet => Ok(Type::HashSet),
            TokenKind::HashMap => Ok(Type::HashMap),
            TokenKind::Decimal => Ok(Type::Decimal),
            TokenKind::BigDecimal => Ok(Type::BigDecimal),
            TokenKind::Address => Ok(Type::Address),
            TokenKind::Hash => Ok(Type::Hash),
            TokenKind::Bucket => Ok(Type::Bucket),
            TokenKind::BucketRef => Ok(Type::BucketRef),
            TokenKind::LazyMap => Ok(Type::LazyMap),
            TokenKind::Vault => Ok(Type::Vault),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn lex(s: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(s);
        let mut tokens = Vec::new();
        loop {
            if let Some(token) = lexer.next_token().unwrap() {
                tokens.push(token);
            } else {
                break;
            }
        }
        tokens
    }

    #[macro_export]
    macro_rules! parse_ok {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(lex($s));
            assert_eq!(parser.parse_value(), Ok($expected));
            assert!(parser.is_eof());
        }};
    }

    #[macro_export]
    macro_rules! parse_error {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(lex($s));
            match parser.parse_value() {
                Ok(_) => {
                    panic!("Expected {:?} but no error is thrown", $expected);
                }
                Err(e) => {
                    assert_eq!(e, $expected);
                }
            }
        }};
    }

    #[test]
    fn test_literals() {
        parse_ok!(r#"true"#, Value::Bool(true));
        parse_ok!(r#"false"#, Value::Bool(false));
        parse_ok!(r#"1i8"#, Value::I8(1));
        parse_ok!(r#"1i16"#, Value::I16(1));
        parse_ok!(r#"1i32"#, Value::I32(1));
        parse_ok!(r#"1i64"#, Value::I64(1));
        parse_ok!(r#"1i128"#, Value::I128(1));
        parse_ok!(r#"1u8"#, Value::U8(1));
        parse_ok!(r#"1u16"#, Value::U16(1));
        parse_ok!(r#"1u32"#, Value::U32(1));
        parse_ok!(r#"1u64"#, Value::U64(1));
        parse_ok!(r#"1u128"#, Value::U128(1));
        parse_ok!(r#""test""#, Value::String("test".to_string()));
    }

    #[test]
    fn test_struct() {
        parse_ok!(
            r#"Struct{"Hello", 123u8}"#,
            Value::Struct(Fields::Named(vec![
                Value::String("Hello".to_string()),
                Value::U8(123),
            ]))
        );
        parse_ok!(
            r#"Struct("Hello", 123u8)"#,
            Value::Struct(Fields::Unnamed(vec![
                Value::String("Hello".to_string()),
                Value::U8(123),
            ]))
        );
        parse_ok!(r#"Struct"#, Value::Struct(Fields::Unit));
    }

    #[test]
    fn test_enum() {
        parse_ok!(
            r#"Enum(0u8, {"Hello", 123u8})"#,
            Value::Enum(
                0,
                Fields::Named(vec![Value::String("Hello".to_string()), Value::U8(123)]),
            )
        );
        parse_ok!(
            r#"Enum(0u8, ("Hello", 123u8))"#,
            Value::Enum(
                0,
                Fields::Unnamed(vec![Value::String("Hello".to_string()), Value::U8(123)]),
            )
        );
        parse_ok!(r#"Enum(0u8)"#, Value::Enum(0, Fields::Unit,));
    }

    #[test]
    fn test_transaction() {}
}
