use crate::ast::{Fields, Instruction, Transaction, Type, Value};
use crate::lexer::{Token, TokenKind};

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
    ( $self:expr, $pattern:path ) => {{
        let token = $self.advance()?;
        if !matches!(token.kind, $pattern) {
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
            TokenKind::I8Literal(value) => advance_ok!(self, Value::I8(value)),
            TokenKind::I16Literal(value) => advance_ok!(self, Value::I16(value)),
            TokenKind::I32Literal(value) => advance_ok!(self, Value::I32(value)),
            TokenKind::I64Literal(value) => advance_ok!(self, Value::I64(value)),
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
        Ok(Value::Struct(match self.peek()?.kind {
            TokenKind::OpenCurlyBrace => Fields::Named(
                self.parse_values_any(TokenKind::OpenCurlyBrace, TokenKind::CloseCurlyBrace)?,
            ),
            TokenKind::OpenBracket => Fields::Named(
                self.parse_values_any(TokenKind::OpenBracket, TokenKind::CloseBracket)?,
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
                TokenKind::OpenBracket => Fields::Named(
                    self.parse_values_any(TokenKind::OpenBracket, TokenKind::CloseBracket)?,
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

    fn parse_values_any(
        &mut self,
        _open: TokenKind,
        _close: TokenKind,
    ) -> Result<Vec<Value>, ParserError> {
        todo!()
    }

    fn parse_values_one(&mut self) -> Result<Value, ParserError> {
        todo!()
    }

    fn parse_generics(&mut self, _n: usize) -> Result<Vec<Type>, ParserError> {
        todo!()
    }
}
