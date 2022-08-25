use crate::manifest::ast::{Instruction, Type, Value};
use crate::manifest::lexer::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedEof,
    UnexpectedToken(Token),
    InvalidNumberOfValues { actual: usize, expected: usize },
    InvalidNumberOfTypes { actual: usize, expected: usize },
    InvalidHex(String),
    MissingEnumName,
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

    pub fn parse_manifest(&mut self) -> Result<Vec<Instruction>, ParserError> {
        let mut instructions = Vec::<Instruction>::new();

        while !self.is_eof() {
            instructions.push(self.parse_instruction()?);
        }

        Ok(instructions)
    }

    pub fn parse_instruction(&mut self) -> Result<Instruction, ParserError> {
        let token = self.advance()?;
        let instruction = match token.kind {
            TokenKind::TakeFromWorktop => Instruction::TakeFromWorktop {
                resource_address: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            TokenKind::TakeFromWorktopByAmount => Instruction::TakeFromWorktopByAmount {
                amount: self.parse_value()?,
                resource_address: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            TokenKind::TakeFromWorktopByIds => Instruction::TakeFromWorktopByIds {
                ids: self.parse_value()?,
                resource_address: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            TokenKind::ReturnToWorktop => Instruction::ReturnToWorktop {
                bucket: self.parse_value()?,
            },
            TokenKind::AssertWorktopContains => Instruction::AssertWorktopContains {
                resource_address: self.parse_value()?,
            },
            TokenKind::AssertWorktopContainsByAmount => {
                Instruction::AssertWorktopContainsByAmount {
                    amount: self.parse_value()?,
                    resource_address: self.parse_value()?,
                }
            }
            TokenKind::AssertWorktopContainsByIds => Instruction::AssertWorktopContainsByIds {
                ids: self.parse_value()?,
                resource_address: self.parse_value()?,
            },
            TokenKind::PopFromAuthZone => Instruction::PopFromAuthZone {
                new_proof: self.parse_value()?,
            },
            TokenKind::PushToAuthZone => Instruction::PushToAuthZone {
                proof: self.parse_value()?,
            },
            TokenKind::ClearAuthZone => Instruction::ClearAuthZone,
            TokenKind::CreateProofFromAuthZone => Instruction::CreateProofFromAuthZone {
                resource_address: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::CreateProofFromAuthZoneByAmount => {
                Instruction::CreateProofFromAuthZoneByAmount {
                    amount: self.parse_value()?,
                    resource_address: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            TokenKind::CreateProofFromAuthZoneByIds => Instruction::CreateProofFromAuthZoneByIds {
                ids: self.parse_value()?,
                resource_address: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::CreateProofFromBucket => Instruction::CreateProofFromBucket {
                bucket: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::CloneProof => Instruction::CloneProof {
                proof: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::DropProof => Instruction::DropProof {
                proof: self.parse_value()?,
            },
            TokenKind::DropAllProofs => Instruction::DropAllProofs,
            TokenKind::CallFunction => Instruction::CallFunction {
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
            },
            TokenKind::CallMethod => Instruction::CallMethod {
                component_address: self.parse_value()?,
                method: self.parse_value()?,
                args: {
                    let mut values = vec![];
                    while self.peek()?.kind != TokenKind::Semicolon {
                        values.push(self.parse_value()?);
                    }
                    values
                },
            },
            TokenKind::CallMethodWithAllResources => Instruction::CallMethodWithAllResources {
                component_address: self.parse_value()?,
                method: self.parse_value()?,
            },
            TokenKind::PublishPackage => Instruction::PublishPackage {
                package: self.parse_value()?,
            },
            _ => {
                return Err(ParserError::UnexpectedToken(token));
            }
        };
        advance_match!(self, TokenKind::Semicolon);
        Ok(instruction)
    }

    pub fn parse_value(&mut self) -> Result<Value, ParserError> {
        let token = self.peek()?;
        match token.kind {
            TokenKind::OpenParenthesis => {
                advance_match!(self, TokenKind::OpenParenthesis);
                advance_match!(self, TokenKind::CloseParenthesis);
                Ok(Value::Unit)
            }
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
            TokenKind::Array => self.parse_array(),
            TokenKind::Tuple => self.parse_tuple(),
            TokenKind::Ok | TokenKind::Err => self.parse_result(),
            TokenKind::List => self.parse_list(),
            TokenKind::Set => self.parse_set(),
            TokenKind::Map => self.parse_map(),
            TokenKind::Decimal
            | TokenKind::PreciseDecimal
            | TokenKind::PackageAddress
            | TokenKind::ComponentAddress
            | TokenKind::ResourceAddress
            | TokenKind::Hash
            | TokenKind::Bucket
            | TokenKind::Proof
            | TokenKind::NonFungibleId
            | TokenKind::NonFungibleAddress
            | TokenKind::Expression => self.parse_scrypto_types(),
            TokenKind::Bytes => self.parse_bytes(),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }
    pub fn parse_struct(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Struct);
        let fields =
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?;
        Ok(Value::Struct(fields))
    }

    pub fn parse_enum(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Enum);
        let mut name_and_fields =
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?;
        let name = match name_and_fields.get(0) {
            Some(Value::String(name)) => name.clone(),
            _ => {
                return Err(ParserError::MissingEnumName);
            }
        };
        name_and_fields.remove(0);
        Ok(Value::Enum(name, name_and_fields))
    }

    pub fn parse_option(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Some => Ok(Value::Option(Some(self.parse_values_one()?).into())),
            TokenKind::None => Ok(Value::Option(None.into())),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }

    pub fn parse_array(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Array);
        let generics = self.parse_generics(1)?;
        Ok(Value::Array(
            generics[0],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
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

    pub fn parse_list(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::List);
        let generics = self.parse_generics(1)?;
        Ok(Value::List(
            generics[0],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_bytes(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Bytes);
        advance_match!(self, TokenKind::OpenParenthesis);
        let token = self.advance()?;
        let bytes = match token.kind {
            TokenKind::StringLiteral(s) => {
                hex::decode(&s).map_err(|_| ParserError::InvalidHex(s.to_owned()))
            }
            _ => Err(ParserError::UnexpectedToken(token)),
        };
        advance_match!(self, TokenKind::CloseParenthesis);
        Ok(Value::Bytes(bytes?))
    }

    pub fn parse_set(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Set);
        let generics = self.parse_generics(1)?;
        Ok(Value::Set(
            generics[0],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_map(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Map);
        let generics = self.parse_generics(2)?;
        Ok(Value::Map(
            generics[0],
            generics[1],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_scrypto_types(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Decimal => Ok(Value::Decimal(self.parse_values_one()?.into())),
            TokenKind::PreciseDecimal => Ok(Value::PreciseDecimal(self.parse_values_one()?.into())),
            TokenKind::PackageAddress => Ok(Value::PackageAddress(self.parse_values_one()?.into())),
            TokenKind::ComponentAddress => {
                Ok(Value::ComponentAddress(self.parse_values_one()?.into()))
            }
            TokenKind::ResourceAddress => {
                Ok(Value::ResourceAddress(self.parse_values_one()?.into()))
            }
            TokenKind::Hash => Ok(Value::Hash(self.parse_values_one()?.into())),
            TokenKind::Bucket => Ok(Value::Bucket(self.parse_values_one()?.into())),
            TokenKind::Proof => Ok(Value::Proof(self.parse_values_one()?.into())),
            TokenKind::NonFungibleId => Ok(Value::NonFungibleId(self.parse_values_one()?.into())),
            TokenKind::NonFungibleAddress => {
                Ok(Value::NonFungibleAddress(self.parse_values_one()?.into()))
            }
            TokenKind::Expression => Ok(Value::Expression(self.parse_values_one()?.into())),
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

    fn parse_generics(&mut self, n: usize) -> Result<Vec<Type>, ParserError> {
        advance_match!(self, TokenKind::LessThan);
        let mut types = Vec::new();
        while self.peek()?.kind != TokenKind::GreaterThan {
            types.push(self.parse_type()?);
            if self.peek()?.kind != TokenKind::GreaterThan {
                advance_match!(self, TokenKind::Comma);
            }
        }
        advance_match!(self, TokenKind::GreaterThan);

        if types.len() != n {
            Err(ParserError::InvalidNumberOfTypes {
                expected: n,
                actual: types.len(),
            })
        } else {
            Ok(types)
        }
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
            TokenKind::Array => Ok(Type::Array),
            TokenKind::Tuple => Ok(Type::Tuple),
            TokenKind::Result => Ok(Type::Result),
            TokenKind::List => Ok(Type::List),
            TokenKind::Set => Ok(Type::Set),
            TokenKind::Map => Ok(Type::Map),
            TokenKind::Decimal => Ok(Type::Decimal),
            TokenKind::PackageAddress => Ok(Type::PackageAddress),
            TokenKind::ComponentAddress => Ok(Type::ComponentAddress),
            TokenKind::ResourceAddress => Ok(Type::ResourceAddress),
            TokenKind::Hash => Ok(Type::Hash),
            TokenKind::Bucket => Ok(Type::Bucket),
            TokenKind::Proof => Ok(Type::Proof),
            TokenKind::NonFungibleId => Ok(Type::NonFungibleId),
            TokenKind::Expression => Ok(Type::Expression),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::lexer::{tokenize, Span};

    #[macro_export]
    macro_rules! parse_instruction_ok {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(tokenize($s).unwrap());
            assert_eq!(parser.parse_instruction(), Ok($expected));
            assert!(parser.is_eof());
        }};
    }

    #[macro_export]
    macro_rules! parse_value_ok {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(tokenize($s).unwrap());
            assert_eq!(parser.parse_value(), Ok($expected));
            assert!(parser.is_eof());
        }};
    }

    #[macro_export]
    macro_rules! parse_value_error {
        ( $s:expr, $expected:expr ) => {{
            let mut parser = Parser::new(tokenize($s).unwrap());
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
        parse_value_ok!(r#"()"#, Value::Unit);
        parse_value_ok!(r#"true"#, Value::Bool(true));
        parse_value_ok!(r#"false"#, Value::Bool(false));
        parse_value_ok!(r#"1i8"#, Value::I8(1));
        parse_value_ok!(r#"1i16"#, Value::I16(1));
        parse_value_ok!(r#"1i32"#, Value::I32(1));
        parse_value_ok!(r#"1i64"#, Value::I64(1));
        parse_value_ok!(r#"1i128"#, Value::I128(1));
        parse_value_ok!(r#"1u8"#, Value::U8(1));
        parse_value_ok!(r#"1u16"#, Value::U16(1));
        parse_value_ok!(r#"1u32"#, Value::U32(1));
        parse_value_ok!(r#"1u64"#, Value::U64(1));
        parse_value_ok!(r#"1u128"#, Value::U128(1));
        parse_value_ok!(r#""test""#, Value::String("test".into()));
    }

    #[test]
    fn test_struct() {
        parse_value_ok!(
            r#"Struct("Hello", 123u8)"#,
            Value::Struct(vec![Value::String("Hello".into()), Value::U8(123),])
        );
        parse_value_ok!(r#"Struct()"#, Value::Struct(vec![]));
    }

    #[test]
    fn test_enum() {
        parse_value_ok!(
            r#"Enum("Variant", "Hello", 123u8)"#,
            Value::Enum(
                "Variant".to_string(),
                vec![Value::String("Hello".into()), Value::U8(123)],
            )
        );
        parse_value_ok!(
            r#"Enum("Variant")"#,
            Value::Enum("Variant".to_string(), vec![])
        );
    }

    #[test]
    fn test_option_result_box() {
        parse_value_ok!(
            r#"Some("test")"#,
            Value::Option(Some(Value::String("test".into())).into())
        );
        parse_value_ok!(r#"None"#, Value::Option(None.into()));
        parse_value_ok!(
            r#"Ok("test")"#,
            Value::Result(Ok(Value::String("test".into())).into())
        );
        parse_value_ok!(
            r#"Err("test")"#,
            Value::Result(Err(Value::String("test".into())).into())
        );
    }

    #[test]
    fn test_array_tuple() {
        parse_value_ok!(
            r#"Array<U8>(1u8, 2u8)"#,
            Value::Array(Type::U8, vec![Value::U8(1), Value::U8(2)])
        );
        parse_value_ok!(
            r#"Tuple(1u8, 2u8)"#,
            Value::Tuple(vec![Value::U8(1), Value::U8(2)])
        );
    }

    #[test]
    fn test_collections() {
        parse_value_ok!(
            r#"Vec<String>("foo", "bar")"#,
            Value::List(
                Type::String,
                vec![Value::String("foo".into()), Value::String("bar".into())]
            )
        );
        parse_value_ok!(
            r#"List<String>("foo", "bar")"#,
            Value::List(
                Type::String,
                vec![Value::String("foo".into()), Value::String("bar".into())]
            )
        );
        parse_value_ok!(
            r#"Set<String>("1st", "2nd", "3rd")"#,
            Value::Set(
                Type::String,
                vec![
                    Value::String("1st".into()),
                    Value::String("2nd".into()),
                    Value::String("3rd".into())
                ]
            )
        );
        parse_value_ok!(
            r#"Map<String, U32>("key1", 8u32, "key2", 100u32)"#,
            Value::Map(
                Type::String,
                Type::U32,
                vec![
                    Value::String("key1".into()),
                    Value::U32(8),
                    Value::String("key2".into()),
                    Value::U32(100)
                ]
            )
        );
    }

    #[test]
    fn test_failures() {
        parse_value_error!(r#"Enum(0u8"#, ParserError::UnexpectedEof);
        parse_value_error!(
            r#"Enum(0u8>"#,
            ParserError::UnexpectedToken(Token {
                kind: TokenKind::GreaterThan,
                span: Span {
                    start: (1, 10),
                    end: (1, 10)
                }
            })
        );
        parse_value_error!(
            r#"PackageAddress("abc", "def")"#,
            ParserError::InvalidNumberOfValues {
                actual: 2,
                expected: 1
            }
        );
        parse_value_error!(
            r#"List<String, String>("abc", "def")"#,
            ParserError::InvalidNumberOfTypes {
                actual: 2,
                expected: 1
            }
        );
    }

    #[test]
    fn test_transaction() {
        parse_instruction_ok!(
            r#"TAKE_FROM_WORKTOP_BY_AMOUNT  Decimal("1.0")  ResourceAddress("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d")  Bucket("xrd_bucket");"#,
            Instruction::TakeFromWorktopByAmount {
                amount: Value::Decimal(Value::String("1.0".into()).into()),
                resource_address: Value::ResourceAddress(
                    Value::String("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d".into())
                        .into()
                ),
                new_bucket: Value::Bucket(Value::String("xrd_bucket".into()).into()),
            }
        );
        parse_instruction_ok!(
            r#"TAKE_FROM_WORKTOP  ResourceAddress("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d")  Bucket("xrd_bucket");"#,
            Instruction::TakeFromWorktop {
                resource_address: Value::ResourceAddress(
                    Value::String("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d".into())
                        .into()
                ),
                new_bucket: Value::Bucket(Value::String("xrd_bucket".into()).into()),
            }
        );
        parse_instruction_ok!(
            r#"ASSERT_WORKTOP_CONTAINS_BY_AMOUNT  Decimal("1.0")  ResourceAddress("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d");"#,
            Instruction::AssertWorktopContainsByAmount {
                amount: Value::Decimal(Value::String("1.0".into()).into()),
                resource_address: Value::ResourceAddress(
                    Value::String("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d".into())
                        .into()
                ),
            }
        );
        parse_instruction_ok!(
            r#"CREATE_PROOF_FROM_BUCKET  Bucket("xrd_bucket")  Proof("admin_auth");"#,
            Instruction::CreateProofFromBucket {
                bucket: Value::Bucket(Value::String("xrd_bucket".into()).into()),
                new_proof: Value::Proof(Value::String("admin_auth".into()).into()),
            }
        );
        parse_instruction_ok!(
            r#"CLONE_PROOF  Proof("admin_auth")  Proof("admin_auth2");"#,
            Instruction::CloneProof {
                proof: Value::Proof(Value::String("admin_auth".into()).into()),
                new_proof: Value::Proof(Value::String("admin_auth2".into()).into()),
            }
        );
        parse_instruction_ok!(
            r#"DROP_PROOF Proof("admin_auth");"#,
            Instruction::DropProof {
                proof: Value::Proof(Value::String("admin_auth".into()).into()),
            }
        );
        parse_instruction_ok!(r#"DROP_ALL_PROOFS;"#, Instruction::DropAllProofs);
        parse_instruction_ok!(
            r#"CALL_FUNCTION  PackageAddress("01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c")  "Airdrop"  "new"  500u32  Map<String, U8>("key", 1u8);"#,
            Instruction::CallFunction {
                package_address: Value::PackageAddress(
                    Value::String("01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c".into())
                        .into()
                ),
                blueprint_name: Value::String("Airdrop".into()),
                function: Value::String("new".into()),
                args: vec![
                    Value::U32(500),
                    Value::Map(
                        Type::String,
                        Type::U8,
                        vec![Value::String("key".into()), Value::U8(1)]
                    )
                ]
            }
        );
        parse_instruction_ok!(
            r#"CALL_METHOD  ComponentAddress("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1")  "refill"  Bucket("xrd_bucket")  Proof("admin_auth");"#,
            Instruction::CallMethod {
                component_address: Value::ComponentAddress(
                    Value::String("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into())
                        .into()
                ),
                method: Value::String("refill".into()),
                args: vec![
                    Value::Bucket(Value::String("xrd_bucket".into()).into()),
                    Value::Proof(Value::String("admin_auth".into()).into())
                ]
            }
        );
        parse_instruction_ok!(
            r#"CALL_METHOD  ComponentAddress("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1")  "withdraw_non_fungible"  NonFungibleId("00")  Proof("admin_auth");"#,
            Instruction::CallMethod {
                component_address: Value::ComponentAddress(
                    Value::String("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into())
                        .into()
                ),
                method: Value::String("withdraw_non_fungible".into()),
                args: vec![
                    Value::NonFungibleId(Value::String("00".into()).into()),
                    Value::Proof(Value::String("admin_auth".into()).into())
                ]
            }
        );
        parse_instruction_ok!(
            r#"CALL_METHOD_WITH_ALL_RESOURCES  ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "deposit_batch";"#,
            Instruction::CallMethodWithAllResources {
                component_address: Value::ComponentAddress(
                    Value::String(
                        "account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064".into()
                    )
                    .into()
                ),
                method: Value::String("deposit_batch".into()),
            }
        );
    }
}
