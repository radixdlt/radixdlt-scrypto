use crate::manifest::ast::{Instruction, Type, Value};
use crate::manifest::enums::KNOWN_ENUM_DISCRIMINATORS;
use crate::manifest::lexer::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedEof,
    UnexpectedToken(Token),
    InvalidNumberOfValues { actual: usize, expected: usize },
    InvalidNumberOfTypes { actual: usize, expected: usize },
    InvalidHex(String),
    MissingEnumDiscriminator,
    InvalidEnumDiscriminator,
    UnknownEnumDiscriminator(String),
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

    fn parse_values_till_semicolon(&mut self) -> Result<Vec<Value>, ParserError> {
        let mut values = Vec::new();
        while self.peek()?.kind != TokenKind::Semicolon {
            values.push(self.parse_value()?);
        }
        Ok(values)
    }

    pub fn parse_instruction(&mut self) -> Result<Instruction, ParserError> {
        let token = self.advance()?;
        let instruction = match token.kind {
            TokenKind::TakeFromWorktop => Instruction::TakeFromWorktop {
                resource_address: self.parse_value()?,
                amount: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            TokenKind::TakeNonFungiblesFromWorktop => Instruction::TakeNonFungiblesFromWorktop {
                resource_address: self.parse_value()?,
                ids: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            TokenKind::TakeAllFromWorktop => Instruction::TakeAllFromWorktop {
                resource_address: self.parse_value()?,
                new_bucket: self.parse_value()?,
            },
            TokenKind::ReturnToWorktop => Instruction::ReturnToWorktop {
                bucket: self.parse_value()?,
            },
            TokenKind::AssertWorktopContains => Instruction::AssertWorktopContains {
                resource_address: self.parse_value()?,
                amount: self.parse_value()?,
            },
            TokenKind::AssertWorktopContainsNonFungibles => {
                Instruction::AssertWorktopContainsNonFungibles {
                    resource_address: self.parse_value()?,
                    ids: self.parse_value()?,
                }
            }
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
            TokenKind::CreateProofFromAuthZoneOfAmount => {
                Instruction::CreateProofFromAuthZoneOfAmount {
                    resource_address: self.parse_value()?,
                    amount: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            TokenKind::CreateProofFromAuthZoneOfNonFungibles => {
                Instruction::CreateProofFromAuthZoneOfNonFungibles {
                    resource_address: self.parse_value()?,
                    ids: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            TokenKind::CreateProofFromAuthZoneOfAll => Instruction::CreateProofFromAuthZoneOfAll {
                resource_address: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::ClearSignatureProofs => Instruction::ClearSignatureProofs,

            TokenKind::CreateProofFromBucket => Instruction::CreateProofFromBucket {
                bucket: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::CreateProofFromBucketOfAmount => {
                Instruction::CreateProofFromBucketOfAmount {
                    bucket: self.parse_value()?,
                    amount: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            TokenKind::CreateProofFromBucketOfNonFungibles => {
                Instruction::CreateProofFromBucketOfNonFungibles {
                    bucket: self.parse_value()?,
                    ids: self.parse_value()?,
                    new_proof: self.parse_value()?,
                }
            }
            TokenKind::CreateProofFromBucketOfAll => Instruction::CreateProofFromBucketOfAll {
                bucket: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::BurnResource => Instruction::BurnResource {
                bucket: self.parse_value()?,
            },

            TokenKind::CloneProof => Instruction::CloneProof {
                proof: self.parse_value()?,
                new_proof: self.parse_value()?,
            },
            TokenKind::DropProof => Instruction::DropProof {
                proof: self.parse_value()?,
            },
            TokenKind::CallFunction => Instruction::CallFunction {
                package_address: self.parse_value()?,
                blueprint_name: self.parse_value()?,
                function_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CallMethod => Instruction::CallMethod {
                address: self.parse_value()?,
                method_name: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::RecallResource => Instruction::RecallResource {
                vault_id: self.parse_value()?,
                amount: self.parse_value()?,
            },
            TokenKind::DropAllProofs => Instruction::DropAllProofs,

            /* Call function aliases */
            TokenKind::PublishPackage => Instruction::PublishPackage {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::PublishPackageAdvanced => Instruction::PublishPackageAdvanced {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateFungibleResource => Instruction::CreateFungibleResource {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateFungibleResourceWithInitialSupply => {
                Instruction::CreateFungibleResourceWithInitialSupply {
                    args: self.parse_values_till_semicolon()?,
                }
            }
            TokenKind::CreateNonFungibleResource => Instruction::CreateNonFungibleResource {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateNonFungibleResourceWithInitialSupply => {
                Instruction::CreateNonFungibleResourceWithInitialSupply {
                    args: self.parse_values_till_semicolon()?,
                }
            }
            TokenKind::CreateAccessController => Instruction::CreateAccessController {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateIdentity => Instruction::CreateIdentity {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateIdentityAdvanced => Instruction::CreateIdentityAdvanced {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateAccount => Instruction::CreateAccount {
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateAccountAdvanced => Instruction::CreateAccountAdvanced {
                args: self.parse_values_till_semicolon()?,
            },

            /* Call non-main method aliases */
            TokenKind::SetMetadata => Instruction::SetMetadata {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::RemoveMetadata => Instruction::RemoveMetadata {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::SetComponentRoyaltyConfig => Instruction::SetComponentRoyaltyConfig {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::ClaimComponentRoyalty => Instruction::ClaimComponentRoyalty {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::SetAuthorityAccessRule => Instruction::SetAuthorityAccessRule {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::SetAuthorityMutability => Instruction::SetAuthorityMutability {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },

            /* Call main method aliases */
            TokenKind::MintFungible => Instruction::MintFungible {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::MintNonFungible => Instruction::MintNonFungible {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::MintUuidNonFungible => Instruction::MintUuidNonFungible {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::SetPackageRoyaltyConfig => Instruction::SetPackageRoyaltyConfig {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::ClaimPackageRoyalty => Instruction::ClaimPackageRoyalty {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
            },
            TokenKind::CreateValidator => Instruction::CreateValidator {
                address: self.parse_value()?,
                args: self.parse_values_till_semicolon()?,
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
            // ==============
            // Basic Types
            // ==============
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
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Array => self.parse_array(),
            TokenKind::Tuple => self.parse_tuple(),
            TokenKind::Map => self.parse_map(),

            // ==============
            // Aliases
            // ==============
            TokenKind::Some
            | TokenKind::None
            | TokenKind::Ok
            | TokenKind::Err
            | TokenKind::Bytes
            | TokenKind::NonFungibleGlobalId => self.parse_alias(),

            // ==============
            // Custom Types
            // ==============
            TokenKind::Address
            | TokenKind::Bucket
            | TokenKind::Proof
            | TokenKind::Expression
            | TokenKind::Blob
            | TokenKind::Decimal
            | TokenKind::PreciseDecimal
            | TokenKind::NonFungibleLocalId => self.parse_custom_types(),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }

    pub fn parse_enum(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Enum);
        let mut discriminator_and_fields =
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?;
        let discriminator = match discriminator_and_fields.get(0) {
            Some(Value::U8(discriminator)) => Ok(*discriminator),
            Some(Value::String(discriminator)) => KNOWN_ENUM_DISCRIMINATORS
                .get(discriminator.as_str())
                .cloned()
                .ok_or(ParserError::UnknownEnumDiscriminator(discriminator.clone())),
            Some(_) => Err(ParserError::InvalidEnumDiscriminator),
            None => Err(ParserError::MissingEnumDiscriminator),
        }?;
        discriminator_and_fields.remove(0);
        Ok(Value::Enum(discriminator, discriminator_and_fields))
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

    pub fn parse_map(&mut self) -> Result<Value, ParserError> {
        advance_match!(self, TokenKind::Map);
        let generics = self.parse_generics(2)?;
        Ok(Value::Map(
            generics[0],
            generics[1],
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?,
        ))
    }

    pub fn parse_alias(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Some => Ok(Value::Some(Box::new(self.parse_values_one()?))),
            TokenKind::None => Ok(Value::None),
            TokenKind::Ok => Ok(Value::Ok(Box::new(self.parse_values_one()?))),
            TokenKind::Err => Ok(Value::Err(Box::new(self.parse_values_one()?))),
            TokenKind::Bytes => Ok(Value::Bytes(Box::new(self.parse_values_one()?))),
            TokenKind::NonFungibleGlobalId => Ok(Value::NonFungibleGlobalId(Box::new(
                self.parse_values_one()?,
            ))),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }

    pub fn parse_custom_types(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Address => Ok(Value::Address(self.parse_values_one()?.into())),
            TokenKind::Bucket => Ok(Value::Bucket(self.parse_values_one()?.into())),
            TokenKind::Proof => Ok(Value::Proof(self.parse_values_one()?.into())),
            TokenKind::Expression => Ok(Value::Expression(self.parse_values_one()?.into())),
            TokenKind::Blob => Ok(Value::Blob(self.parse_values_one()?.into())),
            TokenKind::Decimal => Ok(Value::Decimal(self.parse_values_one()?.into())),
            TokenKind::PreciseDecimal => Ok(Value::PreciseDecimal(self.parse_values_one()?.into())),
            TokenKind::NonFungibleLocalId => {
                Ok(Value::NonFungibleLocalId(self.parse_values_one()?.into()))
            }

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
            TokenKind::Enum => Ok(Type::Enum),
            TokenKind::Array => Ok(Type::Array),
            TokenKind::Tuple => Ok(Type::Tuple),

            // Alias
            TokenKind::Bytes => Ok(Type::Bytes),
            TokenKind::NonFungibleGlobalId => Ok(Type::NonFungibleGlobalId),

            // Custom types
            TokenKind::Address => Ok(Type::Address),
            TokenKind::Bucket => Ok(Type::Bucket),
            TokenKind::Proof => Ok(Type::Proof),
            TokenKind::Expression => Ok(Type::Expression),
            TokenKind::Blob => Ok(Type::Blob),
            TokenKind::Decimal => Ok(Type::Decimal),
            TokenKind::PreciseDecimal => Ok(Type::PreciseDecimal),
            TokenKind::NonFungibleLocalId => Ok(Type::NonFungibleLocalId),

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
    fn test_enum() {
        parse_value_ok!(
            r#"Enum(0u8, "Hello", 123u8)"#,
            Value::Enum(0, vec![Value::String("Hello".into()), Value::U8(123)],)
        );
        parse_value_ok!(r#"Enum(0u8)"#, Value::Enum(0, Vec::new()));
    }

    #[test]
    fn test_array() {
        parse_value_ok!(
            r#"Array<U8>(1u8, 2u8)"#,
            Value::Array(Type::U8, vec![Value::U8(1), Value::U8(2)])
        );
    }

    #[test]
    fn test_tuple() {
        parse_value_ok!(
            r#"Tuple("Hello", 123u8)"#,
            Value::Tuple(vec![Value::String("Hello".into()), Value::U8(123),])
        );
        parse_value_ok!(r#"Tuple()"#, Value::Tuple(Vec::new()));
        parse_value_ok!(
            r#"Tuple(1u8, 2u8)"#,
            Value::Tuple(vec![Value::U8(1), Value::U8(2)])
        );
    }

    #[test]
    fn test_map() {
        parse_value_ok!(
            r#"Map<String, U8>("Hello", 123u8)"#,
            Value::Map(
                Type::String,
                Type::U8,
                vec![Value::String("Hello".into()), Value::U8(123)]
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
                span: Span { start: 8, end: 9 }
            })
        );
        parse_value_error!(
            r#"Address("abc", "def")"#,
            ParserError::InvalidNumberOfValues {
                actual: 2,
                expected: 1
            }
        );
    }

    // Instruction parsing tests have been removed as they're largely outdated (inconsistent with the data model),
    // which may lead developers to invalid syntax.
    //
    // It's also not very useful as instruction parsing basically calls `parse_value` recursively
    //
    // That said, all manifest instructions should be tested in `generator.rs` and `e2e.rs`.
}
