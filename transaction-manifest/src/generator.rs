use crate::ast;
use radix_engine::engine::*;
use radix_engine::model::*;
use sbor::any::{encode_any, Fields, Value};
use sbor::type_id::*;
use sbor::Encoder;
use scrypto::buffer::*;
use scrypto::types::*;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorError {
    WrongTypeOfValue {
        expected_type: Vec<ast::Type>,
        actual: ast::Value,
    },
    InvalidAddress(String),
    InvalidDecimal(String),
    InvalidBigDecimal(String),
    InvalidHash(String),
    InvalidLazyMapId(String),
    InvalidVaultId(String),
    OddNumberOfElements(usize),
    NameResolverError(NameResolverError),
    IdAllocatorError(IdAllocatorError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameResolverError {
    UndefinedBucket(String),
    UndefinedBucketRef(String),
    NamedAlreadyDefined(String),
}

pub struct NameResolver {
    named_buckets: HashMap<String, Bid>,
    named_bucket_refs: HashMap<String, Rid>,
}

impl NameResolver {
    pub fn new() -> Self {
        Self {
            named_buckets: HashMap::new(),
            named_bucket_refs: HashMap::new(),
        }
    }

    pub fn insert_bucket(&mut self, name: String, bid: Bid) -> Result<(), NameResolverError> {
        if self.named_buckets.contains_key(&name) || self.named_bucket_refs.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_buckets.insert(name, bid);
            Ok(())
        }
    }

    pub fn insert_bucket_ref(&mut self, name: String, rid: Rid) -> Result<(), NameResolverError> {
        if self.named_buckets.contains_key(&name) || self.named_bucket_refs.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_bucket_refs.insert(name, rid);
            Ok(())
        }
    }

    pub fn resolve_bucket(&mut self, name: &str) -> Result<Bid, NameResolverError> {
        match self.named_buckets.get(name).cloned() {
            Some(bid) => Ok(bid),
            None => Err(NameResolverError::UndefinedBucket(name.into())),
        }
    }

    pub fn resolve_bucket_ref(&mut self, name: &str) -> Result<Rid, NameResolverError> {
        match self.named_bucket_refs.get(name).cloned() {
            Some(rid) => Ok(rid),
            None => Err(NameResolverError::UndefinedBucketRef(name.into())),
        }
    }
}

pub fn generate_transaction(tx: &ast::Transaction) -> Result<Transaction, GeneratorError> {
    let mut id_allocator = IdAllocator::new(TRANSACTION_OBJECT_ID_RANGE);
    let mut name_resolver = NameResolver::new();
    let mut instructions = Vec::new();

    for instruction in &tx.instructions {
        instructions.push(generate_instruction(
            instruction,
            &mut id_allocator,
            &mut name_resolver,
        )?);
    }

    Ok(Transaction { instructions })
}

pub fn generate_instruction(
    instruction: &ast::Instruction,
    id_allocator: &mut IdAllocator,
    resolver: &mut NameResolver,
) -> Result<Instruction, GeneratorError> {
    Ok(match instruction {
        ast::Instruction::CreateTempBucket {
            amount,
            resource_address,
            new_bucket,
        } => {
            let bid = id_allocator
                .new_bid()
                .map_err(GeneratorError::IdAllocatorError)?;
            let name = generate_string(new_bucket)?;
            resolver
                .insert_bucket(name, bid)
                .map_err(GeneratorError::NameResolverError)?;

            Instruction::CreateTempBucket {
                amount: generate_decimal(amount)?,
                resource_address: generate_address(resource_address)?,
            }
        }
        ast::Instruction::CreateTempBucketRef {
            bucket,
            new_bucket_ref,
        } => {
            let rid = id_allocator
                .new_rid()
                .map_err(GeneratorError::IdAllocatorError)?;
            let name = generate_string(new_bucket_ref)?;
            resolver
                .insert_bucket_ref(name, rid)
                .map_err(GeneratorError::NameResolverError)?;

            Instruction::CreateTempBucketRef {
                bid: generate_bucket(bucket, resolver)?,
            }
        }
        ast::Instruction::CloneTempBucketRef {
            bucket_ref,
            new_bucket_ref,
        } => {
            let rid = id_allocator
                .new_rid()
                .map_err(GeneratorError::IdAllocatorError)?;
            let name = generate_string(new_bucket_ref)?;
            resolver
                .insert_bucket_ref(name, rid)
                .map_err(GeneratorError::NameResolverError)?;

            Instruction::CloneTempBucketRef {
                rid: generate_bucket_ref(bucket_ref, resolver)?,
            }
        }
        ast::Instruction::DropTempBucketRef { bucket_ref } => Instruction::DropTempBucketRef {
            rid: generate_bucket_ref(bucket_ref, resolver)?,
        },
        ast::Instruction::CallFunction {
            package_address,
            blueprint_name,
            function,
            args,
        } => Instruction::CallFunction {
            package_address: generate_address(package_address)?,
            blueprint_name: generate_string(blueprint_name)?,
            function: generate_string(function)?,
            args: generate_args(args, resolver)?,
        },
        ast::Instruction::CallMethod {
            component_address,
            method,
            args,
        } => Instruction::CallMethod {
            component_address: generate_address(component_address)?,
            method: generate_string(method)?,
            args: generate_args(args, resolver)?,
        },
        ast::Instruction::CallMethodWithAllResources {
            component_address,
            method,
        } => Instruction::CallMethodWithAllResources {
            component_address: generate_address(component_address)?,
            method: generate_string(method)?,
        },
    })
}

#[macro_export]
macro_rules! invalid_type {
    ( $v:expr, $($exp:expr),+ ) => {
        Err(GeneratorError::WrongTypeOfValue {
            expected_type: vec!($($exp),+),
            actual: $v.clone(),
        })
    };
}

fn generate_args(
    values: &Vec<ast::Value>,
    resolver: &mut NameResolver,
) -> Result<Vec<Vec<u8>>, GeneratorError> {
    let mut result = Vec::new();
    for v in values {
        let value = generate_value(v, None, resolver)?;

        let mut enc = Encoder::with_type(Vec::new());
        encode_any(None, &value, &mut enc);
        result.push(enc.into());
    }
    Ok(result)
}

fn generate_string(value: &ast::Value) -> Result<String, GeneratorError> {
    match value {
        ast::Value::String(s) => Ok(s.into()),
        v @ _ => invalid_type!(v, ast::Type::String),
    }
}

fn generate_decimal(value: &ast::Value) -> Result<Decimal, GeneratorError> {
    match value {
        ast::Value::Decimal(inner) => match &**inner {
            ast::Value::String(s) => {
                Decimal::from_str(s).map_err(|_| GeneratorError::InvalidDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Decimal),
    }
}

fn generate_big_decimal(value: &ast::Value) -> Result<BigDecimal, GeneratorError> {
    match value {
        ast::Value::BigDecimal(inner) => match &**inner {
            ast::Value::String(s) => {
                BigDecimal::from_str(s).map_err(|_| GeneratorError::InvalidBigDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::BigDecimal),
    }
}

fn generate_address(value: &ast::Value) -> Result<Address, GeneratorError> {
    match value {
        ast::Value::Address(inner) => match &**inner {
            ast::Value::String(s) => {
                Address::from_str(s).map_err(|_| GeneratorError::InvalidAddress(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Address),
    }
}

fn generate_hash(value: &ast::Value) -> Result<H256, GeneratorError> {
    match value {
        ast::Value::Hash(inner) => match &**inner {
            ast::Value::String(s) => {
                H256::from_str(s).map_err(|_| GeneratorError::InvalidHash(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Hash),
    }
}

fn generate_bucket(value: &ast::Value, resolver: &mut NameResolver) -> Result<Bid, GeneratorError> {
    match value {
        ast::Value::Bucket(inner) => match &**inner {
            ast::Value::U32(n) => Ok(Bid(*n)),
            ast::Value::String(s) => resolver
                .resolve_bucket(&s)
                .map_err(GeneratorError::NameResolverError),
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Bucket),
    }
}

fn generate_bucket_ref(
    value: &ast::Value,
    resolver: &mut NameResolver,
) -> Result<Rid, GeneratorError> {
    match value {
        ast::Value::BucketRef(inner) => match &**inner {
            ast::Value::U32(n) => Ok(Rid(*n)),
            ast::Value::String(s) => resolver
                .resolve_bucket_ref(&s)
                .map_err(GeneratorError::NameResolverError),
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::BucketRef),
    }
}

fn generate_lazy_map(value: &ast::Value) -> Result<Mid, GeneratorError> {
    match value {
        ast::Value::LazyMap(inner) => match &**inner {
            ast::Value::String(s) => {
                Mid::from_str(s).map_err(|_| GeneratorError::InvalidLazyMapId(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::LazyMap),
    }
}

fn generate_vault(value: &ast::Value) -> Result<Vid, GeneratorError> {
    match value {
        ast::Value::Vault(inner) => match &**inner {
            ast::Value::String(s) => {
                Vid::from_str(s).map_err(|_| GeneratorError::InvalidVaultId(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Vault),
    }
}

fn generate_value(
    value: &ast::Value,
    expected: Option<ast::Type>,
    resolver: &mut NameResolver,
) -> Result<Value, GeneratorError> {
    if let Some(ty) = expected {
        if ty != value.kind() {
            return Err(GeneratorError::WrongTypeOfValue {
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
        ast::Value::Struct(fields) => Ok(Value::Struct(generate_fields(fields, resolver)?)),
        ast::Value::Enum(index, fields) => {
            Ok(Value::Enum(*index, generate_fields(fields, resolver)?))
        }
        ast::Value::Option(v) => match &**v {
            Some(inner) => Ok(Value::Option(
                Some(generate_value(inner, None, resolver)?).into(),
            )),
            None => Ok(Value::Option(None.into())),
        },
        ast::Value::Box(v) => Ok(Value::Box(generate_value(v, None, resolver)?.into())),
        ast::Value::Array(element_type, elements) => Ok(Value::Array(
            generate_type(element_type),
            generate_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::Tuple(elements) => {
            Ok(Value::Tuple(generate_singletons(elements, None, resolver)?))
        }
        ast::Value::Result(v) => match &**v {
            Ok(inner) => Ok(Value::Result(
                Ok(generate_value(inner, None, resolver)?).into(),
            )),
            Err(inner) => Ok(Value::Result(
                Err(generate_value(inner, None, resolver)?).into(),
            )),
        },
        ast::Value::Vec(element_type, elements) => Ok(Value::Vec(
            generate_type(element_type),
            generate_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::TreeSet(element_type, elements) => Ok(Value::TreeSet(
            generate_type(element_type),
            generate_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::TreeMap(key_type, value_type, elements) => Ok(Value::TreeMap(
            generate_type(key_type),
            generate_type(value_type),
            generate_pairs(elements, *key_type, *value_type, resolver)?,
        )),
        ast::Value::HashSet(element_type, elements) => Ok(Value::HashSet(
            generate_type(element_type),
            generate_singletons(elements, Some(*element_type), resolver)?,
        )),
        ast::Value::HashMap(key_type, value_type, elements) => Ok(Value::HashMap(
            generate_type(key_type),
            generate_type(value_type),
            generate_pairs(elements, *key_type, *value_type, resolver)?,
        )),
        ast::Value::Decimal(_) => {
            generate_decimal(value).map(|v| Value::Custom(SCRYPTO_TYPE_DECIMAL, v.to_vec()))
        }
        ast::Value::BigDecimal(_) => {
            generate_big_decimal(value).map(|v| Value::Custom(SCRYPTO_TYPE_BIG_DECIMAL, v.to_vec()))
        }
        ast::Value::Address(_) => {
            generate_address(value).map(|v| Value::Custom(SCRYPTO_TYPE_ADDRESS, v.to_vec()))
        }
        ast::Value::Hash(_) => {
            generate_hash(value).map(|v| Value::Custom(SCRYPTO_TYPE_H256, v.to_vec()))
        }
        ast::Value::Bucket(_) => {
            generate_bucket(value, resolver).map(|v| Value::Custom(SCRYPTO_TYPE_BID, v.to_vec()))
        }
        ast::Value::BucketRef(_) => generate_bucket_ref(value, resolver)
            .map(|v| Value::Custom(SCRYPTO_TYPE_RID, v.to_vec())),
        ast::Value::LazyMap(_) => {
            generate_lazy_map(value).map(|v| Value::Custom(SCRYPTO_TYPE_MID, v.to_vec()))
        }
        ast::Value::Vault(_) => {
            generate_vault(value).map(|v| Value::Custom(SCRYPTO_TYPE_VID, v.to_vec()))
        }
    }
}

fn generate_fields(
    value: &ast::Fields,
    resolver: &mut NameResolver,
) -> Result<Fields, GeneratorError> {
    match value {
        ast::Fields::Named(fields) => {
            Ok(Fields::Named(generate_singletons(fields, None, resolver)?))
        }
        ast::Fields::Unnamed(fields) => Ok(Fields::Unnamed(generate_singletons(
            fields, None, resolver,
        )?)),
        ast::Fields::Unit => Ok(Fields::Unit),
    }
}

fn generate_singletons(
    elements: &Vec<ast::Value>,
    ty: Option<ast::Type>,
    resolver: &mut NameResolver,
) -> Result<Vec<Value>, GeneratorError> {
    let mut result = vec![];
    for element in elements {
        result.push(generate_value(element, ty, resolver)?);
    }
    Ok(result)
}

fn generate_pairs(
    elements: &Vec<ast::Value>,
    key_type: ast::Type,
    value_type: ast::Type,
    resolver: &mut NameResolver,
) -> Result<Vec<Value>, GeneratorError> {
    if elements.len() % 2 != 0 {
        return Err(GeneratorError::OddNumberOfElements(elements.len()));
    }
    let mut result = vec![];
    for i in 0..elements.len() / 2 {
        result.push(generate_value(&elements[2 * i], Some(key_type), resolver)?);
        result.push(generate_value(
            &elements[2 * i + 1],
            Some(value_type),
            resolver,
        )?);
    }
    Ok(result)
}

fn generate_type(ty: &ast::Type) -> u8 {
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
    macro_rules! generate_value_ok {
        ( $s:expr, $expected:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap()).parse_value().unwrap();
            let mut resolver = NameResolver::new();
            assert_eq!(generate_value(&value, None, &mut resolver), Ok($expected));
        }};
    }

    #[macro_export]
    macro_rules! generate_instruction_ok {
        ( $s:expr, $expected:expr ) => {{
            let instruction = Parser::new(tokenize($s).unwrap())
                .parse_instruction()
                .unwrap();
            let mut id_allocator = IdAllocator::new(TRANSACTION_OBJECT_ID_RANGE);
            let mut resolver = NameResolver::new();
            assert_eq!(
                generate_instruction(&instruction, &mut id_allocator, &mut resolver),
                Ok($expected)
            );
        }};
    }

    #[macro_export]
    macro_rules! generate_value_error {
        ( $s:expr, $expected:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap()).parse_value().unwrap();
            match generate_value(&value, None, &mut NameResolver::new()) {
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
        generate_value_ok!(r#"()"#, Value::Unit);
        generate_value_ok!(r#"true"#, Value::Bool(true));
        generate_value_ok!(r#"false"#, Value::Bool(false));
        generate_value_ok!(r#"1i8"#, Value::I8(1));
        generate_value_ok!(r#"1i128"#, Value::I128(1));
        generate_value_ok!(r#"1u8"#, Value::U8(1));
        generate_value_ok!(r#"1u128"#, Value::U128(1));
        generate_value_ok!(
            r#"Struct({Bucket(1u32), BucketRef(2u32), "bar"})"#,
            Value::Struct(Fields::Named(vec![
                Value::Custom(SCRYPTO_TYPE_BID, Bid(1).to_vec()),
                Value::Custom(SCRYPTO_TYPE_RID, Rid(2).to_vec()),
                Value::String("bar".into())
            ]))
        );
        generate_value_ok!(
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
            ]))
        );
        generate_value_ok!(r#"Struct()"#, Value::Struct(Fields::Unit));
        generate_value_ok!(r#"Enum(0u8, {})"#, Value::Enum(0, Fields::Named(vec![])));
        generate_value_ok!(r#"Enum(1u8, ())"#, Value::Enum(1, Fields::Unnamed(vec![])));
        generate_value_ok!(r#"Enum(2u8)"#, Value::Enum(2, Fields::Unit));
        generate_value_ok!(
            r#"Box(Some("value"))"#,
            Value::Box(Value::Option(Some(Value::String("value".into())).into()).into())
        );
        generate_value_ok!(
            r#"Array<Option>(Some(1u64), None)"#,
            Value::Array(
                TYPE_OPTION,
                vec![
                    Value::Option(Some(Value::U64(1)).into()),
                    Value::Option(None.into())
                ]
            )
        );
        generate_value_ok!(
            r#"Tuple(Ok(1u64), Err(2u64))"#,
            Value::Tuple(vec![
                Value::Result(Ok(Value::U64(1)).into()),
                Value::Result(Err(Value::U64(2)).into()),
            ])
        );
        generate_value_ok!(
            r#"HashMap<HashSet, Vec>(HashSet<U8>(1u8), Vec<U8>(2u8))"#,
            Value::HashMap(
                TYPE_HASH_SET,
                TYPE_VEC,
                vec![
                    Value::HashSet(TYPE_U8, vec![Value::U8(1)]),
                    Value::Vec(TYPE_U8, vec![Value::U8(2)]),
                ]
            )
        );
        generate_value_ok!(
            r#"TreeMap<TreeSet, Vec>(TreeSet<U8>(1u8), Vec<U8>(2u8))"#,
            Value::TreeMap(
                TYPE_TREE_SET,
                TYPE_VEC,
                vec![
                    Value::TreeSet(TYPE_U8, vec![Value::U8(1)]),
                    Value::Vec(TYPE_U8, vec![Value::U8(2)])
                ]
            )
        );
    }

    #[test]
    fn test_failures() {
        generate_value_error!(
            r#"Address(100u32)"#,
            GeneratorError::WrongTypeOfValue {
                expected_type: vec![ast::Type::String],
                actual: ast::Value::U32(100),
            }
        );
        generate_value_error!(
            r#"Address("invalid_address")"#,
            GeneratorError::InvalidAddress("invalid_address".into())
        );
        generate_value_error!(
            r#"Decimal("invalid_decimal")"#,
            GeneratorError::InvalidDecimal("invalid_decimal".into())
        );
        generate_value_error!(
            r#"HashMap<String, String>("abc")"#,
            GeneratorError::OddNumberOfElements(1)
        );
    }

    #[test]
    fn test_instructions() {
        generate_instruction_ok!(
            r#"CREATE_TEMP_BUCKET  Decimal("1.0")  Address("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d")  "xrd_bucket";"#,
            Instruction::CreateTempBucket {
                amount: Decimal::from(1),
                resource_address: Address::from_str(
                    "03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d"
                )
                .unwrap(),
            }
        );
        generate_instruction_ok!(
            r#"CREATE_TEMP_BUCKET_REF  Bucket(5u32)  "admin_auth";"#,
            Instruction::CreateTempBucketRef { bid: Bid(5u32) }
        );
        generate_instruction_ok!(
            r#"CLONE_TEMP_BUCKET_REF  BucketRef(6u32)  "admin_auth";"#,
            Instruction::CloneTempBucketRef { rid: Rid(6u32) }
        );
        generate_instruction_ok!(
            r#"DROP_TEMP_BUCKET_REF  BucketRef(5u32);"#,
            Instruction::DropTempBucketRef { rid: Rid(5u32) }
        );
        generate_instruction_ok!(
            r#"CALL_FUNCTION  Address("01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c")  "Airdrop"  "new"  500u32  HashMap<String, U8>("key", 1u8);"#,
            Instruction::CallFunction {
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
            }
        );
        generate_instruction_ok!(
            r#"CALL_METHOD  Address("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1")  "refill"  Bucket(1u32)  BucketRef(2u32);"#,
            Instruction::CallMethod {
                component_address: Address::from_str(
                    "0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into()
                )
                .unwrap(),
                method: "refill".into(),
                args: vec![scrypto_encode(&Bid(1)), scrypto_encode(&Rid(2))]
            }
        );
        generate_instruction_ok!(
            r#"CALL_METHOD_WITH_ALL_RESOURCES  Address("02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de") "deposit_batch";"#,
            Instruction::CallMethodWithAllResources {
                component_address: Address::from_str(
                    "02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de".into()
                )
                .unwrap(),
                method: "deposit_batch".into(),
            }
        );
    }

    #[test]
    fn test_transaction() {
        let tx = include_str!("../examples/call.rtm");

        assert_eq!(
            crate::compile(tx).unwrap(),
            Transaction {
                instructions: vec![
                    Instruction::CallMethod {
                        component_address: Address::from_str(
                            "02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de".into()
                        )
                        .unwrap(),
                        method: "withdraw".into(),
                        args: vec![
                            scrypto_encode(&Decimal::from(10u32)),
                            scrypto_encode(
                                &Address::from_str(
                                    "030000000000000000000000000000000000000000000000000004"
                                )
                                .unwrap()
                            ),
                            scrypto_encode(&Rid(1)),
                        ]
                    },
                    Instruction::CreateTempBucket {
                        amount: Decimal::from(5),
                        resource_address: Address::from_str(
                            "030000000000000000000000000000000000000000000000000004"
                        )
                        .unwrap(),
                    },
                    Instruction::CallMethod {
                        component_address: Address::from_str(
                            "0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into()
                        )
                        .unwrap(),
                        method: "buy_gumball".into(),
                        args: vec![scrypto_encode(&Bid(512)),]
                    },
                    Instruction::CreateTempBucket {
                        amount: Decimal::from(5),
                        resource_address: Address::from_str(
                            "030000000000000000000000000000000000000000000000000004"
                        )
                        .unwrap(),
                    },
                    Instruction::CreateTempBucketRef { bid: Bid(513) },
                    Instruction::CloneTempBucketRef { rid: Rid(514) },
                    Instruction::DropTempBucketRef { rid: Rid(515) },
                    Instruction::DropTempBucketRef { rid: Rid(514) },
                    Instruction::CallMethodWithAllResources {
                        component_address: Address::from_str(
                            "02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de".into()
                        )
                        .unwrap(),
                        method: "deposit_batch".into(),
                    },
                ]
            }
        );
    }
}
