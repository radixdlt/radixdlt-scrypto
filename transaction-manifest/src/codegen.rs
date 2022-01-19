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
pub enum CodegenError {
    InvalidType {
        exp: Vec<ast::Type>,
        actual: ast::Value,
    },
    InvalidAddress(String),
    InvalidDecimal(String),
    OddNumberOfElements(usize),
}

pub fn compile(tx: &ast::Transaction) -> Result<Transaction, CodegenError> {
    let mut allocations = Vec::<Instruction>::new();
    let mut others = Vec::<Instruction>::new();
    let mut temp_buckets = HashMap::<String, Bid>::new();
    let mut temp_bucket_refs = HashMap::<String, Rid>::new();

    for inst in &tx.instructions {
        match inst {
            ast::Instruction::TakeFromContext {
                amount,
                resource_address,
                to,
            } => {
                let bucket = translate_bucket(to, &mut allocations, &mut temp_buckets)?;
                others.push(Instruction::TakeFromContext {
                    amount: translate_decimal(amount)?,
                    resource_address: translate_address(resource_address)?,
                    to: bucket,
                });
            }
            ast::Instruction::BorrowFromContext {
                amount,
                resource_address,
                to,
            } => {
                let bucket_ref = translate_bucket_ref(to, &mut allocations, &mut temp_bucket_refs)?;
                others.push(Instruction::BorrowFromContext {
                    amount: translate_decimal(amount)?,
                    resource_address: translate_address(resource_address)?,
                    to: bucket_ref,
                });
            }
            ast::Instruction::CallFunction {
                package_address,
                blueprint_name,
                function,
                args,
            } => {
                others.push(Instruction::CallFunction {
                    package_address: translate_address(package_address)?,
                    blueprint_name: translate_string(blueprint_name)?,
                    function: translate_string(function)?,
                    args: translate_args(args)?,
                });
            }
            ast::Instruction::CallMethod {
                component_address,
                method,
                args,
            } => {
                others.push(Instruction::CallMethod {
                    component_address: translate_address(component_address)?,
                    method: translate_string(method)?,
                    args: translate_args(args)?,
                });
            }
            ast::Instruction::DropAllBucketRefs => {
                others.push(Instruction::DropAllBucketRefs);
            }
            ast::Instruction::DepositAllBuckets { account } => {
                others.push(Instruction::DepositAllBuckets {
                    account: translate_address(account)?,
                });
            }
        }
    }

    allocations.extend(others);
    Ok(Transaction {
        instructions: allocations,
    })
}

#[macro_export]
macro_rules! invalid_type {
    ( $v:expr, $($exp:expr),+ ) => {
        Err(CodegenError::InvalidType {
            exp: vec!($($exp),+),
            actual: $v.clone(),
        })
    };
}

fn translate_args(values: &Vec<ast::Value>) -> Result<Vec<SmartValue>, CodegenError> {
    let mut result = Vec::<SmartValue>::new();
    for v in values {
        let value = translate_value(v, None)?;

        let mut enc = Encoder::with_type(Vec::new());
        encode_any(None, &value, &mut enc);
        result.push(SmartValue {
            encoded: enc.into(),
        });
    }
    Ok(result)
}

fn translate_string(value: &ast::Value) -> Result<String, CodegenError> {
    match value {
        ast::Value::String(s) => Ok(s.into()),
        v @ _ => invalid_type!(v, ast::Type::String),
    }
}

fn translate_address(value: &ast::Value) -> Result<Address, CodegenError> {
    match value {
        ast::Value::Address(a) => match &**a {
            ast::Value::String(s) => {
                Address::from_str(s).map_err(|_| CodegenError::InvalidAddress(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Address),
    }
}

fn translate_decimal(value: &ast::Value) -> Result<Decimal, CodegenError> {
    match value {
        ast::Value::Decimal(a) => match &**a {
            ast::Value::String(s) => {
                Decimal::from_str(s).map_err(|_| CodegenError::InvalidDecimal(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Address),
    }
}

fn translate_bucket(
    value: &ast::Value,
    allocations: &mut Vec<Instruction>,
    temp_buckets: &mut HashMap<String, Bid>,
) -> Result<Bid, CodegenError> {
    match value {
        ast::Value::Bucket(a) => match &**a {
            ast::Value::U32(n) => Ok(Bid(*n)),
            ast::Value::String(s) => {
                let bid = temp_buckets.entry(s.clone()).or_insert({
                    allocations.push(Instruction::DeclareTempBucket);
                    Bid(allocations.len() as u32 - 1)
                });
                Ok(*bid)
            }
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Bucket),
    }
}

fn translate_bucket_ref(
    value: &ast::Value,
    allocations: &mut Vec<Instruction>,
    temp_bucket_refs: &mut HashMap<String, Rid>,
) -> Result<Rid, CodegenError> {
    match value {
        ast::Value::BucketRef(a) => match &**a {
            ast::Value::U32(n) => Ok(Rid(*n)),
            ast::Value::String(s) => {
                let rid = temp_bucket_refs.entry(s.clone()).or_insert({
                    allocations.push(Instruction::DeclareTempBucket);
                    Rid(allocations.len() as u32 - 1)
                });
                Ok(*rid)
            }
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::BucketRef),
    }
}

fn translate_value(value: &ast::Value, expected: Option<ast::Type>) -> Result<Value, CodegenError> {
    if let Some(ty) = expected {
        if ty != value.kind() {
            return Err(CodegenError::InvalidType {
                exp: vec![ty],
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
        ast::Value::Struct(fields) => Ok(Value::Struct(translate_fields(fields)?)),
        ast::Value::Enum(index, fields) => Ok(Value::Enum(*index, translate_fields(fields)?)),
        ast::Value::Option(v) => match &**v {
            Some(inner) => Ok(Value::Option(Some(translate_value(inner, None)?).into())),
            None => Ok(Value::Option(None.into())),
        },
        ast::Value::Box(v) => Ok(Value::Box(translate_value(v, None)?.into())),
        ast::Value::Array(element_type, elements) => Ok(Value::Array(
            type_id(element_type),
            translate_singletons(elements, Some(*element_type))?,
        )),
        ast::Value::Tuple(elements) => Ok(Value::Tuple(translate_singletons(elements, None)?)),
        ast::Value::Result(v) => match &**v {
            Ok(inner) => Ok(Value::Result(Ok(translate_value(inner, None)?).into())),
            Err(inner) => Ok(Value::Result(Err(translate_value(inner, None)?).into())),
        },
        ast::Value::Vec(element_type, elements) => Ok(Value::Vec(
            type_id(element_type),
            translate_singletons(elements, Some(*element_type))?,
        )),
        ast::Value::TreeSet(element_type, elements) => Ok(Value::TreeSet(
            type_id(element_type),
            translate_singletons(elements, Some(*element_type))?,
        )),
        ast::Value::TreeMap(key_type, value_type, elements) => Ok(Value::TreeMap(
            type_id(key_type),
            type_id(value_type),
            translate_pairs(elements, *key_type, *value_type)?,
        )),
        ast::Value::HashSet(element_type, elements) => Ok(Value::HashSet(
            type_id(element_type),
            translate_singletons(elements, Some(*element_type))?,
        )),
        ast::Value::HashMap(key_type, value_type, elements) => Ok(Value::HashMap(
            type_id(key_type),
            type_id(value_type),
            translate_pairs(elements, *key_type, *value_type)?,
        )),
        ast::Value::Decimal(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_DECIMAL, v.to_vec()))
        }
        ast::Value::BigDecimal(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_BIG_DECIMAL, v.to_vec()))
        }
        ast::Value::Address(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_ADDRESS, v.to_vec()))
        }
        ast::Value::Hash(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_H256, v.to_vec()))
        }
        ast::Value::Bucket(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_BID, v.to_vec()))
        }
        ast::Value::BucketRef(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_RID, v.to_vec()))
        }
        ast::Value::LazyMap(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_MID, v.to_vec()))
        }
        ast::Value::Vault(v) => {
            translate_decimal(v).map(|v| Value::Custom(SCRYPTO_TYPE_VID, v.to_vec()))
        }
    }
}

fn translate_fields(value: &ast::Fields) -> Result<Fields, CodegenError> {
    match value {
        ast::Fields::Named(fields) => Ok(Fields::Named(translate_singletons(fields, None)?)),
        ast::Fields::Unnamed(fields) => Ok(Fields::Unnamed(translate_singletons(fields, None)?)),
        ast::Fields::Unit => Ok(Fields::Unit),
    }
}

fn translate_singletons(
    elements: &Vec<ast::Value>,
    ty: Option<ast::Type>,
) -> Result<Vec<Value>, CodegenError> {
    let mut result = vec![];
    for element in elements {
        result.push(translate_value(element, ty)?);
    }
    Ok(result)
}

fn translate_pairs(
    elements: &Vec<ast::Value>,
    key_type: ast::Type,
    value_type: ast::Type,
) -> Result<Vec<(Value, Value)>, CodegenError> {
    if elements.len() % 2 != 0 {
        return Err(CodegenError::OddNumberOfElements(elements.len()));
    }
    let mut result = vec![];
    for i in 0..elements.len() / 2 {
        result.push((
            translate_value(&elements[2 * i], Some(key_type))?,
            translate_value(&elements[2 * i + 1], Some(value_type))?,
        ));
    }
    Ok(result)
}

fn type_id(ty: &ast::Type) -> u8 {
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
        ast::Type::Decimal => scrypto::buffer::SCRYPTO_TYPE_DECIMAL,
        ast::Type::BigDecimal => scrypto::buffer::SCRYPTO_TYPE_BIG_DECIMAL,
        ast::Type::Address => scrypto::buffer::SCRYPTO_TYPE_ADDRESS,
        ast::Type::Hash => scrypto::buffer::SCRYPTO_TYPE_H256,
        ast::Type::Bucket => scrypto::buffer::SCRYPTO_TYPE_BID,
        ast::Type::BucketRef => scrypto::buffer::SCRYPTO_TYPE_RID,
        ast::Type::LazyMap => scrypto::buffer::SCRYPTO_TYPE_MID,
        ast::Type::Vault => scrypto::buffer::SCRYPTO_TYPE_VID,
    }
}
