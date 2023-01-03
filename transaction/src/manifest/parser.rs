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
                abi: self.parse_value()?,
                royalty_config: self.parse_value()?,
                metadata: self.parse_value()?,
                access_rules: self.parse_value()?,
            },
            TokenKind::PublishPackageWithOwner => Instruction::PublishPackageWithOwner {
                code: self.parse_value()?,
                abi: self.parse_value()?,
                owner_badge: self.parse_value()?,
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
                index: self.parse_value()?,
                key: self.parse_value()?,
                rule: self.parse_value()?,
            },
            TokenKind::MintFungible => Instruction::MintFungible {
                resource_address: self.parse_value()?,
                amount: self.parse_value()?,
            },
            TokenKind::MintNonFungible => Instruction::MintNonFungible {
                resource_address: self.parse_value()?,
                entries: self.parse_value()?,
            },
            TokenKind::CreateFungibleResource => Instruction::CreateFungibleResource {
                divisibility: self.parse_value()?,
                metadata: self.parse_value()?,
                access_rules: self.parse_value()?,
                initial_supply: self.parse_value()?,
            },
            TokenKind::CreateFungibleResourceWithOwner => {
                Instruction::CreateFungibleResourceWithOwner {
                    divisibility: self.parse_value()?,
                    metadata: self.parse_value()?,
                    owner_badge: self.parse_value()?,
                    initial_supply: self.parse_value()?,
                }
            }
            TokenKind::CreateNonFungibleResource => Instruction::CreateNonFungibleResource {
                id_type: self.parse_value()?,
                metadata: self.parse_value()?,
                access_rules: self.parse_value()?,
                initial_supply: self.parse_value()?,
            },
            TokenKind::CreateNonFungibleResourceWithOwner => {
                Instruction::CreateNonFungibleResourceWithOwner {
                    id_type: self.parse_value()?,
                    metadata: self.parse_value()?,
                    owner_badge: self.parse_value()?,
                    initial_supply: self.parse_value()?,
                }
            }
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
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Array => self.parse_array(),
            TokenKind::Tuple => self.parse_tuple(),

            // ==============
            // Aliases
            // ==============
            TokenKind::Some |
            TokenKind::None |
            TokenKind::Ok |
            TokenKind::Err |
            TokenKind::Bytes => self.parse_alias(),

            // ==============
            // Custom Types
            // ==============

            /* Global address */
            TokenKind::PackageAddress |
            TokenKind::SystemAddress |
            TokenKind::ComponentAddress |
            TokenKind::ResourceAddress |
            /* RE types */
            TokenKind::Ownership |
            TokenKind::Component |
            TokenKind::KeyValueStore |
            TokenKind::NonFungibleAddress |
            TokenKind::Blob |
            /* TX types */
            TokenKind::Bucket |
            TokenKind::Proof |
            TokenKind::Expression |
            /* Uninterpreted */
            TokenKind::Hash |
            TokenKind::EcdsaSecp256k1PublicKey |
            TokenKind::EcdsaSecp256k1Signature |
            TokenKind::EddsaEd25519PublicKey |
            TokenKind::EddsaEd25519Signature |
            TokenKind::Decimal |
            TokenKind::PreciseDecimal |
            TokenKind::NonFungibleId => self.parse_scrypto_types(),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
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

    pub fn parse_alias(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Some => Ok(Value::Some(Box::new(self.parse_values_one()?))),
            TokenKind::None => Ok(Value::None),
            TokenKind::Ok => Ok(Value::Ok(Box::new(self.parse_values_one()?))),
            TokenKind::Err => Ok(Value::Err(Box::new(self.parse_values_one()?))),
            TokenKind::Bytes => Ok(Value::Bytes(Box::new(self.parse_values_one()?))),
            _ => Err(ParserError::UnexpectedToken(token)),
        }
    }

    pub fn parse_scrypto_types(&mut self) -> Result<Value, ParserError> {
        let token = self.advance()?;
        match token.kind {
            // RE global address types
            TokenKind::PackageAddress => Ok(Value::PackageAddress(self.parse_values_one()?.into())),
            TokenKind::SystemAddress => Ok(Value::SystemAddress(self.parse_values_one()?.into())),
            TokenKind::ComponentAddress => {
                Ok(Value::ComponentAddress(self.parse_values_one()?.into()))
            }
            TokenKind::ResourceAddress => {
                Ok(Value::ResourceAddress(self.parse_values_one()?.into()))
            }

            // RE interpreted types
            TokenKind::Ownership => Ok(Value::Own(self.parse_values_one()?.into())),
            TokenKind::Component => Ok(Value::Component(self.parse_values_one()?.into())),
            TokenKind::KeyValueStore => Ok(Value::KeyValueStore(self.parse_values_one()?.into())),
            TokenKind::NonFungibleAddress => {
                let values = self.parse_values_two()?;
                Ok(Value::NonFungibleAddress(values.0.into(), values.1.into()))
            }
            TokenKind::Blob => Ok(Value::Blob(self.parse_values_one()?.into())),

            // TX interpreted types
            TokenKind::Bucket => Ok(Value::Bucket(self.parse_values_one()?.into())),
            TokenKind::Proof => Ok(Value::Proof(self.parse_values_one()?.into())),
            TokenKind::Expression => Ok(Value::Expression(self.parse_values_one()?.into())),

            // Uninterpreted
            TokenKind::Hash => Ok(Value::Hash(self.parse_values_one()?.into())),
            TokenKind::EcdsaSecp256k1PublicKey => Ok(Value::EcdsaSecp256k1PublicKey(
                self.parse_values_one()?.into(),
            )),
            TokenKind::EcdsaSecp256k1Signature => Ok(Value::EcdsaSecp256k1Signature(
                self.parse_values_one()?.into(),
            )),
            TokenKind::EddsaEd25519PublicKey => Ok(Value::EddsaEd25519PublicKey(
                self.parse_values_one()?.into(),
            )),
            TokenKind::EddsaEd25519Signature => Ok(Value::EddsaEd25519Signature(
                self.parse_values_one()?.into(),
            )),
            TokenKind::Decimal => Ok(Value::Decimal(self.parse_values_one()?.into())),
            TokenKind::PreciseDecimal => Ok(Value::PreciseDecimal(self.parse_values_one()?.into())),
            TokenKind::NonFungibleId => Ok(Value::NonFungibleId(self.parse_values_one()?.into())),

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

    fn parse_values_two(&mut self) -> Result<(Value, Value), ParserError> {
        let values =
            self.parse_values_any(TokenKind::OpenParenthesis, TokenKind::CloseParenthesis)?;
        if values.len() != 2 {
            Err(ParserError::InvalidNumberOfValues {
                actual: values.len(),
                expected: 2,
            })
        } else {
            Ok((values[0].clone(), values[1].clone()))
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
            TokenKind::Enum => Ok(Type::Enum),
            TokenKind::Array => Ok(Type::Array),
            TokenKind::Tuple => Ok(Type::Tuple),

            // RE global address types
            TokenKind::PackageAddress => Ok(Type::PackageAddress),
            TokenKind::ComponentAddress => Ok(Type::ComponentAddress),
            TokenKind::ResourceAddress => Ok(Type::ResourceAddress),
            TokenKind::SystemAddress => Ok(Type::SystemAddress),

            // RE interpreted types
            TokenKind::Ownership => Ok(Type::Own),
            TokenKind::Component => Ok(Type::Component),
            TokenKind::KeyValueStore => Ok(Type::KeyValueStore),
            TokenKind::NonFungibleAddress => Ok(Type::NonFungibleAddress),
            TokenKind::Blob => Ok(Type::Blob),

            // TX interpreted types
            TokenKind::Bucket => Ok(Type::Bucket),
            TokenKind::Proof => Ok(Type::Proof),
            TokenKind::Expression => Ok(Type::Expression),

            // Uninterpreted
            TokenKind::Hash => Ok(Type::Hash),
            TokenKind::EcdsaSecp256k1PublicKey => Ok(Type::EcdsaSecp256k1PublicKey),
            TokenKind::EcdsaSecp256k1Signature => Ok(Type::EcdsaSecp256k1Signature),
            TokenKind::EddsaEd25519PublicKey => Ok(Type::EddsaEd25519PublicKey),
            TokenKind::EddsaEd25519Signature => Ok(Type::EddsaEd25519Signature),
            TokenKind::Decimal => Ok(Type::Decimal),
            TokenKind::PreciseDecimal => Ok(Type::PreciseDecimal),
            TokenKind::NonFungibleId => Ok(Type::NonFungibleId),

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
            Value::Enum("Variant".to_string(), Vec::new())
        );
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
            r#"CALL_FUNCTION  PackageAddress("01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c")  "Airdrop"  "new"  500u32;"#,
            Instruction::CallFunction {
                package_address: Value::PackageAddress(
                    Value::String("01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c".into())
                        .into()
                ),
                blueprint_name: Value::String("Airdrop".into()),
                function_name: Value::String("new".into()),
                args: vec![Value::U32(500),]
            }
        );
        parse_instruction_ok!(
            r#"CALL_METHOD  ComponentAddress("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1")  "refill"  Bucket("xrd_bucket")  Proof("admin_auth");"#,
            Instruction::CallMethod {
                component_address: Value::ComponentAddress(
                    Value::String("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into())
                        .into()
                ),
                method_name: Value::String("refill".into()),
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
                method_name: Value::String("withdraw_non_fungible".into()),
                args: vec![
                    Value::NonFungibleId(Value::String("00".into()).into()),
                    Value::Proof(Value::String("admin_auth".into()).into())
                ]
            }
        );

        parse_instruction_ok!(
            r#"PUBLISH_PACKAGE Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d") Array<Tuple>() Array<Tuple>() Array<Tuple>(Tuple(Enum("SetMetadata"), Tuple(Enum("DenyAll"), Enum("DenyAll"))), Tuple(Enum("GetMetadata"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("SetRoyaltyConfig"), Tuple(Enum("DenyAll"), Enum("DenyAll"))), Tuple(Enum("ClaimRoyalty"), Tuple(Enum("DenyAll"), Enum("DenyAll"))));"#,
            Instruction::PublishPackage {
                code: Value::Blob(Box::new(Value::String(
                    "36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618".into()
                ))),
                abi: Value::Blob(Box::new(Value::String(
                    "15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d".into()
                ))),
                royalty_config: Value::Array(Type::Tuple, Vec::new()),
                metadata: Value::Array(Type::Tuple, Vec::new()),
                access_rules: Value::Array(
                    Type::Tuple,
                    vec![
                        Value::Tuple(vec![
                            Value::Enum("SetMetadata".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("DenyAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                        Value::Tuple(vec![
                            Value::Enum("GetMetadata".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                        Value::Tuple(vec![
                            Value::Enum("SetRoyaltyConfig".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("DenyAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                        Value::Tuple(vec![
                            Value::Enum("ClaimRoyalty".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("DenyAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                    ]
                )
            }
        );
        parse_instruction_ok!(
            r#"PUBLISH_PACKAGE_WITH_OWNER Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d") NonFungibleAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak", 1u32);"#,
            Instruction::PublishPackageWithOwner {
                code: Value::Blob(Box::new(Value::String(
                    "36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618".into()
                ))),
                abi: Value::Blob(Box::new(Value::String(
                    "15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d".into()
                ))),
                owner_badge: Value::NonFungibleAddress(
                    Box::new(Value::String(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak".into()
                    )),
                    Box::new(Value::U32(1))
                )
            }
        );

        parse_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE 18u8 Array<Tuple>( Tuple("name", "Token")) Array<Tuple>(Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) Some(Decimal("500"));"#,
            Instruction::CreateFungibleResource {
                divisibility: Value::U8(18),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                access_rules: Value::Array(
                    Type::Tuple,
                    vec![
                        Value::Tuple(vec![
                            Value::Enum("Withdraw".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                        Value::Tuple(vec![
                            Value::Enum("Deposit".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                    ]
                ),
                initial_supply: Value::Some(Box::new(Value::Decimal(Box::new(Value::String(
                    "500".into()
                )))))
            }
        );
        parse_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE 18u8 Array<Tuple>( Tuple("name", "Token")) Array<Tuple>(Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) None;"#,
            Instruction::CreateFungibleResource {
                divisibility: Value::U8(18),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                access_rules: Value::Array(
                    Type::Tuple,
                    vec![
                        Value::Tuple(vec![
                            Value::Enum("Withdraw".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                        Value::Tuple(vec![
                            Value::Enum("Deposit".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                    ]
                ),
                initial_supply: Value::None
            }
        );
        parse_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE_WITH_OWNER 18u8 Array<Tuple>( Tuple("name", "Token")) NonFungibleAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak", 1u32) Some(Decimal("500"));"#,
            Instruction::CreateFungibleResourceWithOwner {
                divisibility: Value::U8(18),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                owner_badge: Value::NonFungibleAddress(
                    Box::new(Value::String(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak".into()
                    )),
                    Box::new(Value::U32(1))
                ),
                initial_supply: Value::Some(Box::new(Value::Decimal(Box::new(Value::String(
                    "500".into()
                )))))
            }
        );
        parse_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE_WITH_OWNER 18u8 Array<Tuple>( Tuple("name", "Token")) NonFungibleAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak", 1u32) None;"#,
            Instruction::CreateFungibleResourceWithOwner {
                divisibility: Value::U8(18),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                owner_badge: Value::NonFungibleAddress(
                    Box::new(Value::String(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak".into()
                    )),
                    Box::new(Value::U32(1))
                ),
                initial_supply: Value::None
            }
        );

        parse_instruction_ok!(
            r#"
            CREATE_NON_FUNGIBLE_RESOURCE 
                Enum("U32") 
                Array<Tuple>(Tuple("name", "Token")) 
                Array<Tuple>(Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) 
                Some(
                    Array<Tuple>(
                        Tuple(
                            NonFungibleId(1u32), 
                            Tuple(
                                Tuple("Hello World", Decimal("12")),
                                Tuple(12u8, 19u128)
                            )
                        )
                    )
                );
            "#,
            Instruction::CreateNonFungibleResource {
                id_type: Value::Enum("U32".into(), Vec::new()),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                access_rules: Value::Array(
                    Type::Tuple,
                    vec![
                        Value::Tuple(vec![
                            Value::Enum("Withdraw".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                        Value::Tuple(vec![
                            Value::Enum("Deposit".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                    ]
                ),
                initial_supply: Value::Some(Box::new(Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::NonFungibleId(Box::new(Value::U32(1))),
                        Value::Tuple(vec![
                            Value::Tuple(vec![
                                Value::String("Hello World".into()),
                                Value::Decimal(Box::new(Value::String("12".into())))
                            ]),
                            Value::Tuple(vec![Value::U8(12), Value::U128(19),]),
                        ])
                    ])]
                )))
            }
        );
        parse_instruction_ok!(
            r#"CREATE_NON_FUNGIBLE_RESOURCE Enum("U32") Array<Tuple>( Tuple("name", "Token")) Array<Tuple>( Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) None;"#,
            Instruction::CreateNonFungibleResource {
                id_type: Value::Enum("U32".into(), Vec::new()),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                access_rules: Value::Array(
                    Type::Tuple,
                    vec![
                        Value::Tuple(vec![
                            Value::Enum("Withdraw".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                        Value::Tuple(vec![
                            Value::Enum("Deposit".into(), Vec::new()),
                            Value::Tuple(vec![
                                Value::Enum("AllowAll".into(), Vec::new()),
                                Value::Enum("DenyAll".into(), Vec::new()),
                            ])
                        ]),
                    ]
                ),
                initial_supply: Value::None
            }
        );
        parse_instruction_ok!(
            r#"
            CREATE_NON_FUNGIBLE_RESOURCE_WITH_OWNER 
                Enum("U32") 
                Array<Tuple>(Tuple("name", "Token")) 
                NonFungibleAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak", 1u32) 
                Some(
                    Array<Tuple>(
                        Tuple(
                            NonFungibleId(1u32), 
                            Tuple(
                                Tuple("Hello World", Decimal("12")),
                                Tuple(12u8, 19u128)
                            )
                        )
                    )
                );
            "#,
            Instruction::CreateNonFungibleResourceWithOwner {
                id_type: Value::Enum("U32".into(), Vec::new()),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                owner_badge: Value::NonFungibleAddress(
                    Box::new(Value::String(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak".into()
                    )),
                    Box::new(Value::U32(1))
                ),
                initial_supply: Value::Some(Box::new(Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::NonFungibleId(Box::new(Value::U32(1))),
                        Value::Tuple(vec![
                            Value::Tuple(vec![
                                Value::String("Hello World".into()),
                                Value::Decimal(Box::new(Value::String("12".into())))
                            ]),
                            Value::Tuple(vec![Value::U8(12), Value::U128(19),]),
                        ])
                    ])]
                )))
            }
        );
        parse_instruction_ok!(
            r#"CREATE_NON_FUNGIBLE_RESOURCE_WITH_OWNER Enum("U32") Array<Tuple>( Tuple("name", "Token")) NonFungibleAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak", 1u32) None;"#,
            Instruction::CreateNonFungibleResourceWithOwner {
                id_type: Value::Enum("U32".into(), Vec::new()),
                metadata: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::String("name".into()),
                        Value::String("Token".into()),
                    ])]
                ),
                owner_badge: Value::NonFungibleAddress(
                    Box::new(Value::String(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak".into()
                    )),
                    Box::new(Value::U32(1))
                ),
                initial_supply: Value::None
            }
        );

        parse_instruction_ok!(
            r#"MINT_FUNGIBLE ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak") Decimal("100");"#,
            Instruction::MintFungible {
                resource_address: Value::ResourceAddress(Box::new(Value::String(
                    "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak".into()
                ))),
                amount: Value::Decimal(Box::new(Value::String("100".into())))
            }
        );
        parse_instruction_ok!(
            r#"
            MINT_NON_FUNGIBLE 
                ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak") 
                Array<Tuple>(
                    Tuple(
                        NonFungibleId(1u32), 
                        Tuple(
                            Tuple("Hello World", Decimal("12")),
                            Tuple(12u8, 19u128)
                        )
                    )
                );
            "#,
            Instruction::MintNonFungible {
                resource_address: Value::ResourceAddress(Box::new(Value::String(
                    "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak".into()
                ))),
                entries: Value::Array(
                    Type::Tuple,
                    vec![Value::Tuple(vec![
                        Value::NonFungibleId(Box::new(Value::U32(1))),
                        Value::Tuple(vec![
                            Value::Tuple(vec![
                                Value::String("Hello World".into()),
                                Value::Decimal(Box::new(Value::String("12".into())))
                            ]),
                            Value::Tuple(vec![Value::U8(12), Value::U128(19),]),
                        ])
                    ])]
                )
            }
        );
    }
}
