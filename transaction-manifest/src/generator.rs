use crate::ast;
use radix_engine::engine::*;
use radix_engine::model::*;
use sbor::any::{encode_any, Fields, Value};
use sbor::type_id::*;
use sbor::Encoder;
use scrypto::engine::types::*;
use scrypto::types::*;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorError {
    InvalidType {
        expected_type: ast::Type,
        actual: ast::Type,
    },
    InvalidValue {
        expected_type: Vec<ast::Type>,
        actual: ast::Value,
    },
    InvalidPackageId(String),
    InvalidComponentId(String),
    InvalidResourceDefId(String),
    InvalidDecimal(String),
    InvalidBigDecimal(String),
    InvalidHash(String),
    InvalidLazyMapId(String),
    InvalidVaultId(String),
    InvalidNonFungibleKey(String),
    OddNumberOfElements(usize),
    NameResolverError(NameResolverError),
    IdValidatorError(IdValidatorError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameResolverError {
    UndefinedBucket(String),
    UndefinedProof(String),
    NamedAlreadyDefined(String),
}

pub struct NameResolver {
    named_buckets: HashMap<String, BucketId>,
    named_proofs: HashMap<String, ProofId>,
}

impl NameResolver {
    pub fn new() -> Self {
        Self {
            named_buckets: HashMap::new(),
            named_proofs: HashMap::new(),
        }
    }

    pub fn insert_bucket(
        &mut self,
        name: String,
        bucket_id: BucketId,
    ) -> Result<(), NameResolverError> {
        if self.named_buckets.contains_key(&name) || self.named_proofs.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_buckets.insert(name, bucket_id);
            Ok(())
        }
    }

    pub fn insert_proof(
        &mut self,
        name: String,
        proof_id: ProofId,
    ) -> Result<(), NameResolverError> {
        if self.named_buckets.contains_key(&name) || self.named_proofs.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_proofs.insert(name, proof_id);
            Ok(())
        }
    }

    pub fn resolve_bucket(&mut self, name: &str) -> Result<BucketId, NameResolverError> {
        match self.named_buckets.get(name).cloned() {
            Some(bucket_id) => Ok(bucket_id),
            None => Err(NameResolverError::UndefinedBucket(name.into())),
        }
    }

    pub fn resolve_proof(&mut self, name: &str) -> Result<ProofId, NameResolverError> {
        match self.named_proofs.get(name).cloned() {
            Some(proof_id) => Ok(proof_id),
            None => Err(NameResolverError::UndefinedProof(name.into())),
        }
    }
}

pub fn generate_transaction(tx: &ast::Transaction) -> Result<Transaction, GeneratorError> {
    let mut id_validator = IdValidator::new();
    let mut name_resolver = NameResolver::new();
    let mut instructions = Vec::new();

    for instruction in &tx.instructions {
        instructions.push(generate_instruction(
            instruction,
            &mut id_validator,
            &mut name_resolver,
        )?);
    }

    Ok(Transaction { instructions })
}

pub fn generate_instruction(
    instruction: &ast::Instruction,
    id_validator: &mut IdValidator,
    resolver: &mut NameResolver,
) -> Result<Instruction, GeneratorError> {
    Ok(match instruction {
        ast::Instruction::TakeFromWorktop {
            amount,
            resource_def_id,
            new_bucket,
        } => {
            let bucket_id = id_validator
                .new_bucket()
                .map_err(GeneratorError::IdValidatorError)?;
            declare_bucket(new_bucket, resolver, bucket_id)?;

            Instruction::TakeFromWorktop {
                amount: generate_decimal(amount)?,
                resource_def_id: generate_resource_def_id(resource_def_id)?,
            }
        }
        ast::Instruction::TakeAllFromWorktop {
            resource_def_id,
            new_bucket,
        } => {
            let bucket_id = id_validator
                .new_bucket()
                .map_err(GeneratorError::IdValidatorError)?;
            declare_bucket(new_bucket, resolver, bucket_id)?;

            Instruction::TakeAllFromWorktop {
                resource_def_id: generate_resource_def_id(resource_def_id)?,
            }
        }
        ast::Instruction::TakeNonFungiblesFromWorktop {
            keys,
            resource_def_id,
            new_bucket,
        } => {
            let bucket_id = id_validator
                .new_bucket()
                .map_err(GeneratorError::IdValidatorError)?;
            declare_bucket(new_bucket, resolver, bucket_id)?;

            Instruction::TakeNonFungiblesFromWorktop {
                keys: generate_non_fungible_keys(keys)?,
                resource_def_id: generate_resource_def_id(resource_def_id)?,
            }
        }
        ast::Instruction::ReturnToWorktop { bucket } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(bucket_id)
                .map_err(GeneratorError::IdValidatorError)?;
            Instruction::ReturnToWorktop { bucket_id }
        }
        ast::Instruction::AssertWorktopContains {
            amount,
            resource_def_id,
        } => Instruction::AssertWorktopContains {
            amount: generate_decimal(amount)?,
            resource_def_id: generate_resource_def_id(resource_def_id)?,
        },
        ast::Instruction::CreateBucketProof { bucket, new_proof } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            let proof_id = id_validator
                .new_proof(bucket_id)
                .map_err(GeneratorError::IdValidatorError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            Instruction::CreateBucketProof { bucket_id }
        }
        ast::Instruction::CloneProof { proof, new_proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            let proof_id2 = id_validator
                .clone_proof(proof_id)
                .map_err(GeneratorError::IdValidatorError)?;
            declare_proof(new_proof, resolver, proof_id2)?;

            Instruction::CloneProof { proof_id }
        }
        ast::Instruction::DropProof { proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(proof_id)
                .map_err(GeneratorError::IdValidatorError)?;
            Instruction::DropProof { proof_id }
        }
        ast::Instruction::CallFunction {
            package_id,
            blueprint_name,
            function,
            args,
        } => {
            let args = generate_args(args, resolver)?;
            for arg in &args {
                let validated_arg = ValidatedData::from_slice(arg).unwrap();
                id_validator
                    .move_resources(&validated_arg)
                    .map_err(GeneratorError::IdValidatorError)?;
            }
            Instruction::CallFunction {
                package_id: generate_package_id(package_id)?,
                blueprint_name: generate_string(blueprint_name)?,
                function: generate_string(function)?,
                args,
            }
        }
        ast::Instruction::CallMethod {
            component_id,
            method,
            args,
        } => {
            let args = generate_args(args, resolver)?;
            for arg in &args {
                let validated_arg = ValidatedData::from_slice(arg).unwrap();
                id_validator
                    .move_resources(&validated_arg)
                    .map_err(GeneratorError::IdValidatorError)?;
            }
            Instruction::CallMethod {
                component_id: generate_component_id(component_id)?,
                method: generate_string(method)?,
                args,
            }
        }
        ast::Instruction::CallMethodWithAllResources {
            component_id,
            method,
        } => {
            id_validator
                .move_all_resources()
                .map_err(GeneratorError::IdValidatorError)?;
            Instruction::CallMethodWithAllResources {
                component_id: generate_component_id(component_id)?,
                method: generate_string(method)?,
            }
        }
    })
}

#[macro_export]
macro_rules! invalid_type {
    ( $v:expr, $($exp:expr),+ ) => {
        Err(GeneratorError::InvalidValue {
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

fn generate_package_id(value: &ast::Value) -> Result<PackageId, GeneratorError> {
    match value {
        ast::Value::PackageId(inner) => match &**inner {
            ast::Value::String(s) => {
                PackageId::from_str(s).map_err(|_| GeneratorError::InvalidPackageId(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::PackageId),
    }
}

fn generate_component_id(value: &ast::Value) -> Result<ComponentId, GeneratorError> {
    match value {
        ast::Value::ComponentId(inner) => match &**inner {
            ast::Value::String(s) => {
                ComponentId::from_str(s).map_err(|_| GeneratorError::InvalidComponentId(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::ComponentId),
    }
}

fn generate_resource_def_id(value: &ast::Value) -> Result<ResourceDefId, GeneratorError> {
    match value {
        ast::Value::ResourceDefId(inner) => match &**inner {
            ast::Value::String(s) => ResourceDefId::from_str(s)
                .map_err(|_| GeneratorError::InvalidResourceDefId(s.into())),
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::ResourceDefId),
    }
}

fn generate_hash(value: &ast::Value) -> Result<Hash, GeneratorError> {
    match value {
        ast::Value::Hash(inner) => match &**inner {
            ast::Value::String(s) => {
                Hash::from_str(s).map_err(|_| GeneratorError::InvalidHash(s.into()))
            }
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Hash),
    }
}

fn declare_bucket(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bucket_id: BucketId,
) -> Result<(), GeneratorError> {
    match value {
        ast::Value::Bucket(inner) => match &**inner {
            ast::Value::String(name) => resolver
                .insert_bucket(name.to_string(), bucket_id)
                .map_err(GeneratorError::NameResolverError),
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Bucket),
    }
}

fn generate_bucket(
    value: &ast::Value,
    resolver: &mut NameResolver,
) -> Result<BucketId, GeneratorError> {
    match value {
        ast::Value::Bucket(inner) => match &**inner {
            ast::Value::U32(n) => Ok(*n),
            ast::Value::String(s) => resolver
                .resolve_bucket(&s)
                .map_err(GeneratorError::NameResolverError),
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Bucket),
    }
}

fn declare_proof(
    value: &ast::Value,
    resolver: &mut NameResolver,
    proof_id: ProofId,
) -> Result<(), GeneratorError> {
    match value {
        ast::Value::Proof(inner) => match &**inner {
            ast::Value::String(name) => resolver
                .insert_proof(name.to_string(), proof_id)
                .map_err(GeneratorError::NameResolverError),
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Proof),
    }
}

fn generate_proof(
    value: &ast::Value,
    resolver: &mut NameResolver,
) -> Result<ProofId, GeneratorError> {
    match value {
        ast::Value::Proof(inner) => match &**inner {
            ast::Value::U32(n) => Ok(*n),
            ast::Value::String(s) => resolver
                .resolve_proof(&s)
                .map_err(GeneratorError::NameResolverError),
            v @ _ => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::Proof),
    }
}

fn generate_non_fungible_key(value: &ast::Value) -> Result<NonFungibleKey, GeneratorError> {
    match value {
        ast::Value::NonFungibleKey(inner) => match &**inner {
            ast::Value::String(s) => NonFungibleKey::from_str(s)
                .map_err(|_| GeneratorError::InvalidNonFungibleKey(s.into())),
            v @ _ => invalid_type!(v, ast::Type::String),
        },
        v @ _ => invalid_type!(v, ast::Type::NonFungibleKey),
    }
}

fn generate_non_fungible_keys(
    value: &ast::Value,
) -> Result<BTreeSet<NonFungibleKey>, GeneratorError> {
    match value {
        ast::Value::TreeSet(kind, values) => {
            if kind != &ast::Type::NonFungibleKey {
                return Err(GeneratorError::InvalidType {
                    expected_type: ast::Type::String,
                    actual: kind.clone(),
                });
            }

            values
                .iter()
                .map(|v| generate_non_fungible_key(v))
                .collect()
        }
        v @ _ => invalid_type!(v, ast::Type::TreeSet),
    }
}

fn generate_value(
    value: &ast::Value,
    expected: Option<ast::Type>,
    resolver: &mut NameResolver,
) -> Result<Value, GeneratorError> {
    if let Some(ty) = expected {
        if ty != value.kind() {
            return Err(GeneratorError::InvalidValue {
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
            generate_decimal(value).map(|v| Value::Custom(CustomType::Decimal.id(), v.to_vec()))
        }
        ast::Value::BigDecimal(_) => generate_big_decimal(value)
            .map(|v| Value::Custom(CustomType::BigDecimal.id(), v.to_vec())),
        ast::Value::PackageId(_) => generate_package_id(value)
            .map(|v| Value::Custom(CustomType::PackageId.id(), v.to_vec())),
        ast::Value::ComponentId(_) => generate_component_id(value)
            .map(|v| Value::Custom(CustomType::ComponentId.id(), v.to_vec())),
        ast::Value::ResourceDefId(_) => generate_resource_def_id(value)
            .map(|v| Value::Custom(CustomType::ResourceDefId.id(), v.to_vec())),
        ast::Value::Hash(_) => {
            generate_hash(value).map(|v| Value::Custom(CustomType::Hash.id(), v.to_vec()))
        }
        ast::Value::Bucket(_) => generate_bucket(value, resolver).map(|v| {
            Value::Custom(
                CustomType::Bucket.id(),
                scrypto::resource::Bucket(v).to_vec(),
            )
        }),
        ast::Value::Proof(_) => generate_proof(value, resolver)
            .map(|v| Value::Custom(CustomType::Proof.id(), scrypto::resource::Proof(v).to_vec())),
        ast::Value::NonFungibleKey(_) => generate_non_fungible_key(value)
            .map(|v| Value::Custom(CustomType::NonFungibleKey.id(), v.to_vec())),
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
        ast::Type::Decimal => CustomType::Decimal.id(),
        ast::Type::BigDecimal => CustomType::BigDecimal.id(),
        ast::Type::PackageId => CustomType::PackageId.id(),
        ast::Type::ComponentId => CustomType::ComponentId.id(),
        ast::Type::ResourceDefId => CustomType::ResourceDefId.id(),
        ast::Type::Hash => CustomType::Hash.id(),
        ast::Type::Bucket => CustomType::Bucket.id(),
        ast::Type::Proof => CustomType::Proof.id(),
        ast::Type::NonFungibleKey => CustomType::NonFungibleKey.id(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::Parser;
    use scrypto::buffer::*;

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
            let mut id_validator = IdValidator::new();
            let mut resolver = NameResolver::new();
            assert_eq!(
                generate_instruction(&instruction, &mut id_validator, &mut resolver),
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
            r#"Struct({Bucket(1u32), Proof(2u32), "bar"})"#,
            Value::Struct(Fields::Named(vec![
                Value::Custom(
                    CustomType::Bucket.id(),
                    scrypto::resource::Bucket(1).to_vec()
                ),
                Value::Custom(CustomType::Proof.id(), scrypto::resource::Proof(2).to_vec()),
                Value::String("bar".into())
            ]))
        );
        generate_value_ok!(
            r#"Struct((Decimal("1.0"), BigDecimal("2.0"), Hash("aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c")))"#,
            Value::Struct(Fields::Unnamed(vec![
                Value::Custom(
                    CustomType::Decimal.id(),
                    Decimal::from_str("1.0").unwrap().to_vec()
                ),
                Value::Custom(
                    CustomType::BigDecimal.id(),
                    BigDecimal::from_str("2.0").unwrap().to_vec()
                ),
                Value::Custom(
                    CustomType::Hash.id(),
                    Hash::from_str(
                        "aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c"
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
            r#"ComponentId(100u32)"#,
            GeneratorError::InvalidValue {
                expected_type: vec![ast::Type::String],
                actual: ast::Value::U32(100),
            }
        );
        generate_value_error!(
            r#"PackageId("invalid_package_id")"#,
            GeneratorError::InvalidPackageId("invalid_package_id".into())
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
            r#"TAKE_FROM_WORKTOP  Decimal("1.0")  ResourceDefId("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d")  Bucket("xrd_bucket");"#,
            Instruction::TakeFromWorktop {
                amount: Decimal::from(1),
                resource_def_id: ResourceDefId::from_str(
                    "03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d"
                )
                .unwrap(),
            }
        );
        generate_instruction_ok!(
            r#"TAKE_ALL_FROM_WORKTOP  ResourceDefId("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d")  Bucket("xrd_bucket");"#,
            Instruction::TakeAllFromWorktop {
                resource_def_id: ResourceDefId::from_str(
                    "03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d"
                )
                .unwrap(),
            }
        );
        generate_instruction_ok!(
            r#"ASSERT_WORKTOP_CONTAINS  Decimal("1.0")  ResourceDefId("03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d");"#,
            Instruction::AssertWorktopContains {
                amount: Decimal::from(1),
                resource_def_id: ResourceDefId::from_str(
                    "03cbdf875789d08cc80c97e2915b920824a69ea8d809e50b9fe09d"
                )
                .unwrap(),
            }
        );
        generate_instruction_ok!(
            r#"CALL_FUNCTION  PackageId("01d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c")  "Airdrop"  "new"  500u32  HashMap<String, U8>("key", 1u8);"#,
            Instruction::CallFunction {
                package_id: PackageId::from_str(
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
            r#"CALL_METHOD  ComponentId("0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1")  "refill"  Proof(1u32);"#,
            Instruction::CallMethod {
                component_id: ComponentId::from_str(
                    "0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into()
                )
                .unwrap(),
                method: "refill".into(),
                args: vec![scrypto_encode(&scrypto::resource::Proof(1))]
            }
        );
        generate_instruction_ok!(
            r#"CALL_METHOD_WITH_ALL_RESOURCES  ComponentId("02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de") "deposit_batch";"#,
            Instruction::CallMethodWithAllResources {
                component_id: ComponentId::from_str(
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
                        component_id: ComponentId::from_str(
                            "02d43f479e9b2beb9df98bc3888344fc25eda181e8f710ce1bf1de".into()
                        )
                        .unwrap(),
                        method: "withdraw".into(),
                        args: vec![
                            scrypto_encode(&Decimal::from(5u32)),
                            scrypto_encode(
                                &ResourceDefId::from_str(
                                    "030000000000000000000000000000000000000000000000000004"
                                )
                                .unwrap()
                            ),
                            scrypto_encode(&scrypto::resource::Proof(1)),
                        ]
                    },
                    Instruction::TakeFromWorktop {
                        amount: Decimal::from(2),
                        resource_def_id: ResourceDefId::from_str(
                            "030000000000000000000000000000000000000000000000000004"
                        )
                        .unwrap(),
                    },
                    Instruction::CallMethod {
                        component_id: ComponentId::from_str(
                            "0292566c83de7fd6b04fcc92b5e04b03228ccff040785673278ef1".into()
                        )
                        .unwrap(),
                        method: "buy_gumball".into(),
                        args: vec![scrypto_encode(&scrypto::resource::Bucket(512)),]
                    },
                    Instruction::AssertWorktopContains {
                        amount: Decimal::from(3),
                        resource_def_id: ResourceDefId::from_str(
                            "030000000000000000000000000000000000000000000000000004"
                        )
                        .unwrap(),
                    },
                    Instruction::AssertWorktopContains {
                        amount: Decimal::from(1),
                        resource_def_id: ResourceDefId::from_str(
                            "03aedb7960d1f87dc25138f4cd101da6c98d57323478d53c5fb951"
                        )
                        .unwrap(),
                    },
                    Instruction::TakeAllFromWorktop {
                        resource_def_id: ResourceDefId::from_str(
                            "030000000000000000000000000000000000000000000000000004"
                        )
                        .unwrap(),
                    },
                    Instruction::CreateBucketProof { bucket_id: 513 },
                    Instruction::CloneProof { proof_id: 514 },
                    Instruction::DropProof { proof_id: 515 },
                    Instruction::DropProof { proof_id: 514 },
                    Instruction::ReturnToWorktop { bucket_id: 513 },
                    Instruction::TakeNonFungiblesFromWorktop {
                        keys: BTreeSet::from([
                            NonFungibleKey::from_str("11").unwrap(),
                            NonFungibleKey::from_str("22").unwrap(),
                        ]),
                        resource_def_id: ResourceDefId::from_str(
                            "030000000000000000000000000000000000000000000000000004"
                        )
                        .unwrap(),
                    },
                    Instruction::CallMethodWithAllResources {
                        component_id: ComponentId::from_str(
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
