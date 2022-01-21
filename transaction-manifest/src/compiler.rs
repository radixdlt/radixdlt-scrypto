use crate::ast;
use radix_engine::transaction::*;
use sbor::any::{encode_any, Fields, Value};
use sbor::type_id::*;
use sbor::Encoder;
use scrypto::buffer::*;
use scrypto::types::*;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    WrongTypeOfValue {
        expected_type: Vec<ast::Type>,
        actual: ast::Value,
    },
    InvalidAddress(String),
    InvalidDecimal(String),
    OddNumberOfElements(usize),
}

pub struct NameResolver {
    instructions: Vec<Instruction>,
    named_buckets: HashMap<String, Bid>,
    named_bucket_refs: HashMap<String, Rid>,
}

impl NameResolver {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            named_buckets: HashMap::new(),
            named_bucket_refs: HashMap::new(),
        }
    }

    pub fn new_bucket(&mut self) -> Bid {
        self.instructions.push(Instruction::DeclareTempBucket);
        Bid(self.instructions.len() as u32 - 1)
    }

    pub fn new_bucket_ref(&mut self) -> Rid {
        self.instructions.push(Instruction::DeclareTempBucketRef);
        Rid(self.instructions.len() as u32 - 1)
    }

    pub fn resolve_bucket(&mut self, name: &str) -> Bid {
        match self.named_buckets.get(name).cloned() {
            Some(bid) => bid,
            None => {
                let bid = self.new_bucket();
                self.named_buckets.insert(name.to_string(), bid);
                bid
            }
        }
    }

    pub fn resolve_bucket_ref(&mut self, name: &str) -> Rid {
        // TODO warning if a single name is used for both bucket and bucket_ref.
        match self.named_bucket_refs.get(name).cloned() {
            Some(rid) => rid,
            None => {
                let rid = self.new_bucket_ref();
                self.named_bucket_refs.insert(name.to_string(), rid);
                rid
            }
        }
    }

    pub fn instructions(&self) -> Vec<Instruction> {
        self.instructions.clone()
    }
}

pub fn compile_transaction(tx: &ast::Transaction) -> Result<Transaction, CompilerError> {
    let mut name_resolver = NameResolver::new();
    let mut other_instructions = Vec::new();

    for instruction in &tx.instructions {
        if let Some(i) = compile_instruction(instruction, &mut name_resolver)? {
            other_instructions.push(i);
        }
    }

    let mut instructions = name_resolver.instructions();
    instructions.extend(other_instructions);
    Ok(Transaction { instructions })
}

pub fn compile_instruction(
    instruction: &ast::Instruction,
    resolver: &mut NameResolver,
) -> Result<Option<Instruction>, CompilerError> {
    Ok(match instruction {
        ast::Instruction::DeclareTempBucket => {
            resolver.new_bucket();
            None
        }
        ast::Instruction::DeclareTempBucketRef => {
            resolver.new_bucket_ref();
            None
        }
        ast::Instruction::TakeFromContext {
            amount,
            resource_address,
            to,
        } => Some(Instruction::TakeFromContext {
            amount: compile_decimal(amount)?,
            resource_address: compile_address(resource_address)?,
            to: compile_bucket(to, resolver)?,
        }),
        ast::Instruction::BorrowFromContext {
            amount,
            resource_address,
            to,
        } => Some(Instruction::BorrowFromContext {
            amount: compile_decimal(amount)?,
            resource_address: compile_address(resource_address)?,
            to: compile_bucket_ref(to, resolver)?,
        }),
        ast::Instruction::CallFunction {
            package_address,
            blueprint_name,
            function,
            args,
        } => Some(Instruction::CallFunction {
            package_address: compile_address(package_address)?,
            blueprint_name: compile_string(blueprint_name)?,
            function: compile_string(function)?,
            args: compile_args(args, resolver)?,
        }),
        ast::Instruction::CallMethod {
            component_address,
            method,
            args,
        } => Some(Instruction::CallMethod {
            component_address: compile_address(component_address)?,
            method: compile_string(method)?,
            args: compile_args(args, resolver)?,
        }),
        ast::Instruction::DropAllBucketRefs => Some(Instruction::DropAllBucketRefs),
        ast::Instruction::DepositAllBuckets { account } => Some(Instruction::DepositAllBuckets {
            account: compile_address(account)?,
        }),
    })
}

#[macro_export]
macro_rules! invalid_type {
    ( $v:expr, $($exp:expr),+ ) => {
        Err(CompilerError::WrongTypeOfValue {
            expected_type: vec!($($exp),+),
            actual: $v.clone(),
        })
    };
}

fn compile_args(
    values: &Vec<ast::Value>,
    resolver: &mut NameResolver,
) -> Result<Vec<Vec<u8>>, CompilerError> {
    let mut result = Vec::new();
    for v in values {
        let value = compile_value(v, None, resolver)?;

        let mut enc = Encoder::with_type(Vec::new());
        encode_any(None, &value, &mut enc);
        result.push(enc.into());
    }
    Ok(result)
}

fn compile_string(value: &ast::Value) -> Result<String, CompilerError> {
    match value {
        ast::Value::String(s) => Ok(s.into()),
        v @ _ => invalid_type!(v, ast::Type::String),
    }
}

fn compile_decimal(value: &ast::Value) -> Result<Decimal, CompilerError> {
    match value {
        ast::Value::Decimal(inner) => match &**inner {
            ast::Value::String(s) => {
                Decimal::from_str(s).map_err(|_| CompilerError::InvalidDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Decimal),
    }
}

fn compile_big_decimal(value: &ast::Value) -> Result<BigDecimal, CompilerError> {
    match value {
        ast::Value::BigDecimal(inner) => match &**inner {
            ast::Value::String(s) => {
                BigDecimal::from_str(s).map_err(|_| CompilerError::InvalidDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::BigDecimal),
    }
}

fn compile_address(value: &ast::Value) -> Result<Address, CompilerError> {
    match value {
        ast::Value::Address(inner) => match &**inner {
            ast::Value::String(s) => {
                Address::from_str(s).map_err(|_| CompilerError::InvalidAddress(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Address),
    }
}

fn compile_hash(value: &ast::Value) -> Result<H256, CompilerError> {
    match value {
        ast::Value::Hash(inner) => match &**inner {
            ast::Value::String(s) => {
                H256::from_str(s).map_err(|_| CompilerError::InvalidDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Decimal),
    }
}

fn compile_bucket(value: &ast::Value, resolver: &mut NameResolver) -> Result<Bid, CompilerError> {
    match value {
        ast::Value::Bucket(inner) => match &**inner {
            ast::Value::U32(n) => Ok(Bid(*n)),
            ast::Value::String(s) => Ok(resolver.resolve_bucket(s)),
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Bucket),
    }
}

fn compile_bucket_ref(
    value: &ast::Value,
    resolver: &mut NameResolver,
) -> Result<Rid, CompilerError> {
    match value {
        ast::Value::BucketRef(inner) => match &**inner {
            ast::Value::U32(n) => Ok(Rid(*n)),
            ast::Value::String(s) => Ok(resolver.resolve_bucket_ref(s)),
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::BucketRef),
    }
}

fn compile_lazy_map(value: &ast::Value) -> Result<Mid, CompilerError> {
    match value {
        ast::Value::LazyMap(inner) => match &**inner {
            ast::Value::String(s) => {
                Mid::from_str(s).map_err(|_| CompilerError::InvalidDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::LazyMap),
    }
}

fn compile_vault(value: &ast::Value) -> Result<Vid, CompilerError> {
    match value {
        ast::Value::Vault(inner) => match &**inner {
            ast::Value::String(s) => {
                Vid::from_str(s).map_err(|_| CompilerError::InvalidDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Vault),
    }
}

fn compile_value(
    value: &ast::Value,
    expected: Option<ast::Type>,
    resolver: &mut NameResolver,
) -> Result<Value, CompilerError> {
    if let Some(ty) = expected {
        if ty != value.kind() {
            return Err(CompilerError::WrongTypeOfValue {
                expected_type: vec![ty],
                actual: value.clone(),
            });
        }
    }

    match value {
        ast::Value::Unit => Ok(Value::Unit),
        ast::Value::Bool(v) => Ok(Value::Bool(*v)),
        ast::Value::I8(v) => Ok(Value::I8(*v)),
        ast::Value::I16(v) => Ok(Value::I16(*v)),
        ast::Value::I32(v) => Ok(Value::I32(*v)),
        ast::Value::I64(v) => Ok(Value::I64(*v)),
        ast::Value::I128(v) => Ok(Value::I128(*v)),
        ast::Value::U8(v) => Ok(Value::U8(*v)),
        ast::Value::U16(v) => Ok(Value::U16(*v)),
        ast::Value::U32(v) => Ok(Value::U32(*v)),
        ast::Value::U64(v) => Ok(Value::U64(*v)),
        ast::Value::U128(v) => Ok(Value::U128(*v)),
        ast::Value::String(v) => Ok(Value::String(v.clone())),
        ast::Value::Struct(fields) => Ok(Value::Struct(compile_fields(fields, resolver)?)),
        ast::Value::Enum(index, fields) => {
            Ok(Value::Enum(*index, compile_fields(fields, resolver)?))
        }
        ast::Value::Option(v) => match &**v {
            Some(inner) => Ok(Value::Option(
                Some(compile_value(inner, None, resolver)?).into(),
            )),
            None => Ok(Value::Option(None.into())),
        },
        ast::Value::Box(v) => Ok(Value::Box(compile_value(v, None, resolver)?.into())),
        ast::Value::Array(element_type, elements) => Ok(Value::Array(
            compile_type(element_type),
            compile_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::Tuple(elements) => {
            Ok(Value::Tuple(compile_singletons(elements, None, resolver)?))
        }
        ast::Value::Result(v) => match &**v {
            Ok(inner) => Ok(Value::Result(
                Ok(compile_value(inner, None, resolver)?).into(),
            )),
            Err(inner) => Ok(Value::Result(
                Err(compile_value(inner, None, resolver)?).into(),
            )),
        },
        ast::Value::Vec(element_type, elements) => Ok(Value::Vec(
            compile_type(element_type),
            compile_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::TreeSet(element_type, elements) => Ok(Value::TreeSet(
            compile_type(element_type),
            compile_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::TreeMap(key_type, value_type, elements) => Ok(Value::TreeMap(
            compile_type(key_type),
            compile_type(value_type),
            compile_pairs(elements, *key_type, *value_type, resolver)?,
        )),
        ast::Value::HashSet(element_type, elements) => Ok(Value::HashSet(
            compile_type(element_type),
            compile_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::HashMap(key_type, value_type, elements) => Ok(Value::HashMap(
            compile_type(key_type),
            compile_type(value_type),
            compile_pairs(elements, *key_type, *value_type, resolver)?,
        )),
        ast::Value::Decimal(_) => {
            compile_decimal(value).map(|v| Value::Custom(SCRYPTO_TYPE_DECIMAL, v.to_vec()))
        }
        ast::Value::BigDecimal(_) => {
            compile_big_decimal(value).map(|v| Value::Custom(SCRYPTO_TYPE_BIG_DECIMAL, v.to_vec()))
        }
        ast::Value::Address(_) => {
            compile_address(value).map(|v| Value::Custom(SCRYPTO_TYPE_ADDRESS, v.to_vec()))
        }
        ast::Value::Hash(_) => {
            compile_hash(value).map(|v| Value::Custom(SCRYPTO_TYPE_H256, v.to_vec()))
        }
        ast::Value::Bucket(_) => {
            compile_bucket(value, resolver).map(|v| Value::Custom(SCRYPTO_TYPE_BID, v.to_vec()))
        }
        ast::Value::BucketRef(_) => {
            compile_bucket_ref(value, resolver).map(|v| Value::Custom(SCRYPTO_TYPE_RID, v.to_vec()))
        }
        ast::Value::LazyMap(_) => {
            compile_lazy_map(value).map(|v| Value::Custom(SCRYPTO_TYPE_MID, v.to_vec()))
        }
        ast::Value::Vault(_) => {
            compile_vault(value).map(|v| Value::Custom(SCRYPTO_TYPE_VID, v.to_vec()))
        }
    }
}

fn compile_fields(
    value: &ast::Fields,
    resolver: &mut NameResolver,
) -> Result<Fields, CompilerError> {
    match value {
        ast::Fields::Named(fields) => {
            Ok(Fields::Named(compile_singletons(fields, None, resolver)?))
        }
        ast::Fields::Unnamed(fields) => {
            Ok(Fields::Unnamed(compile_singletons(fields, None, resolver)?))
        }
        ast::Fields::Unit => Ok(Fields::Unit),
    }
}

fn compile_singletons(
    elements: &Vec<ast::Value>,
    ty: Option<ast::Type>,
    resolver: &mut NameResolver,
) -> Result<Vec<Value>, CompilerError> {
    let mut result = vec![];
    for element in elements {
        result.push(compile_value(element, ty, resolver)?);
    }
    Ok(result)
}

fn compile_pairs(
    elements: &Vec<ast::Value>,
    key_type: ast::Type,
    value_type: ast::Type,
    resolver: &mut NameResolver,
) -> Result<Vec<(Value, Value)>, CompilerError> {
    if elements.len() % 2 != 0 {
        return Err(CompilerError::OddNumberOfElements(elements.len()));
    }
    let mut result = vec![];
    for i in 0..elements.len() / 2 {
        result.push((
            compile_value(&elements[2 * i], Some(key_type), resolver)?,
            compile_value(&elements[2 * i + 1], Some(value_type), resolver)?,
        ));
    }
    Ok(result)
}

fn compile_type(ty: &ast::Type) -> u8 {
    match ty {
        ast::Type::Unit => TYPE_UNIT,
        ast::Type::Bool => TYPE_BOOL,
        ast::Type::I8 => TYPE_I8,
        ast::Type::I16 => TYPE_I16,
        ast::Type::I32 => TYPE_I32,
        ast::Type::I64 => TYPE_I64,
        ast::Type::I128 => TYPE_I128,
        ast::Type::U8 => TYPE_U8,
        ast::Type::U16 => TYPE_U16,
        ast::Type::U32 => TYPE_U32,
        ast::Type::U64 => TYPE_U64,
        ast::Type::U128 => TYPE_U128,
        ast::Type::String => TYPE_STRING,
        ast::Type::Struct => TYPE_STRUCT,
        ast::Type::Enum => TYPE_ENUM,
        ast::Type::Option => TYPE_OPTION,
        ast::Type::Box => TYPE_BOX,
        ast::Type::Array => TYPE_ARRAY,
        ast::Type::Tuple => TYPE_TUPLE,
        ast::Type::Result => TYPE_RESULT,
        ast::Type::Vec => TYPE_VEC,
        ast::Type::TreeSet => TYPE_TREE_SET,
        ast::Type::TreeMap => TYPE_TREE_MAP,
        ast::Type::HashSet => TYPE_HASH_SET,
        ast::Type::HashMap => TYPE_HASH_MAP,
        ast::Type::Decimal => SCRYPTO_TYPE_DECIMAL,
        ast::Type::BigDecimal => SCRYPTO_TYPE_BIG_DECIMAL,
        ast::Type::Address => SCRYPTO_TYPE_ADDRESS,
        ast::Type::Hash => SCRYPTO_TYPE_H256,
        ast::Type::Bucket => SCRYPTO_TYPE_BID,
        ast::Type::BucketRef => SCRYPTO_TYPE_RID,
        ast::Type::LazyMap => SCRYPTO_TYPE_MID,
        ast::Type::Vault => SCRYPTO_TYPE_VID,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::Parser;

    #[macro_export]
    macro_rules! compile_value_ok {
        ( $s:expr, $expected:expr, $allocations:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap()).parse_value().unwrap();
            let mut resolver = NameResolver::new();
            assert_eq!(compile_value(&value, None, &mut resolver), Ok($expected));
            assert_eq!(resolver.instructions(), $allocations);
        }};
    }

    #[macro_export]
    macro_rules! compile_instruction_ok {
        ( $s:expr, $expected:expr, $allocations:expr ) => {{
            let instruction = Parser::new(tokenize($s).unwrap())
                .parse_instruction()
                .unwrap();
            let mut resolver = NameResolver::new();
            assert_eq!(
                compile_instruction(&instruction, &mut resolver),
                Ok($expected)
            );
            assert_eq!(resolver.instructions(), $allocations);
        }};
    }

    #[macro_export]
    macro_rules! compile_value_error {
        ( $s:expr, $expected:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap()).parse_value().unwrap();
            match compile_value(&value, None, &mut NameResolver::new()) {
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
    fn test_value() {
        compile_value_ok!(r#"()"#, Value::Unit, vec![]);
        compile_value_ok!(r#"true"#, Value::Bool(true), vec![]);
        compile_value_ok!(r#"false"#, Value::Bool(false), vec![]);
        compile_value_ok!(r#"1i8"#, Value::I8(1), vec![]);
        compile_value_ok!(r#"1i128"#, Value::I128(1), vec![]);
        compile_value_ok!(r#"1u8"#, Value::U8(1), vec![]);
        compile_value_ok!(r#"1u128"#, Value::U128(1), vec![]);
        compile_value_ok!(
            r#"Struct({Bucket("foo"), BucketRef("foo"), "bar"})"#,
            Value::Struct(Fields::Named(vec![
                Value::Custom(SCRYPTO_TYPE_BID, Bid(0).to_vec()),
                Value::Custom(SCRYPTO_TYPE_RID, Rid(1).to_vec()),
                Value::String("bar".into())
            ])),
            vec![
                Instruction::DeclareTempBucket,
                Instruction::DeclareTempBucketRef
            ]
        );
        compile_value_ok!(
            r#"Struct((Decimal("1.0"), BigDecimal("2.0"), Hash("aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c"), Vault("aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c00000001"), LazyMap("aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c00000002")))"#,
            Value::Struct(Fields::Unnamed(vec![
                Value::Custom(
                    SCRYPTO_TYPE_DECIMAL,
                    Decimal::from_str("1.0").unwrap().to_vec()
                ),
                Value::Custom(
                    SCRYPTO_TYPE_BIG_DECIMAL,
                    BigDecimal::from_str("2.0").unwrap().to_vec()
                ),
                Value::Custom(
                    SCRYPTO_TYPE_H256,
                    H256::from_str(
                        "aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c"
                    )
                    .unwrap()
                    .to_vec()
                ),
                Value::Custom(
                    SCRYPTO_TYPE_VID,
                    Vid::from_str(
                        "aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c00000001"
                    )
                    .unwrap()
                    .to_vec()
                ),
                Value::Custom(
                    SCRYPTO_TYPE_MID,
                    Mid::from_str(
                        "aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c00000002"
                    )
                    .unwrap()
                    .to_vec()
                ),
            ])),
            vec![]
        );
        compile_value_ok!(r#"Struct()"#, Value::Struct(Fields::Unit), vec![]);
        compile_value_ok!(
            r#"Enum(0u8, {})"#,
            Value::Enum(0, Fields::Named(vec![])),
            vec![]
        );
        compile_value_ok!(
            r#"Enum(1u8, ())"#,
            Value::Enum(1, Fields::Unnamed(vec![])),
            vec![]
        );
        compile_value_ok!(r#"Enum(2u8)"#, Value::Enum(2, Fields::Unit), vec![]);
        compile_value_ok!(
            r#"Box(Some("value"))"#,
            Value::Box(Value::Option(Some(Value::String("value".into())).into()).into()),
            vec![]
        );
        compile_value_ok!(
            r#"Array<Option>(Some(1u64), None)"#,
            Value::Array(
                TYPE_OPTION,
                vec![
                    Value::Option(Some(Value::U64(1)).into()),
                    Value::Option(None.into())
                ]
            ),
            vec![]
        );
        compile_value_ok!(
            r#"Tuple(Ok(1u64), Err(2u64))"#,
            Value::Tuple(vec![
                Value::Result(Ok(Value::U64(1)).into()),
                Value::Result(Err(Value::U64(2)).into()),
            ]),
            vec![]
        );
        compile_value_ok!(
            r#"HashMap<HashSet, Vec>(HashSet<U8>(1u8), Vec<U8>(2u8))"#,
            Value::HashMap(
                TYPE_HASH_SET,
                TYPE_VEC,
                vec![(
                    Value::HashSet(TYPE_U8, vec![Value::U8(1)]),
                    Value::Vec(TYPE_U8, vec![Value::U8(2)]),
                )]
            ),
            vec![]
        );
        compile_value_ok!(
            r#"TreeMap<TreeSet, Vec>(TreeSet<U8>(1u8), Vec<U8>(2u8))"#,
            Value::TreeMap(
                TYPE_TREE_SET,
                TYPE_VEC,
                vec![(
                    Value::TreeSet(TYPE_U8, vec![Value::U8(1)]),
                    Value::Vec(TYPE_U8, vec![Value::U8(2)])
                )]
            ),
            vec![]
        );
    }

    #[test]
    fn test_failures() {
        compile_value_error!(
            r#"Address(100u32)"#,
            CompilerError::WrongTypeOfValue {
                expected_type: vec![ast::Type::String],
                actual: ast::Value::U32(100),
            }
        );
        compile_value_error!(
            r#"Address("invalid_address")"#,
            CompilerError::InvalidAddress("invalid_address".into())
        );
        compile_value_error!(
            r#"Decimal("invalid_decimal")"#,
            CompilerError::InvalidDecimal("invalid_decimal".into())
        );
        compile_value_error!(
            r#"HashMap<String, String>("abc")"#,
            CompilerError::OddNumberOfElements(1)
        );
    }

    #[test]
    fn test_transaction() {
        compile_instruction_ok!(
            r#"DECLARE_TEMP_BUCKET;"#,
            None,
            vec![Instruction::DeclareTempBucket]
        );
        compile_instruction_ok!(
            r#"DECLARE_TEMP_BUCKET_REF;"#,
            None,
            vec![Instruction::DeclareTempBucketRef]
        );
        compile_instruction_ok!(
            r#"TAKE_FROM_CONTEXT  Decimal("1.0")  Address("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d")  Bucket("xrd_bucket");"#,
            Some(Instruction::TakeFromContext {
                amount: Decimal::from(1),
                resource_address: Address::from_str(
                    "03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d"
                )
                .unwrap(),
                to: Bid(0),
            }),
            vec![Instruction::DeclareTempBucket]
        );
        compile_instruction_ok!(
            r#"BORROW_FROM_CONTEXT  Decimal("1.0")  Address("03559905076cb3d4b9312640393a7bc6e1d4e491a8b1b62fa73a94")  BucketRef("admin_auth");"#,
            Some(Instruction::BorrowFromContext {
                amount: Decimal::from(1),
                resource_address: Address::from_str(
                    "03559905076cb3d4b9312640393a7bc6e1d4e491a8b1b62fa73a94".into()
                )
                .unwrap(),
                to: Rid(0),
            }),
            vec![Instruction::DeclareTempBucketRef]
        );
        compile_instruction_ok!(
            r#"CALL_FUNCTION  Address("01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c")  "Airdrop"  "new"  500u32  HashMap<String, U8>("key", 1u8);"#,
            Some(Instruction::CallFunction {
                package_address: Address::from_str(
                    "01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c".into()
                )
                .unwrap(),
                blueprint_name: "Airdrop".into(),
                function: "new".into(),
                args: vec![
                    scrypto_encode(&500u32),
                    scrypto_encode(&HashMap::from([("key", 1u8),])),
                ]
            }),
            vec![]
        );
        compile_instruction_ok!(
            r#"CALL_METHOD  Address("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1")  "refill"  Bucket("xrd_bucket")  BucketRef("admin_auth");"#,
            Some(Instruction::CallMethod {
                component_address: Address::from_str(
                    "0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into()
                )
                .unwrap(),
                method: "refill".into(),
                args: vec![scrypto_encode(&Bid(0)), scrypto_encode(&Rid(1))]
            }),
            vec![
                Instruction::DeclareTempBucket,
                Instruction::DeclareTempBucketRef
            ]
        );
        compile_instruction_ok!(
            r#"DROP_ALL_BUCKET_REFS;"#,
            Some(Instruction::DropAllBucketRefs),
            vec![]
        );
        compile_instruction_ok!(
            r#"DEPOSIT_ALL_BUCKETS  Address("02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de");"#,
            Some(Instruction::DepositAllBuckets {
                account: Address::from_str(
                    "02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de".into()
                )
                .unwrap(),
            }),
            vec![]
        );
    }
}
