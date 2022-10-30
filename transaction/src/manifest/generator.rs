use sbor::any::{encode_any, Value};
use sbor::rust::collections::BTreeSet;
use sbor::rust::collections::HashMap;
use sbor::rust::str::FromStr;
use sbor::type_id::*;
use scrypto::abi::*;
use scrypto::address::Bech32Decoder;
use scrypto::buffer::scrypto_decode;
use scrypto::buffer::scrypto_encode;
use scrypto::component::ComponentAddress;
use scrypto::component::PackageAddress;
use scrypto::component::{Component, KeyValueStore};
use scrypto::core::{Blob, Expression, SystemAddress};
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::math::*;
use scrypto::resource::{
    MintParams, NonFungibleAddress, NonFungibleId, ResourceAddress, ResourceManagerBurnInput,
    ResourceManagerCreateInput, ResourceManagerMintInput, Vault,
};
use scrypto::values::*;
use scrypto::{args, args_from_value_vec};

use crate::errors::*;
use crate::manifest::ast;
use crate::model::*;
use crate::validation::*;

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
    InvalidPackageAddress(String),
    InvalidSystemAddress(String),
    InvalidComponentAddress(String),
    InvalidResourceAddress(String),
    InvalidDecimal(String),
    InvalidPreciseDecimal(String),
    InvalidHash(String),
    InvalidNodeId(String),
    InvalidKeyValueStoreId(String),
    InvalidVaultId(String),
    InvalidNonFungibleId(String),
    InvalidNonFungibleAddress(String),
    InvalidExpression(String),
    InvalidComponent(String),
    InvalidKeyValueStore(String),
    InvalidVault(String),
    InvalidEcdsaSecp256k1PublicKey(String),
    InvalidEcdsaSecp256k1Signature(String),
    InvalidEddsaEd25519PublicKey(String),
    InvalidEddsaEd25519Signature(String),
    BlobNotFound(String),
    NameResolverError(NameResolverError),
    IdValidationError(IdValidationError),
    InvalidBlobHash,
    ArgumentsDoNotMatchAbi,
    UnknownNativeFunction(String, String),
    UnknownMethod(String),
    InvalidGlobal(String),
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

pub fn generate_manifest(
    instructions: &[ast::Instruction],
    bech32_decoder: &Bech32Decoder,
    blobs: HashMap<Hash, Vec<u8>>,
) -> Result<TransactionManifest, GeneratorError> {
    let mut id_validator = IdValidator::new();
    let mut name_resolver = NameResolver::new();
    let mut output = Vec::new();

    for instruction in instructions {
        output.push(generate_instruction(
            instruction,
            &mut id_validator,
            &mut name_resolver,
            bech32_decoder,
            &blobs,
        )?);
    }

    Ok(TransactionManifest {
        instructions: output,
        blobs: blobs.into_values().collect(),
    })
}

pub fn generate_instruction(
    instruction: &ast::Instruction,
    id_validator: &mut IdValidator,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &HashMap<Hash, Vec<u8>>,
) -> Result<Instruction, GeneratorError> {
    Ok(match instruction {
        ast::Instruction::TakeFromWorktop {
            resource_address,
            new_bucket,
        } => {
            let bucket_id = id_validator
                .new_bucket()
                .map_err(GeneratorError::IdValidationError)?;
            declare_bucket(new_bucket, resolver, bucket_id)?;

            Instruction::TakeFromWorktop {
                resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            }
        }
        ast::Instruction::TakeFromWorktopByAmount {
            amount,
            resource_address,
            new_bucket,
        } => {
            let bucket_id = id_validator
                .new_bucket()
                .map_err(GeneratorError::IdValidationError)?;
            declare_bucket(new_bucket, resolver, bucket_id)?;

            Instruction::TakeFromWorktopByAmount {
                amount: generate_decimal(amount)?,
                resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            }
        }
        ast::Instruction::TakeFromWorktopByIds {
            ids,
            resource_address,
            new_bucket,
        } => {
            let bucket_id = id_validator
                .new_bucket()
                .map_err(GeneratorError::IdValidationError)?;
            declare_bucket(new_bucket, resolver, bucket_id)?;

            Instruction::TakeFromWorktopByIds {
                ids: generate_non_fungible_ids(ids)?,
                resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            }
        }
        ast::Instruction::ReturnToWorktop { bucket } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(bucket_id)
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::ReturnToWorktop { bucket_id }
        }
        ast::Instruction::AssertWorktopContains { resource_address } => {
            Instruction::AssertWorktopContains {
                resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            }
        }
        ast::Instruction::AssertWorktopContainsByAmount {
            amount,
            resource_address,
        } => Instruction::AssertWorktopContainsByAmount {
            amount: generate_decimal(amount)?,
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
        },
        ast::Instruction::AssertWorktopContainsByIds {
            ids,
            resource_address,
        } => Instruction::AssertWorktopContainsByIds {
            ids: generate_non_fungible_ids(ids)?,
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
        },
        ast::Instruction::PopFromAuthZone { new_proof } => {
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            Instruction::PopFromAuthZone
        }
        ast::Instruction::PushToAuthZone { proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::PushToAuthZone { proof_id }
        }
        ast::Instruction::ClearAuthZone => Instruction::ClearAuthZone,

        ast::Instruction::CreateProofFromAuthZone {
            resource_address,
            new_proof,
        } => {
            let resource_address = generate_resource_address(resource_address, bech32_decoder)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            Instruction::CreateProofFromAuthZone { resource_address }
        }
        ast::Instruction::CreateProofFromAuthZoneByAmount {
            amount,
            resource_address,
            new_proof,
        } => {
            let amount = generate_decimal(amount)?;
            let resource_address = generate_resource_address(resource_address, bech32_decoder)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            Instruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_address,
            }
        }
        ast::Instruction::CreateProofFromAuthZoneByIds {
            ids,
            resource_address,
            new_proof,
        } => {
            let ids = generate_non_fungible_ids(ids)?;
            let resource_address = generate_resource_address(resource_address, bech32_decoder)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            Instruction::CreateProofFromAuthZoneByIds {
                ids,
                resource_address,
            }
        }
        ast::Instruction::CreateProofFromBucket { bucket, new_proof } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            let proof_id = id_validator
                .new_proof(ProofKind::BucketProof(bucket_id))
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            Instruction::CreateProofFromBucket { bucket_id }
        }
        ast::Instruction::CloneProof { proof, new_proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            let proof_id2 = id_validator
                .clone_proof(proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id2)?;

            Instruction::CloneProof { proof_id }
        }
        ast::Instruction::DropProof { proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::DropProof { proof_id }
        }
        ast::Instruction::DropAllProofs => {
            id_validator
                .drop_all_proofs()
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::DropAllProofs
        }
        ast::Instruction::CallFunction {
            package_address,
            blueprint_name,
            function_name,
            args,
        } => {
            let package_address = generate_package_address(package_address, bech32_decoder)?;
            let blueprint_name = generate_string(&blueprint_name)?;
            let function_name = generate_string(&function_name)?;
            let args = generate_args(args, resolver, bech32_decoder, blobs)?;
            let mut fields = Vec::new();
            for arg in &args {
                let validated_arg = ScryptoValue::from_slice(arg).unwrap();
                id_validator
                    .move_resources(&validated_arg)
                    .map_err(GeneratorError::IdValidationError)?;
                fields.push(validated_arg.dom);
            }

            Instruction::CallFunction {
                function_ident: ScryptoFunctionIdent {
                    package: ScryptoPackage::Global(package_address),
                    blueprint_name,
                    function_name,
                },
                args: args_from_value_vec!(fields),
            }
        }
        ast::Instruction::CallMethod {
            receiver,
            method,
            args,
        } => {
            let receiver = generate_scrypto_receiver(receiver, bech32_decoder)?;
            let method_name = generate_string(&method)?;
            let args = generate_args(args, resolver, bech32_decoder, blobs)?;
            let mut fields = Vec::new();
            for arg in &args {
                let validated_arg = ScryptoValue::from_slice(arg).unwrap();
                id_validator
                    .move_resources(&validated_arg)
                    .map_err(GeneratorError::IdValidationError)?;
                fields.push(validated_arg.dom);
            }

            Instruction::CallMethod {
                method_ident: ScryptoMethodIdent {
                    receiver,
                    method_name,
                },
                args: args_from_value_vec!(fields),
            }
        }
        ast::Instruction::CallNativeFunction {
            blueprint_name,
            function_name,
            args,
        } => {
            let blueprint_name = generate_string(&blueprint_name)?;
            let function_name = generate_string(&function_name)?;
            let args = generate_args(args, resolver, bech32_decoder, blobs)?;
            let mut fields = Vec::new();
            for arg in &args {
                let validated_arg = ScryptoValue::from_slice(arg).unwrap();
                id_validator
                    .move_resources(&validated_arg)
                    .map_err(GeneratorError::IdValidationError)?;
                fields.push(validated_arg.dom);
            }

            Instruction::CallNativeFunction {
                function_ident: NativeFunctionIdent {
                    blueprint_name,
                    function_name,
                },
                args: args_from_value_vec!(fields),
            }
        }
        ast::Instruction::CallNativeMethod {
            receiver,
            method,
            args,
        } => {
            let receiver = generate_receiver(receiver, bech32_decoder, resolver)?;
            let method_name = generate_string(&method)?;
            let args = generate_args(args, resolver, bech32_decoder, blobs)?;
            let mut fields = Vec::new();
            for arg in &args {
                let validated_arg = ScryptoValue::from_slice(arg).unwrap();
                id_validator
                    .move_resources(&validated_arg)
                    .map_err(GeneratorError::IdValidationError)?;
                fields.push(validated_arg.dom);
            }

            Instruction::CallNativeMethod {
                method_ident: NativeMethodIdent {
                    receiver,
                    method_name,
                },
                args: args_from_value_vec!(fields),
            }
        }

        ast::Instruction::PublishPackage { code, abi } => Instruction::PublishPackage {
            code: generate_blob(code, blobs)?,
            abi: generate_blob(abi, blobs)?,
        },
        ast::Instruction::CreateResource {
            resource_type,
            metadata,
            access_rules,
            mint_params,
        } => {
            // Generates call data
            let mut args = Vec::new();
            for arg in [
                generate_value(resource_type, None, resolver, bech32_decoder, blobs)?,
                generate_value(metadata, None, resolver, bech32_decoder, blobs)?,
                generate_value(access_rules, None, resolver, bech32_decoder, blobs)?,
                generate_value(mint_params, None, resolver, bech32_decoder, blobs)?,
            ] {
                let validated_arg = ScryptoValue::from_value(arg)
                    .expect("Failed to convert value into ScryptoValue");
                id_validator
                    .move_resources(&validated_arg)
                    .map_err(GeneratorError::IdValidationError)?;
                args.push(validated_arg.dom);
            }
            let args = args_from_value_vec!(args);

            // Check if call data matches ABI
            if scrypto_decode::<ResourceManagerCreateInput>(&args).is_err() {
                return Err(GeneratorError::ArgumentsDoNotMatchAbi);
            }

            Instruction::CallNativeFunction {
                function_ident: NativeFunctionIdent {
                    blueprint_name: "ResourceManager".to_owned(),
                    function_name: ResourceManagerFunction::Create.to_string(),
                },
                args,
            }
        }
        ast::Instruction::BurnBucket { bucket } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            Instruction::CallNativeFunction {
                function_ident: NativeFunctionIdent {
                    blueprint_name: "ResourceManager".to_owned(),
                    function_name: ResourceManagerFunction::BurnBucket.to_string(),
                },
                args: scrypto_encode(&ResourceManagerBurnInput {
                    bucket: scrypto::resource::Bucket(bucket_id),
                }),
            }
        }
        ast::Instruction::MintFungible {
            resource_address,
            amount,
        } => {
            let resource_address = generate_resource_address(resource_address, bech32_decoder)?;
            let input = ResourceManagerMintInput {
                mint_params: MintParams::Fungible {
                    amount: generate_decimal(amount)?,
                },
            };

            Instruction::CallNativeMethod {
                method_ident: NativeMethodIdent {
                    receiver: RENodeId::Global(GlobalAddress::Resource(resource_address)),
                    method_name: ResourceManagerMethod::Mint.to_string(),
                },
                args: args!(input),
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
    bech32_decoder: &Bech32Decoder,
    blobs: &HashMap<Hash, Vec<u8>>,
) -> Result<Vec<Vec<u8>>, GeneratorError> {
    let mut result = Vec::new();
    for v in values {
        let value = generate_value(v, None, resolver, bech32_decoder, blobs)?;

        result.push(encode_any(&value));
    }
    Ok(result)
}

fn generate_string(value: &ast::Value) -> Result<String, GeneratorError> {
    match value {
        ast::Value::String(s) => Ok(s.into()),
        v => invalid_type!(v, ast::Type::String),
    }
}

fn generate_decimal(value: &ast::Value) -> Result<Decimal, GeneratorError> {
    match value {
        ast::Value::Decimal(inner) => match &**inner {
            ast::Value::String(s) => {
                Decimal::from_str(s).map_err(|_| GeneratorError::InvalidDecimal(s.into()))
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Decimal),
    }
}

fn generate_precise_decimal(value: &ast::Value) -> Result<PreciseDecimal, GeneratorError> {
    match value {
        ast::Value::PreciseDecimal(inner) => match &**inner {
            ast::Value::String(s) => PreciseDecimal::from_str(s)
                .map_err(|_| GeneratorError::InvalidPreciseDecimal(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Decimal),
    }
}

fn generate_ecdsa_secp256k1_public_key(
    value: &ast::Value,
) -> Result<EcdsaSecp256k1PublicKey, GeneratorError> {
    match value {
        ast::Value::EcdsaSecp256k1PublicKey(inner) => match &**inner {
            ast::Value::String(s) => EcdsaSecp256k1PublicKey::from_str(s)
                .map_err(|_| GeneratorError::InvalidEcdsaSecp256k1PublicKey(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::EcdsaSecp256k1PublicKey),
    }
}

fn generate_ecdsa_secp256k1_signature(
    value: &ast::Value,
) -> Result<EcdsaSecp256k1Signature, GeneratorError> {
    match value {
        ast::Value::EcdsaSecp256k1Signature(inner) => match &**inner {
            ast::Value::String(s) => EcdsaSecp256k1Signature::from_str(s)
                .map_err(|_| GeneratorError::InvalidEcdsaSecp256k1Signature(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::EcdsaSecp256k1Signature),
    }
}

fn generate_eddsa_ed25519_public_key(
    value: &ast::Value,
) -> Result<EddsaEd25519PublicKey, GeneratorError> {
    match value {
        ast::Value::EddsaEd25519PublicKey(inner) => match &**inner {
            ast::Value::String(s) => EddsaEd25519PublicKey::from_str(s)
                .map_err(|_| GeneratorError::InvalidEddsaEd25519PublicKey(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::EddsaEd25519PublicKey),
    }
}

fn generate_eddsa_ed25519_signature(
    value: &ast::Value,
) -> Result<EddsaEd25519Signature, GeneratorError> {
    match value {
        ast::Value::EddsaEd25519Signature(inner) => match &**inner {
            ast::Value::String(s) => EddsaEd25519Signature::from_str(s)
                .map_err(|_| GeneratorError::InvalidEddsaEd25519Signature(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::EddsaEd25519Signature),
    }
}

fn generate_package_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<PackageAddress, GeneratorError> {
    match value {
        ast::Value::PackageAddress(inner) => match &**inner {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_package_address(s)
                .map_err(|_| GeneratorError::InvalidPackageAddress(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::PackageAddress),
    }
}

fn generate_system_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<SystemAddress, GeneratorError> {
    match value {
        ast::Value::SystemAddress(inner) => match &**inner {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_system_address(s)
                .map_err(|_| GeneratorError::InvalidSystemAddress(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::SystemAddress),
    }
}

fn generate_component_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<ComponentAddress, GeneratorError> {
    match value {
        ast::Value::ComponentAddress(inner) => match &**inner {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_component_address(s)
                .map_err(|_| GeneratorError::InvalidComponentAddress(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::ComponentAddress),
    }
}

fn generate_resource_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<ResourceAddress, GeneratorError> {
    match value {
        ast::Value::ResourceAddress(inner) => match &**inner {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_resource_address(s)
                .map_err(|_| GeneratorError::InvalidResourceAddress(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::ResourceAddress),
    }
}

fn generate_scrypto_receiver(
    receiver: &ast::ScryptoReceiver,
    bech32_decoder: &Bech32Decoder,
) -> Result<ScryptoReceiver, GeneratorError> {
    match receiver {
        ast::ScryptoReceiver::Global(v) => match v {
            ast::Value::String(s) => Ok(ScryptoReceiver::Global(
                bech32_decoder
                    .validate_and_decode_component_address(&s)
                    .map_err(|_| GeneratorError::InvalidResourceAddress(s.to_owned()))?,
            )),
            v => invalid_type!(v, ast::Type::String),
        },
        ast::ScryptoReceiver::Component(v) => Ok(ScryptoReceiver::Component(generate_node_id(v)?)),
    }
}

fn generate_receiver(
    receiver: &ast::Receiver,
    bech32_decoder: &Bech32Decoder,
    resolver: &mut NameResolver,
) -> Result<RENodeId, GeneratorError> {
    match receiver {
        ast::Receiver::Ref(re_node) => Ok(generate_re_node_id(re_node, bech32_decoder, resolver)?),
    }
}

fn generate_re_node_id(
    re_node: &ast::RENode,
    bech32_decoder: &Bech32Decoder,
    resolver: &mut NameResolver,
) -> Result<RENodeId, GeneratorError> {
    match re_node {
        ast::RENode::Bucket(value) => {
            let bucket_id = match value {
                ast::Value::U32(n) => Ok(*n),
                ast::Value::String(s) => resolver
                    .resolve_bucket(&s)
                    .map_err(GeneratorError::NameResolverError),
                v => invalid_type!(v, ast::Type::U32, ast::Type::String),
            }?;

            Ok(RENodeId::Bucket(bucket_id))
        }
        ast::RENode::Proof(value) => {
            let bucket_id = match value {
                ast::Value::U32(n) => Ok(*n),
                ast::Value::String(s) => resolver
                    .resolve_proof(&s)
                    .map_err(GeneratorError::NameResolverError),
                v => invalid_type!(v, ast::Type::U32, ast::Type::String),
            }?;

            Ok(RENodeId::Bucket(bucket_id))
        }
        ast::RENode::AuthZoneStack(value) => {
            let auth_zone_id = match value {
                ast::Value::U32(v) => Ok(*v),
                v => invalid_type!(v, ast::Type::U32),
            }?;
            Ok(RENodeId::AuthZoneStack(auth_zone_id))
        }
        ast::RENode::Worktop => Ok(RENodeId::Worktop),
        ast::RENode::KeyValueStore(node_id) => {
            Ok(RENodeId::KeyValueStore(generate_node_id(node_id)?))
        }
        ast::RENode::NonFungibleStore(node_id) => {
            Ok(RENodeId::NonFungibleStore(generate_node_id(node_id)?))
        }
        ast::RENode::Component(node_id) => Ok(RENodeId::Component(generate_node_id(node_id)?)),
        ast::RENode::EpochManager(node_id) => {
            Ok(RENodeId::EpochManager(generate_node_id(node_id)?))
        }
        ast::RENode::Vault(node_id) => Ok(RENodeId::Vault(generate_node_id(node_id)?)),
        ast::RENode::ResourceManager(node_id) => {
            Ok(RENodeId::ResourceManager(generate_node_id(node_id)?))
        }
        ast::RENode::Package(node_id) => Ok(RENodeId::Package(generate_node_id(node_id)?)),
        ast::RENode::Global(value) => match value {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_package_address(s)
                .map(|a| RENodeId::Global(GlobalAddress::Package(a)))
                .or_else(|_| {
                    bech32_decoder
                        .validate_and_decode_component_address(s)
                        .map(|a| RENodeId::Global(GlobalAddress::Component(a)))
                })
                .or_else(|_| {
                    bech32_decoder
                        .validate_and_decode_resource_address(s)
                        .map(|a| RENodeId::Global(GlobalAddress::Resource(a)))
                })
                .map_err(|_| GeneratorError::InvalidGlobal(s.into())),
            v => return invalid_type!(v, ast::Type::String),
        },
    }
}

fn generate_node_id(node_id: &ast::Value) -> Result<(Hash, u32), GeneratorError> {
    match node_id {
        ast::Value::String(s) => {
            if s.len() != 2 * (Hash::LENGTH + 4) {
                return Err(GeneratorError::InvalidNodeId(s.into()));
            }
            let hash = Hash::from_str(&s[0..Hash::LENGTH * 2])
                .map_err(|_| GeneratorError::InvalidNodeId(s.into()))?;
            let index = u32::from_str_radix(&s[Hash::LENGTH * 2..], 16)
                .map_err(|_| GeneratorError::InvalidNodeId(s.into()))?;
            Ok((hash, index))
        }
        v => invalid_type!(v, ast::Type::String),
    }
}

fn generate_hash(value: &ast::Value) -> Result<Hash, GeneratorError> {
    match value {
        ast::Value::Hash(inner) => match &**inner {
            ast::Value::String(s) => {
                Hash::from_str(s).map_err(|_| GeneratorError::InvalidHash(s.into()))
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Hash),
    }
}

fn generate_component(value: &ast::Value) -> Result<Component, GeneratorError> {
    match value {
        ast::Value::Component(inner) => match &**inner {
            ast::Value::String(s) => {
                Component::from_str(s).map_err(|_| GeneratorError::InvalidComponent(s.into()))
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Component),
    }
}

fn generate_key_value_store(value: &ast::Value) -> Result<KeyValueStore<(), ()>, GeneratorError> {
    match value {
        ast::Value::KeyValueStore(inner) => match &**inner {
            ast::Value::String(s) => KeyValueStore::from_str(s)
                .map_err(|_| GeneratorError::InvalidKeyValueStore(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::KeyValueStore),
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
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Bucket),
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
            v => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Bucket),
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
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Proof),
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
            v => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Proof),
    }
}

fn generate_vault(value: &ast::Value) -> Result<Vault, GeneratorError> {
    match value {
        ast::Value::Vault(inner) => match &**inner {
            ast::Value::String(s) => {
                Vault::from_str(s).map_err(|_| GeneratorError::InvalidVault(s.into()))
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Vault),
    }
}

fn generate_non_fungible_id(value: &ast::Value) -> Result<NonFungibleId, GeneratorError> {
    match value {
        ast::Value::NonFungibleId(inner) => match &**inner {
            ast::Value::String(s) => NonFungibleId::from_str(s)
                .map_err(|_| GeneratorError::InvalidNonFungibleId(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::NonFungibleId),
    }
}

fn generate_non_fungible_address(value: &ast::Value) -> Result<NonFungibleAddress, GeneratorError> {
    match value {
        ast::Value::NonFungibleAddress(inner) => match &**inner {
            ast::Value::String(s) => NonFungibleAddress::from_str(s)
                .map_err(|_| GeneratorError::InvalidNonFungibleAddress(s.into())),
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::NonFungibleAddress),
    }
}

fn generate_expression(value: &ast::Value) -> Result<Expression, GeneratorError> {
    match value {
        ast::Value::Expression(inner) => match &**inner {
            ast::Value::String(s) => {
                Expression::from_str(s).map_err(|_| GeneratorError::InvalidExpression(s.into()))
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Expression),
    }
}

fn generate_blob(
    value: &ast::Value,
    blobs: &HashMap<Hash, Vec<u8>>,
) -> Result<Blob, GeneratorError> {
    match value {
        ast::Value::Blob(inner) => match &**inner {
            ast::Value::String(s) => {
                let hash = Hash::from_str(s).map_err(|_| GeneratorError::InvalidBlobHash)?;
                blobs
                    .get(&hash)
                    .ok_or(GeneratorError::BlobNotFound(s.clone()))?;
                Ok(Blob(hash))
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Blob),
    }
}

fn generate_non_fungible_ids(
    value: &ast::Value,
) -> Result<BTreeSet<NonFungibleId>, GeneratorError> {
    match value {
        ast::Value::Array(kind, values) => {
            if kind != &ast::Type::NonFungibleId {
                return Err(GeneratorError::InvalidType {
                    expected_type: ast::Type::String,
                    actual: kind.clone(),
                });
            }

            values.iter().map(|v| generate_non_fungible_id(v)).collect()
        }
        v => invalid_type!(v, ast::Type::Array),
    }
}

fn generate_value(
    value: &ast::Value,
    expected: Option<ast::Type>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &HashMap<Hash, Vec<u8>>,
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
        ast::Value::Bool(value) => Ok(Value::Bool { value: *value }),
        ast::Value::I8(value) => Ok(Value::I8 { value: *value }),
        ast::Value::I16(value) => Ok(Value::I16 { value: *value }),
        ast::Value::I32(value) => Ok(Value::I32 { value: *value }),
        ast::Value::I64(value) => Ok(Value::I64 { value: *value }),
        ast::Value::I128(value) => Ok(Value::I128 { value: *value }),
        ast::Value::U8(value) => Ok(Value::U8 { value: *value }),
        ast::Value::U16(value) => Ok(Value::U16 { value: *value }),
        ast::Value::U32(value) => Ok(Value::U32 { value: *value }),
        ast::Value::U64(value) => Ok(Value::U64 { value: *value }),
        ast::Value::U128(value) => Ok(Value::U128 { value: *value }),
        ast::Value::String(value) => Ok(Value::String {
            value: value.clone(),
        }),
        ast::Value::Struct(fields) => Ok(Value::Struct {
            fields: generate_singletons(fields, None, resolver, bech32_decoder, blobs)?,
        }),
        ast::Value::Enum(discriminator, fields) => Ok(Value::Enum {
            discriminator: discriminator.clone(),
            fields: generate_singletons(fields, None, resolver, bech32_decoder, blobs)?,
        }),
        ast::Value::Array(element_type, elements) => Ok(Value::Array {
            element_type_id: generate_type_id(element_type),
            elements: generate_singletons(
                elements,
                Some(*element_type),
                resolver,
                bech32_decoder,
                blobs,
            )?,
        }),
        ast::Value::Tuple(elements) => Ok(Value::Tuple {
            elements: generate_singletons(elements, None, resolver, bech32_decoder, blobs)?,
        }),
        ast::Value::PackageAddress(_) => {
            generate_package_address(value, bech32_decoder).map(|v| Value::Custom {
                type_id: ScryptoType::PackageAddress.id(),
                bytes: v.to_vec(),
            })
        }
        ast::Value::SystemAddress(_) => {
            generate_system_address(value, bech32_decoder).map(|v| Value::Custom {
                type_id: ScryptoType::SystemAddress.id(),
                bytes: v.to_vec(),
            })
        }
        ast::Value::ComponentAddress(_) => {
            generate_component_address(value, bech32_decoder).map(|v| Value::Custom {
                type_id: ScryptoType::ComponentAddress.id(),
                bytes: v.to_vec(),
            })
        }
        ast::Value::ResourceAddress(_) => {
            generate_resource_address(value, bech32_decoder).map(|v| Value::Custom {
                type_id: ScryptoType::ResourceAddress.id(),
                bytes: v.to_vec(),
            })
        }

        ast::Value::Component(_) => generate_component(value).map(|v| Value::Custom {
            type_id: ScryptoType::Component.id(),
            bytes: v.to_vec(),
        }),
        ast::Value::KeyValueStore(_) => generate_key_value_store(value).map(|v| Value::Custom {
            type_id: ScryptoType::KeyValueStore.id(),
            bytes: v.to_vec(),
        }),
        ast::Value::Bucket(_) => generate_bucket(value, resolver).map(|v| Value::Custom {
            type_id: ScryptoType::Bucket.id(),
            bytes: scrypto::resource::Bucket(v).to_vec(),
        }),
        ast::Value::Proof(_) => generate_proof(value, resolver).map(|v| Value::Custom {
            type_id: ScryptoType::Proof.id(),
            bytes: scrypto::resource::Proof(v).to_vec(),
        }),
        ast::Value::Vault(_) => generate_vault(value).map(|v| Value::Custom {
            type_id: ScryptoType::Vault.id(),
            bytes: v.to_vec(),
        }),

        ast::Value::Expression(_) => generate_expression(value).map(|v| Value::Custom {
            type_id: ScryptoType::Expression.id(),
            bytes: v.to_vec(),
        }),
        ast::Value::Blob(_) => generate_blob(value, blobs).map(|v| Value::Custom {
            type_id: ScryptoType::Blob.id(),
            bytes: v.to_vec(),
        }),
        ast::Value::NonFungibleAddress(_) => {
            generate_non_fungible_address(value).map(|v| Value::Custom {
                type_id: ScryptoType::NonFungibleAddress.id(),
                bytes: v.to_vec(),
            })
        }

        ast::Value::Hash(_) => generate_hash(value).map(|v| Value::Custom {
            type_id: ScryptoType::Hash.id(),
            bytes: v.to_vec(),
        }),
        ast::Value::Decimal(_) => generate_decimal(value).map(|v| Value::Custom {
            type_id: ScryptoType::Decimal.id(),
            bytes: v.to_vec(),
        }),
        ast::Value::PreciseDecimal(_) => generate_precise_decimal(value).map(|v| Value::Custom {
            type_id: ScryptoType::PreciseDecimal.id(),
            bytes: v.to_vec(),
        }),
        ast::Value::EcdsaSecp256k1PublicKey(_) => {
            generate_ecdsa_secp256k1_public_key(value).map(|v| Value::Custom {
                type_id: ScryptoType::EcdsaSecp256k1PublicKey.id(),
                bytes: v.to_vec(),
            })
        }
        ast::Value::EcdsaSecp256k1Signature(_) => {
            generate_ecdsa_secp256k1_signature(value).map(|v| Value::Custom {
                type_id: ScryptoType::EcdsaSecp256k1Signature.id(),
                bytes: v.to_vec(),
            })
        }
        ast::Value::EddsaEd25519PublicKey(_) => {
            generate_eddsa_ed25519_public_key(value).map(|v| Value::Custom {
                type_id: ScryptoType::EddsaEd25519PublicKey.id(),
                bytes: v.to_vec(),
            })
        }
        ast::Value::EddsaEd25519Signature(_) => {
            generate_eddsa_ed25519_signature(value).map(|v| Value::Custom {
                type_id: ScryptoType::EddsaEd25519Signature.id(),
                bytes: v.to_vec(),
            })
        }
        ast::Value::NonFungibleId(_) => generate_non_fungible_id(value).map(|v| Value::Custom {
            type_id: ScryptoType::NonFungibleId.id(),
            bytes: v.to_vec(),
        }),
    }
}

fn generate_singletons(
    elements: &Vec<ast::Value>,
    ty: Option<ast::Type>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &HashMap<Hash, Vec<u8>>,
) -> Result<Vec<Value>, GeneratorError> {
    let mut result = vec![];
    for element in elements {
        result.push(generate_value(
            element,
            ty,
            resolver,
            bech32_decoder,
            blobs,
        )?);
    }
    Ok(result)
}

fn generate_type_id(ty: &ast::Type) -> u8 {
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
        ast::Type::Array => TYPE_ARRAY,
        ast::Type::Tuple => TYPE_TUPLE,

        // Globals
        ast::Type::PackageAddress => ScryptoType::PackageAddress.id(),
        ast::Type::ComponentAddress => ScryptoType::ComponentAddress.id(),
        ast::Type::ResourceAddress => ScryptoType::ResourceAddress.id(),
        ast::Type::SystemAddress => ScryptoType::SystemAddress.id(),

        // RE Nodes
        ast::Type::Component => ScryptoType::Component.id(),
        ast::Type::KeyValueStore => ScryptoType::KeyValueStore.id(),
        ast::Type::Bucket => ScryptoType::Bucket.id(),
        ast::Type::Proof => ScryptoType::Proof.id(),
        ast::Type::Vault => ScryptoType::Vault.id(),

        // Other interpreted types
        ast::Type::Expression => ScryptoType::Expression.id(),
        ast::Type::Blob => ScryptoType::Blob.id(),
        ast::Type::NonFungibleAddress => ScryptoType::NonFungibleAddress.id(),

        // Uninterpreted=> ScryptoType::Decimal.id(),
        ast::Type::Hash => ScryptoType::Hash.id(),
        ast::Type::EcdsaSecp256k1PublicKey => ScryptoType::EcdsaSecp256k1PublicKey.id(),
        ast::Type::EcdsaSecp256k1Signature => ScryptoType::EcdsaSecp256k1Signature.id(),
        ast::Type::EddsaEd25519PublicKey => ScryptoType::EddsaEd25519PublicKey.id(),
        ast::Type::EddsaEd25519Signature => ScryptoType::EddsaEd25519Signature.id(),
        ast::Type::Decimal => ScryptoType::Decimal.id(),
        ast::Type::PreciseDecimal => ScryptoType::PreciseDecimal.id(),
        ast::Type::NonFungibleId => ScryptoType::NonFungibleId.id(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::lexer::tokenize;
    use crate::manifest::parser::Parser;
    use scrypto::address::Bech32Decoder;
    use scrypto::core::NetworkDefinition;
    use scrypto::{args, pdec};

    #[macro_export]
    macro_rules! generate_value_ok {
        ( $s:expr,   $expected:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap()).parse_value().unwrap();
            let mut resolver = NameResolver::new();
            assert_eq!(
                generate_value(
                    &value,
                    None,
                    &mut resolver,
                    &Bech32Decoder::new(&NetworkDefinition::simulator()),
                    &mut HashMap::new()
                ),
                Ok($expected)
            );
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
                generate_instruction(
                    &instruction,
                    &mut id_validator,
                    &mut resolver,
                    &Bech32Decoder::new(&NetworkDefinition::simulator()),
                    &mut HashMap::new()
                ),
                Ok($expected)
            );
        }};
    }

    #[macro_export]
    macro_rules! generate_value_error {
        ( $s:expr, $expected:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap()).parse_value().unwrap();
            match generate_value(
                &value,
                None,
                &mut NameResolver::new(),
                &Bech32Decoder::new(&NetworkDefinition::simulator()),
                &mut HashMap::new(),
            ) {
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
        generate_value_ok!(r#"true"#, Value::Bool { value: true });
        generate_value_ok!(r#"false"#, Value::Bool { value: false });
        generate_value_ok!(r#"1i8"#, Value::I8 { value: 1 });
        generate_value_ok!(r#"1i128"#, Value::I128 { value: 1 });
        generate_value_ok!(r#"1u8"#, Value::U8 { value: 1 });
        generate_value_ok!(r#"1u128"#, Value::U128 { value: 1 });
        generate_value_ok!(
            r#"Struct(Bucket(1u32), Proof(2u32), "bar")"#,
            Value::Struct {
                fields: vec![
                    Value::Custom {
                        type_id: ScryptoType::Bucket.id(),
                        bytes: scrypto::resource::Bucket(1).to_vec()
                    },
                    Value::Custom {
                        type_id: ScryptoType::Proof.id(),
                        bytes: scrypto::resource::Proof(2).to_vec()
                    },
                    Value::String {
                        value: "bar".into()
                    }
                ]
            }
        );
        generate_value_ok!(
            r#"Struct(Decimal("1.0"), Hash("aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c"))"#,
            Value::Struct {
                fields: vec![
                    Value::Custom {
                        type_id: ScryptoType::Decimal.id(),
                        bytes: Decimal::from_str("1.0").unwrap().to_vec()
                    },
                    Value::Custom {
                        type_id: ScryptoType::Hash.id(),
                        bytes: Hash::from_str(
                            "aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c"
                        )
                        .unwrap()
                        .to_vec()
                    },
                ]
            }
        );
        generate_value_ok!(r#"Struct()"#, Value::Struct { fields: vec![] });
        generate_value_ok!(
            r#"Enum("Variant", "abc")"#,
            Value::Enum {
                discriminator: "Variant".to_string(),
                fields: vec![Value::String {
                    value: "abc".to_owned()
                }]
            }
        );
        generate_value_ok!(
            r#"Enum("Variant")"#,
            Value::Enum {
                discriminator: "Variant".to_string(),
                fields: vec![]
            }
        );
        generate_value_ok!(
            r#"Expression("ENTIRE_WORKTOP")"#,
            Value::Custom {
                type_id: ScryptoType::Expression.id(),
                bytes: scrypto::core::Expression("ENTIRE_WORKTOP".to_owned()).to_vec()
            }
        );
    }

    #[test]
    fn test_failures() {
        generate_value_error!(
            r#"ComponentAddress(100u32)"#,
            GeneratorError::InvalidValue {
                expected_type: vec![ast::Type::String],
                actual: ast::Value::U32(100),
            }
        );
        generate_value_error!(
            r#"PackageAddress("invalid_package_address")"#,
            GeneratorError::InvalidPackageAddress("invalid_package_address".into())
        );
        generate_value_error!(
            r#"Decimal("invalid_decimal")"#,
            GeneratorError::InvalidDecimal("invalid_decimal".into())
        );
    }

    #[test]
    fn test_instructions() {
        let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());
        let component1 = bech32_decoder
            .validate_and_decode_component_address(
                "component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum",
            )
            .unwrap();

        generate_instruction_ok!(
            r#"TAKE_FROM_WORKTOP_BY_AMOUNT  Decimal("1.0")  ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak")  Bucket("xrd_bucket");"#,
            Instruction::TakeFromWorktopByAmount {
                amount: Decimal::from(1),
                resource_address: Bech32Decoder::for_simulator()
                    .validate_and_decode_resource_address(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak"
                    )
                    .unwrap(),
            }
        );
        generate_instruction_ok!(
            r#"TAKE_FROM_WORKTOP  ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak")  Bucket("xrd_bucket");"#,
            Instruction::TakeFromWorktop {
                resource_address: Bech32Decoder::for_simulator()
                    .validate_and_decode_resource_address(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak"
                    )
                    .unwrap(),
            }
        );
        generate_instruction_ok!(
            r#"ASSERT_WORKTOP_CONTAINS_BY_AMOUNT  Decimal("1.0")  ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak");"#,
            Instruction::AssertWorktopContainsByAmount {
                amount: Decimal::from(1),
                resource_address: Bech32Decoder::for_simulator()
                    .validate_and_decode_resource_address(
                        "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak"
                    )
                    .unwrap(),
            }
        );
        generate_instruction_ok!(
            r#"CALL_FUNCTION  PackageAddress("package_sim1q8gl2qqsusgzmz92es68wy2fr7zjc523xj57eanm597qrz3dx7")  "Airdrop"  "new"  500u32  PreciseDecimal("120");"#,
            Instruction::CallFunction {
                function_ident: ScryptoFunctionIdent {
                    package: ScryptoPackage::Global(
                        Bech32Decoder::for_simulator()
                            .validate_and_decode_package_address(
                                "package_sim1q8gl2qqsusgzmz92es68wy2fr7zjc523xj57eanm597qrz3dx7"
                                    .into()
                            )
                            .unwrap()
                    ),
                    blueprint_name: "Airdrop".into(),
                    function_name: "new".to_string(),
                },
                args: args!(500u32, pdec!("120"))
            }
        );
        generate_instruction_ok!(
            r#"CALL_METHOD  ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")  "refill";"#,
            Instruction::CallMethod {
                method_ident: ScryptoMethodIdent {
                    receiver: ScryptoReceiver::Global(component1),
                    method_name: "refill".to_string(),
                },
                args: args!()
            }
        );
    }
}
