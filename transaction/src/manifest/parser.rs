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
            TokenKind::ClearSignatureProofs => Instruction::ClearSignatureProofs,
            TokenKind::CallFunction => Instruction::CallFunction {
                package_address: self.parse_value()?,
                blueprint_name: self.parse_value()?,
                function_name: self.parse_value()?,
                args: {
                    let mut values = Vec::new();
                    while self.peek()?.kind != TokenKind::Semicolon {
                        values.push(self.parse_value()?);
                    }
                    values
                },
            },
            TokenKind::CallMethod => Instruction::CallMethod {
                component_address: self.parse_value()?,
                method_name: self.parse_value()?,
                args: {
                    let mut values = Vec::new();
                    while self.peek()?.kind != TokenKind::Semicolon {
                        values.push(self.parse_value()?);
                    }
                    values
                },
            },

            TokenKind::PublishPackage => Instruction::PublishPackage {
                code: self.parse_value()?,
                schema: self.parse_value()?,
                royalty_config: self.parse_value()?,
                metadata: self.parse_value()?,
            },
            TokenKind::PublishPackageAdvanced => Instruction::PublishPackageAdvanced {
                code: self.parse_value()?,
                schema: self.parse_value()?,
                royalty_config: self.parse_value()?,
                metadata: self.parse_value()?,
                access_rules: self.parse_value()?,
            },
            TokenKind::BurnResource => Instruction::BurnResource {
                bucket: self.parse_value()?,
            },
            TokenKind::RecallResource => Instruction::RecallResource {
                vault_id: self.parse_value()?,
                amount: self.parse_value()?,
            },
            TokenKind::SetMetadata => Instruction::SetMetadata {
                entity_address: self.parse_value()?,
                key: self.parse_value()?,
                value: self.parse_value()?,
            },
            TokenKind::RemoveMetadata => Instruction::RemoveMetadata {
                entity_address: self.parse_value()?,
                key: self.parse_value()?,
            },
            TokenKind::SetPackageRoyaltyConfig => Instruction::SetPackageRoyaltyConfig {
                package_address: self.parse_value()?,
                royalty_config: self.parse_value()?,
            },
            TokenKind::SetComponentRoyaltyConfig => Instruction::SetComponentRoyaltyConfig {
                component_address: self.parse_value()?,
                royalty_config: self.parse_value()?,
            },
            TokenKind::ClaimPackageRoyalty => Instruction::ClaimPackageRoyalty {
                package_address: self.parse_value()?,
            },
            TokenKind::ClaimComponentRoyalty => Instruction::ClaimComponentRoyalty {
                component_address: self.parse_value()?,
            },
            TokenKind::SetMethodAccessRule => Instruction::SetMethodAccessRule {
                entity_address: self.parse_value()?,
                key: self.parse_value()?,
                rule: self.parse_value()?,
            },
            TokenKind::MintFungible => Instruction::MintFungible {
                resource_address: self.parse_value()?,
                amount: self.parse_value()?,
            },
            TokenKind::MintNonFungible => Instruction::MintNonFungible {
                resource_address: self.parse_value()?,
                args: self.parse_value()?,
            },
            TokenKind::MintUuidNonFungible => Instruction::MintUuidNonFungible {
                resource_address: self.parse_value()?,
                args: self.parse_value()?,
            },
            TokenKind::CreateFungibleResource => Instruction::CreateFungibleResource {
                divisibility: self.parse_value()?,
                metadata: self.parse_value()?,
                access_rules: self.parse_value()?,
            },
            TokenKind::CreateFungibleResourceWithInitialSupply => {
                Instruction::CreateFungibleResourceWithInitialSupply {
                    divisibility: self.parse_value()?,
                    metadata: self.parse_value()?,
                    access_rules: self.parse_value()?,
                    initial_supply: self.parse_value()?,
                }
            }
            TokenKind::CreateNonFungibleResource => Instruction::CreateNonFungibleResource {
                id_type: self.parse_value()?,
                schema: self.parse_value()?,
                metadata: self.parse_value()?,
                access_rules: self.parse_value()?,
            },
            TokenKind::CreateNonFungibleResourceWithInitialSupply => {
                Instruction::CreateNonFungibleResourceWithInitialSupply {
                    id_type: self.parse_value()?,
                    schema: self.parse_value()?,
                    metadata: self.parse_value()?,
                    access_rules: self.parse_value()?,
                    initial_supply: self.parse_value()?,
                }
            }
            TokenKind::CreateValidator => Instruction::CreateValidator {
                key: self.parse_value()?,
            },
            TokenKind::CreateAccessController => Instruction::CreateAccessController {
                controlled_asset: self.parse_value()?,
                rule_set: self.parse_value()?,
                timed_recovery_delay_in_minutes: self.parse_value()?,
            },
            TokenKind::CreateIdentity => Instruction::CreateIdentity {},
            TokenKind::CreateIdentityAdvanced => Instruction::CreateIdentityAdvanced {
                config: self.parse_value()?,
            },
            TokenKind::CreateAccount => Instruction::CreateAccount {},
            TokenKind::CreateAccountAdvanced => Instruction::CreateAccountAdvanced {
                config: self.parse_value()?,
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
