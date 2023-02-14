use radix_engine_interface::address::Bech32Decoder;
use radix_engine_interface::api::types::GlobalAddress;
use radix_engine_interface::crypto::{
    EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature, EddsaEd25519PublicKey, EddsaEd25519Signature,
    Hash,
};
use radix_engine_interface::data::types::*;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, IndexedScryptoValue, ScryptoCustomValue, ScryptoDecode,
    ScryptoValue, ScryptoValueKind,
};
use radix_engine_interface::math::{Decimal, PreciseDecimal};
use radix_engine_interface::model::*;
use sbor::rust::borrow::Borrow;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::str::FromStr;
use sbor::rust::vec;
use sbor::value_kind::*;
use sbor::*;

use crate::errors::*;
use crate::manifest::ast;
use crate::model::*;
use crate::validation::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorError {
    InvalidAstType {
        expected_type: ast::Type,
        actual: ast::Type,
    },
    InvalidAstValue {
        expected_type: Vec<ast::Type>,
        actual: ast::Value,
    },
    UnexpectedValue {
        expected_type: ScryptoValueKind,
        actual: ast::Value,
    },
    InvalidPackageAddress(String),
    InvalidComponentAddress(String),
    InvalidResourceAddress(String),
    InvalidDecimal(String),
    InvalidPreciseDecimal(String),
    InvalidHash(String),
    InvalidNodeId(String),
    InvalidKeyValueStoreId(String),
    InvalidVaultId(String),
    InvalidNonFungibleLocalId(String),
    InvalidNonFungibleGlobalId,
    InvalidExpression(String),
    InvalidComponent(String),
    InvalidKeyValueStore(String),
    InvalidVault(String),
    InvalidEcdsaSecp256k1PublicKey(String),
    InvalidEcdsaSecp256k1Signature(String),
    InvalidEddsaEd25519PublicKey(String),
    InvalidEddsaEd25519Signature(String),
    InvalidBlobHash,
    BlobNotFound(String),
    InvalidBytesHex(String),
    SborEncodeError(EncodeError),
    NameResolverError(NameResolverError),
    IdValidationError(ManifestIdValidationError),
    ArgumentEncodingError(EncodeError),
    ArgumentDecodingError(DecodeError),
    InvalidEntityAddress(String),
    InvalidLength {
        value_type: ast::Type,
        expected_length: usize,
        actual: usize,
    },
    OddNumberOfElements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameResolverError {
    UndefinedBucket(String),
    UndefinedProof(String),
    NamedAlreadyDefined(String),
}

pub struct NameResolver {
    named_buckets: BTreeMap<String, ManifestBucket>,
    named_proofs: BTreeMap<String, ManifestProof>,
}

impl NameResolver {
    pub fn new() -> Self {
        Self {
            named_buckets: BTreeMap::new(),
            named_proofs: BTreeMap::new(),
        }
    }

    pub fn insert_bucket(
        &mut self,
        name: String,
        bucket_id: ManifestBucket,
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
        proof_id: ManifestProof,
    ) -> Result<(), NameResolverError> {
        if self.named_buckets.contains_key(&name) || self.named_proofs.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_proofs.insert(name, proof_id);
            Ok(())
        }
    }

    pub fn resolve_bucket(&mut self, name: &str) -> Result<ManifestBucket, NameResolverError> {
        match self.named_buckets.get(name).cloned() {
            Some(bucket_id) => Ok(bucket_id),
            None => Err(NameResolverError::UndefinedBucket(name.into())),
        }
    }

    pub fn resolve_proof(&mut self, name: &str) -> Result<ManifestProof, NameResolverError> {
        match self.named_proofs.get(name).cloned() {
            Some(proof_id) => Ok(proof_id),
            None => Err(NameResolverError::UndefinedProof(name.into())),
        }
    }
}

pub fn generate_manifest(
    instructions: &[ast::Instruction],
    bech32_decoder: &Bech32Decoder,
    blobs: BTreeMap<Hash, Vec<u8>>,
) -> Result<TransactionManifest, GeneratorError> {
    let mut id_validator = ManifestIdValidator::new();
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
    id_validator: &mut ManifestIdValidator,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<BasicInstruction, GeneratorError> {
    Ok(match instruction {
        ast::Instruction::TakeFromWorktop {
            resource_address,
            new_bucket,
        } => {
            let bucket_id = id_validator
                .new_bucket()
                .map_err(GeneratorError::IdValidationError)?;
            declare_bucket(new_bucket, resolver, bucket_id)?;

            BasicInstruction::TakeFromWorktop {
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

            BasicInstruction::TakeFromWorktopByAmount {
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

            BasicInstruction::TakeFromWorktopByIds {
                ids: generate_non_fungible_local_ids(ids)?,
                resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            }
        }
        ast::Instruction::ReturnToWorktop { bucket } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(&bucket_id)
                .map_err(GeneratorError::IdValidationError)?;
            BasicInstruction::ReturnToWorktop { bucket_id }
        }
        ast::Instruction::AssertWorktopContains { resource_address } => {
            BasicInstruction::AssertWorktopContains {
                resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            }
        }
        ast::Instruction::AssertWorktopContainsByAmount {
            amount,
            resource_address,
        } => BasicInstruction::AssertWorktopContainsByAmount {
            amount: generate_decimal(amount)?,
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
        },
        ast::Instruction::AssertWorktopContainsByIds {
            ids,
            resource_address,
        } => BasicInstruction::AssertWorktopContainsByIds {
            ids: generate_non_fungible_local_ids(ids)?,
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
        },
        ast::Instruction::PopFromAuthZone { new_proof } => {
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            BasicInstruction::PopFromAuthZone
        }
        ast::Instruction::PushToAuthZone { proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(&proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            BasicInstruction::PushToAuthZone { proof_id }
        }
        ast::Instruction::ClearAuthZone => BasicInstruction::ClearAuthZone,

        ast::Instruction::CreateProofFromAuthZone {
            resource_address,
            new_proof,
        } => {
            let resource_address = generate_resource_address(resource_address, bech32_decoder)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            BasicInstruction::CreateProofFromAuthZone { resource_address }
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

            BasicInstruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_address,
            }
        }
        ast::Instruction::CreateProofFromAuthZoneByIds {
            ids,
            resource_address,
            new_proof,
        } => {
            let ids = generate_non_fungible_local_ids(ids)?;
            let resource_address = generate_resource_address(resource_address, bech32_decoder)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            BasicInstruction::CreateProofFromAuthZoneByIds {
                ids,
                resource_address,
            }
        }
        ast::Instruction::CreateProofFromBucket { bucket, new_proof } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            let proof_id = id_validator
                .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            BasicInstruction::CreateProofFromBucket { bucket_id }
        }
        ast::Instruction::CloneProof { proof, new_proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            let proof_id2 = id_validator
                .clone_proof(&proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id2)?;

            BasicInstruction::CloneProof { proof_id }
        }
        ast::Instruction::DropProof { proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(&proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            BasicInstruction::DropProof { proof_id }
        }
        ast::Instruction::DropAllProofs => {
            id_validator
                .drop_all_proofs()
                .map_err(GeneratorError::IdValidationError)?;
            BasicInstruction::DropAllProofs
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

            let indexed_args = IndexedScryptoValue::from_value(args);
            id_validator
                .move_resources(&indexed_args.buckets(), &indexed_args.proofs())
                .map_err(GeneratorError::IdValidationError)?;

            BasicInstruction::CallFunction {
                package_address,
                blueprint_name,
                function_name,
                args: indexed_args.to_vec(),
            }
        }
        ast::Instruction::CallMethod {
            component_address,
            method_name,
            args,
        } => {
            let component_address = generate_component_address(component_address, bech32_decoder)?;
            let method_name = generate_string(&method_name)?;
            let args = generate_args(args, resolver, bech32_decoder, blobs)?;

            let indexed_args = IndexedScryptoValue::from_value(args);
            id_validator
                .move_resources(&indexed_args.buckets(), &indexed_args.proofs())
                .map_err(GeneratorError::IdValidationError)?;

            BasicInstruction::CallMethod {
                component_address,
                method_name,
                args: indexed_args.into_vec(),
            }
        }
        ast::Instruction::PublishPackage {
            code,
            abi,
            royalty_config,
            metadata,
            access_rules,
        } => BasicInstruction::PublishPackage {
            code: generate_blob(code, blobs)?,
            abi: generate_blob(abi, blobs)?,
            royalty_config: generate_typed_value(royalty_config, resolver, bech32_decoder, blobs)?,
            metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
            access_rules: generate_typed_value(access_rules, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::PublishPackageWithOwner {
            code,
            abi,
            owner_badge,
        } => BasicInstruction::PublishPackageWithOwner {
            code: generate_blob(code, blobs)?,
            abi: generate_blob(abi, blobs)?,
            owner_badge: generate_non_fungible_global_id(owner_badge, bech32_decoder)?,
        },
        ast::Instruction::BurnResource { bucket } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(&bucket_id)
                .map_err(GeneratorError::IdValidationError)?;
            BasicInstruction::BurnResource { bucket_id }
        }
        ast::Instruction::RecallResource { vault_id, amount } => BasicInstruction::RecallResource {
            vault_id: generate_typed_value(vault_id, resolver, bech32_decoder, blobs)?,
            amount: generate_decimal(amount)?,
        },
        ast::Instruction::SetMetadata {
            entity_address,
            key,
            value,
        } => BasicInstruction::SetMetadata {
            entity_address: generate_entity_address(entity_address, bech32_decoder)?,
            key: generate_string(key)?,
            value: generate_string(value)?,
        },
        ast::Instruction::SetPackageRoyaltyConfig {
            package_address,
            royalty_config,
        } => BasicInstruction::SetPackageRoyaltyConfig {
            package_address: generate_package_address(package_address, bech32_decoder)?,
            royalty_config: generate_typed_value(royalty_config, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::SetComponentRoyaltyConfig {
            component_address,
            royalty_config,
        } => BasicInstruction::SetComponentRoyaltyConfig {
            component_address: generate_component_address(component_address, bech32_decoder)?,
            royalty_config: generate_typed_value(royalty_config, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::ClaimPackageRoyalty { package_address } => {
            BasicInstruction::ClaimPackageRoyalty {
                package_address: generate_package_address(package_address, bech32_decoder)?,
            }
        }
        ast::Instruction::ClaimComponentRoyalty { component_address } => {
            BasicInstruction::ClaimComponentRoyalty {
                component_address: generate_component_address(component_address, bech32_decoder)?,
            }
        }
        ast::Instruction::SetMethodAccessRule {
            entity_address,
            index,
            key,
            rule,
        } => BasicInstruction::SetMethodAccessRule {
            entity_address: generate_entity_address(entity_address, bech32_decoder)?,
            index: generate_typed_value(index, resolver, bech32_decoder, blobs)?,
            key: generate_typed_value(key, resolver, bech32_decoder, blobs)?,
            rule: generate_typed_value(rule, resolver, bech32_decoder, blobs)?,
        },

        ast::Instruction::MintFungible {
            resource_address,
            amount,
        } => BasicInstruction::MintFungible {
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            amount: generate_decimal(amount)?,
        },
        ast::Instruction::MintNonFungible {
            resource_address,
            entries,
        } => BasicInstruction::MintNonFungible {
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            entries: generate_non_fungible_mint_params(entries, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::MintUuidNonFungible {
            resource_address,
            entries,
        } => BasicInstruction::MintUuidNonFungible {
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            entries: generate_uuid_non_fungible_mint_params(
                entries,
                resolver,
                bech32_decoder,
                blobs,
            )?,
        },

        ast::Instruction::CreateFungibleResource {
            divisibility,
            metadata,
            access_rules,
            initial_supply,
        } => BasicInstruction::CreateFungibleResource {
            divisibility: generate_u8(divisibility)?,
            metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
            access_rules: generate_typed_value(access_rules, resolver, bech32_decoder, blobs)?,
            initial_supply: generate_typed_value(initial_supply, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::CreateFungibleResourceWithOwner {
            divisibility,
            metadata,
            owner_badge,
            initial_supply,
        } => BasicInstruction::CreateFungibleResourceWithOwner {
            divisibility: generate_u8(divisibility)?,
            metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
            owner_badge: generate_non_fungible_global_id(owner_badge, bech32_decoder)?,
            initial_supply: generate_typed_value(initial_supply, resolver, bech32_decoder, blobs)?,
        },

        ast::Instruction::CreateNonFungibleResource {
            id_type,
            metadata,
            access_rules,
            initial_supply,
        } => BasicInstruction::CreateNonFungibleResource {
            id_type: generate_typed_value(id_type, resolver, bech32_decoder, blobs)?,
            metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
            access_rules: generate_typed_value(access_rules, resolver, bech32_decoder, blobs)?,
            initial_supply: generate_from_enum_if_some(
                initial_supply,
                resolver,
                bech32_decoder,
                blobs,
                generate_non_fungible_mint_params,
            )?,
        },
        ast::Instruction::CreateNonFungibleResourceWithOwner {
            id_type,
            metadata,
            owner_badge,
            initial_supply,
        } => BasicInstruction::CreateNonFungibleResourceWithOwner {
            id_type: generate_typed_value(id_type, resolver, bech32_decoder, blobs)?,
            metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
            owner_badge: generate_non_fungible_global_id(owner_badge, bech32_decoder)?,
            initial_supply: generate_from_enum_if_some(
                initial_supply,
                resolver,
                bech32_decoder,
                blobs,
                generate_non_fungible_mint_params,
            )?,
        },
        ast::Instruction::CreateValidator {
            key,
            owner_access_rule,
        } => BasicInstruction::CreateValidator {
            key: generate_typed_value(key, resolver, bech32_decoder, blobs)?,
            owner_access_rule: generate_typed_value(
                owner_access_rule,
                resolver,
                bech32_decoder,
                blobs,
            )?,
        },
        ast::Instruction::CreateAccessController {
            controlled_asset,
            primary_role,
            recovery_role,
            confirmation_role,
            timed_recovery_delay_in_minutes,
        } => BasicInstruction::CreateAccessController {
            controlled_asset: generate_typed_value(
                controlled_asset,
                resolver,
                bech32_decoder,
                blobs,
            )?,
            primary_role: generate_typed_value(primary_role, resolver, bech32_decoder, blobs)?,
            recovery_role: generate_typed_value(recovery_role, resolver, bech32_decoder, blobs)?,
            confirmation_role: generate_typed_value(
                confirmation_role,
                resolver,
                bech32_decoder,
                blobs,
            )?,
            timed_recovery_delay_in_minutes: generate_typed_value(
                timed_recovery_delay_in_minutes,
                resolver,
                bech32_decoder,
                blobs,
            )?,
        },
        ast::Instruction::CreateIdentity { access_rule } => BasicInstruction::CreateIdentity {
            access_rule: generate_typed_value(access_rule, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::AssertAccessRule { access_rule } => BasicInstruction::AssertAccessRule {
            access_rule: generate_typed_value(access_rule, resolver, bech32_decoder, blobs)?,
        },
    })
}

#[macro_export]
macro_rules! invalid_type {
    ( $v:expr, $($exp:expr),+ ) => {
        Err(GeneratorError::InvalidAstValue {
            expected_type: vec!($($exp),+),
            actual: $v.clone(),
        })
    };
}

fn generate_typed_value<T: ScryptoDecode>(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<T, GeneratorError> {
    let value = generate_value(value, None, resolver, bech32_decoder, blobs)?;
    let encoded = scrypto_encode(&value).map_err(GeneratorError::ArgumentEncodingError)?;
    let decoded: T = scrypto_decode(&encoded).map_err(GeneratorError::ArgumentDecodingError)?;
    Ok(decoded)
}

fn generate_args(
    values: &Vec<ast::Value>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<ScryptoValue, GeneratorError> {
    let mut fields = Vec::new();
    for v in values {
        fields.push(generate_value(v, None, resolver, bech32_decoder, blobs)?);
    }

    Ok(ScryptoValue::Tuple { fields })
}

fn generate_string(value: &ast::Value) -> Result<String, GeneratorError> {
    match value {
        ast::Value::String(s) => Ok(s.into()),
        v => invalid_type!(v, ast::Type::String),
    }
}

fn generate_u8(value: &ast::Value) -> Result<u8, GeneratorError> {
    match value {
        ast::Value::U8(inner) => Ok(*inner),
        v => invalid_type!(v, ast::Type::U8),
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

fn generate_resource_address_internal(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<ResourceAddress, GeneratorError> {
    match value {
        ast::Value::String(s) => bech32_decoder
            .validate_and_decode_resource_address(s)
            .map_err(|_| GeneratorError::InvalidResourceAddress(s.into())),
        v => invalid_type!(v, ast::Type::String),
    }
}

fn generate_resource_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<ResourceAddress, GeneratorError> {
    match value {
        ast::Value::ResourceAddress(inner) => {
            generate_resource_address_internal(inner, bech32_decoder)
        }
        v => invalid_type!(v, ast::Type::ResourceAddress),
    }
}

fn generate_entity_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<GlobalAddress, GeneratorError> {
    match value {
        ast::Value::PackageAddress(value) => match value.borrow() {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_package_address(s)
                .map(|a| GlobalAddress::Package(a))
                .map_err(|_| GeneratorError::InvalidEntityAddress(s.into())),
            v => return invalid_type!(v, ast::Type::String),
        },
        ast::Value::ComponentAddress(value) => match value.borrow() {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_component_address(s)
                .map(|a| GlobalAddress::Component(a))
                .map_err(|_| GeneratorError::InvalidEntityAddress(s.into())),
            v => return invalid_type!(v, ast::Type::String),
        },
        ast::Value::ResourceAddress(value) => match value.borrow() {
            ast::Value::String(s) => bech32_decoder
                .validate_and_decode_resource_address(s)
                .map(|a| GlobalAddress::Resource(a))
                .map_err(|_| GeneratorError::InvalidEntityAddress(s.into())),
            v => return invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(
            v,
            ast::Type::PackageAddress,
            ast::Type::ResourceAddress,
            ast::Type::ComponentAddress
        ),
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

fn generate_ownership(value: &ast::Value) -> Result<Own, GeneratorError> {
    match value {
        ast::Value::Own(inner) => match &**inner {
            ast::Value::String(_) => {
                todo!()
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Own),
    }
}

fn declare_bucket(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bucket_id: ManifestBucket,
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
) -> Result<ManifestBucket, GeneratorError> {
    match value {
        ast::Value::Bucket(inner) => match &**inner {
            ast::Value::U32(n) => Ok(ManifestBucket(*n)),
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
    proof_id: ManifestProof,
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
) -> Result<ManifestProof, GeneratorError> {
    match value {
        ast::Value::Proof(inner) => match &**inner {
            ast::Value::U32(n) => Ok(ManifestProof(*n)),
            ast::Value::String(s) => resolver
                .resolve_proof(&s)
                .map_err(GeneratorError::NameResolverError),
            v => invalid_type!(v, ast::Type::U32, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Proof),
    }
}

fn generate_non_fungible_local_id(
    value: &ast::Value,
) -> Result<NonFungibleLocalId, GeneratorError> {
    match value {
        ast::Value::NonFungibleLocalId(inner) => match inner.as_ref() {
            ast::Value::String(s) => NonFungibleLocalId::from_str(s.as_str())
                .map_err(|_| GeneratorError::InvalidNonFungibleLocalId(s.clone())),
            v => invalid_type!(v, ast::Type::String)?,
        },
        v => invalid_type!(v, ast::Type::NonFungibleLocalId),
    }
}

fn generate_non_fungible_global_id(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<NonFungibleGlobalId, GeneratorError> {
    match value {
        ast::Value::Tuple(elements) => {
            if elements.len() != 2 {
                return Err(GeneratorError::InvalidNonFungibleGlobalId);
            }
            let resource_address = generate_resource_address(&elements[0], bech32_decoder)?;
            let non_fungible_local_id = generate_non_fungible_local_id(&elements[1])?;
            Ok(NonFungibleGlobalId::new(
                resource_address,
                non_fungible_local_id,
            ))
        }
        ast::Value::NonFungibleGlobalId(value) => match value.as_ref() {
            ast::Value::String(s) => {
                NonFungibleGlobalId::try_from_canonical_string(bech32_decoder, s.as_str())
                    .map_err(|_| GeneratorError::InvalidNonFungibleGlobalId)
            }
            v => invalid_type!(v, ast::Type::String)?,
        },
        v => invalid_type!(v, ast::Type::NonFungibleGlobalId, ast::Type::Tuple),
    }
}

fn generate_expression(value: &ast::Value) -> Result<ManifestExpression, GeneratorError> {
    match value {
        ast::Value::Expression(inner) => match &**inner {
            ast::Value::String(s) => match s.as_str() {
                "ENTIRE_WORKTOP" => Ok(ManifestExpression::EntireWorktop),
                "ENTIRE_AUTH_ZONE" => Ok(ManifestExpression::EntireAuthZone),
                _ => Err(GeneratorError::InvalidExpression(s.into())),
            },
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Expression),
    }
}

fn generate_blob(
    value: &ast::Value,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<ManifestBlobRef, GeneratorError> {
    match value {
        ast::Value::Blob(inner) => match &**inner {
            ast::Value::String(s) => {
                let hash = Hash::from_str(s).map_err(|_| GeneratorError::InvalidBlobHash)?;
                blobs
                    .get(&hash)
                    .ok_or(GeneratorError::BlobNotFound(s.clone()))?;
                Ok(ManifestBlobRef(hash))
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::Blob),
    }
}

fn generate_non_fungible_local_ids(
    value: &ast::Value,
) -> Result<BTreeSet<NonFungibleLocalId>, GeneratorError> {
    match value {
        ast::Value::Array(kind, values) => {
            if kind != &ast::Type::NonFungibleLocalId {
                return Err(GeneratorError::InvalidAstType {
                    expected_type: ast::Type::String,
                    actual: kind.clone(),
                });
            }

            values
                .iter()
                .map(|v| generate_non_fungible_local_id(v))
                .collect()
        }
        v => invalid_type!(v, ast::Type::Array),
    }
}

fn generate_byte_vec_from_hex(value: &ast::Value) -> Result<Vec<u8>, GeneratorError> {
    let bytes = match value {
        ast::Value::String(s) => {
            hex::decode(s).map_err(|_| GeneratorError::InvalidBytesHex(s.to_owned()))?
        }
        v => invalid_type!(v, ast::Type::String)?,
    };
    Ok(bytes)
}

/// This function generates args from an [`ast::Value`]. This is useful when minting NFTs to be able
/// to specify their data in a human readable format instead of SBOR.
fn generate_args_from_tuple(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<ScryptoValue, GeneratorError> {
    match value {
        ast::Value::Tuple(values) => generate_args(values, resolver, bech32_decoder, blobs),
        v => invalid_type!(v, ast::Type::Tuple),
    }
}

fn generate_from_enum_if_some<F, T>(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
    generator: F,
) -> Result<Option<T>, GeneratorError>
where
    F: Fn(
        &ast::Value,
        &mut NameResolver,
        &Bech32Decoder,
        &BTreeMap<Hash, Vec<u8>>,
    ) -> Result<T, GeneratorError>,
{
    let value = match value {
        ast::Value::None => {
            return Ok(None);
        }
        ast::Value::Enum(variant, fields)
            if *variant == OPTION_VARIANT_NONE && fields.len() == 0 =>
        {
            return Ok(None);
        }
        ast::Value::Some(value) => &**value,
        ast::Value::Enum(variant, fields)
            if *variant == OPTION_VARIANT_SOME && fields.len() == 1 =>
        {
            &fields[0]
        }
        v => invalid_type!(v, ast::Type::Enum)?,
    };
    Ok(Some(generator(value, resolver, bech32_decoder, blobs)?))
}

/// This function generates the mint parameters of a non fungible resource from an array which has
/// the following structure:
///
/// Map<NonFungibleLocalId, Tuple>
/// - Every key is a NonFungibleLocalId
/// - Every value is a Tuple of length 2
///    - [0] Tuple (immutable data)
///    - [1] Tuple (mutable data)
fn generate_non_fungible_mint_params(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>, GeneratorError> {
    match value {
        ast::Value::Map(key_type, value_type, elements) => {
            if key_type != &ast::Type::NonFungibleLocalId {
                return Err(GeneratorError::InvalidAstType {
                    expected_type: ast::Type::NonFungibleLocalId,
                    actual: key_type.clone(),
                });
            };
            if value_type != &ast::Type::Tuple {
                return Err(GeneratorError::InvalidAstType {
                    expected_type: ast::Type::Tuple,
                    actual: value_type.clone(),
                });
            };
            if elements.len() % 2 != 0 {
                return Err(GeneratorError::OddNumberOfElements);
            }

            let mut mint_params = BTreeMap::new();
            for i in 0..elements.len() / 2 {
                let non_fungible_local_id = generate_non_fungible_local_id(&elements[i * 2])?;
                let non_fungible_data = match elements[i * 2 + 1].clone() {
                    ast::Value::Tuple(values) => {
                        if values.len() != 2 {
                            return Err(GeneratorError::InvalidLength {
                                value_type: ast::Type::Tuple,
                                expected_length: 2,
                                actual: values.len(),
                            });
                        }

                        let immutable_data = scrypto_encode(&generate_args_from_tuple(
                            &values[0],
                            resolver,
                            bech32_decoder,
                            blobs,
                        )?)
                        .map_err(GeneratorError::ArgumentEncodingError)?;
                        let mutable_data = scrypto_encode(&generate_args_from_tuple(
                            &values[1],
                            resolver,
                            bech32_decoder,
                            blobs,
                        )?)
                        .map_err(GeneratorError::ArgumentEncodingError)?;

                        (immutable_data, mutable_data)
                    }
                    v => invalid_type!(v, ast::Type::Tuple)?,
                };
                mint_params.insert(non_fungible_local_id, non_fungible_data);
            }

            Ok(mint_params)
        }
        v => invalid_type!(v, ast::Type::Array)?,
    }
}

fn generate_uuid_non_fungible_mint_params(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>, GeneratorError> {
    match value {
        ast::Value::Array(kind, elements) => {
            if kind != &ast::Type::Tuple {
                return Err(GeneratorError::InvalidAstType {
                    expected_type: ast::Type::Tuple,
                    actual: kind.clone(),
                });
            };

            let mut mint_params = Vec::new();
            for element in elements.into_iter() {
                match element {
                    ast::Value::Tuple(values) => {
                        if values.len() != 2 {
                            return Err(GeneratorError::InvalidLength {
                                value_type: ast::Type::Tuple,
                                expected_length: 2,
                                actual: values.len(),
                            });
                        }

                        let immutable_data = scrypto_encode(&generate_args_from_tuple(
                            &values[0],
                            resolver,
                            bech32_decoder,
                            blobs,
                        )?)
                        .map_err(GeneratorError::ArgumentEncodingError)?;
                        let mutable_data = scrypto_encode(&generate_args_from_tuple(
                            &values[1],
                            resolver,
                            bech32_decoder,
                            blobs,
                        )?)
                        .map_err(GeneratorError::ArgumentEncodingError)?;

                        mint_params.push((immutable_data, mutable_data));
                    }
                    v => invalid_type!(v, ast::Type::Tuple)?,
                }
            }

            Ok(mint_params)
        }
        v => invalid_type!(v, ast::Type::Array)?,
    }
}

pub fn generate_value(
    value: &ast::Value,
    expected_type: Option<ScryptoValueKind>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<ScryptoValue, GeneratorError> {
    if let Some(ty) = expected_type {
        if ty != value.value_kind() {
            return Err(GeneratorError::UnexpectedValue {
                expected_type: ty,
                actual: value.clone(),
            });
        }
    }

    match value {
        // ==============
        // Basic types
        // ==============
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
        ast::Value::Tuple(fields) => Ok(Value::Tuple {
            fields: generate_singletons(fields, None, resolver, bech32_decoder, blobs)?,
        }),
        ast::Value::Enum(discriminator, fields) => Ok(Value::Enum {
            discriminator: discriminator.clone(),
            fields: generate_singletons(fields, None, resolver, bech32_decoder, blobs)?,
        }),
        ast::Value::Array(element_type, elements) => {
            let element_value_kind = element_type.value_kind();
            Ok(Value::Array {
                element_value_kind,
                elements: generate_singletons(
                    elements,
                    Some(element_value_kind),
                    resolver,
                    bech32_decoder,
                    blobs,
                )?,
            })
        }
        ast::Value::Map(key_type, value_type, entries) => {
            let key_value_kind = key_type.value_kind();
            let value_value_kind = value_type.value_kind();
            Ok(Value::Map {
                key_value_kind,
                value_value_kind,
                entries: generate_kv_entries(
                    entries,
                    key_value_kind,
                    value_value_kind,
                    resolver,
                    bech32_decoder,
                    blobs,
                )?,
            })
        }
        // ==============
        // Aliases
        // ==============
        ast::Value::Some(value) => Ok(Value::Enum {
            discriminator: OPTION_VARIANT_SOME,
            fields: vec![generate_value(
                value,
                None,
                resolver,
                bech32_decoder,
                blobs,
            )?],
        }),
        ast::Value::None => Ok(Value::Enum {
            discriminator: OPTION_VARIANT_NONE,
            fields: vec![],
        }),
        ast::Value::Ok(value) => Ok(Value::Enum {
            discriminator: RESULT_VARIANT_OK,
            fields: vec![generate_value(
                value,
                None,
                resolver,
                bech32_decoder,
                blobs,
            )?],
        }),
        ast::Value::Err(value) => Ok(Value::Enum {
            discriminator: RESULT_VARIANT_ERR,
            fields: vec![generate_value(
                value,
                None,
                resolver,
                bech32_decoder,
                blobs,
            )?],
        }),
        ast::Value::Bytes(value) => {
            let bytes = generate_byte_vec_from_hex(value)?;
            Ok(Value::Array {
                element_value_kind: ValueKind::U8,
                elements: bytes.iter().map(|i| Value::U8 { value: *i }).collect(),
            })
        }
        ast::Value::NonFungibleGlobalId(value) => {
            let global_id = match value.as_ref() {
                ast::Value::String(s) => {
                    NonFungibleGlobalId::try_from_canonical_string(bech32_decoder, s.as_str())
                        .map_err(|_| GeneratorError::InvalidNonFungibleGlobalId)
                }
                v => invalid_type!(v, ast::Type::String)?,
            }?;
            Ok(Value::Tuple {
                fields: vec![
                    Value::Custom {
                        value: ScryptoCustomValue::ResourceAddress(global_id.resource_address()),
                    },
                    Value::Custom {
                        value: ScryptoCustomValue::NonFungibleLocalId(global_id.local_id().clone()),
                    },
                ],
            })
        }
        // ==============
        // Custom Types
        // ==============
        ast::Value::PackageAddress(_) => {
            generate_package_address(value, bech32_decoder).map(|v| Value::Custom {
                value: ScryptoCustomValue::PackageAddress(v),
            })
        }
        ast::Value::ComponentAddress(_) => {
            generate_component_address(value, bech32_decoder).map(|v| Value::Custom {
                value: ScryptoCustomValue::ComponentAddress(v),
            })
        }
        ast::Value::ResourceAddress(_) => {
            generate_resource_address(value, bech32_decoder).map(|v| Value::Custom {
                value: ScryptoCustomValue::ResourceAddress(v),
            })
        }

        ast::Value::Own(_) => generate_ownership(value).map(|v| Value::Custom {
            value: ScryptoCustomValue::Own(v),
        }),
        ast::Value::Blob(_) => generate_blob(value, blobs).map(|v| Value::Custom {
            value: ScryptoCustomValue::Blob(v),
        }),

        ast::Value::Bucket(_) => generate_bucket(value, resolver).map(|v| Value::Custom {
            value: ScryptoCustomValue::Bucket(v),
        }),
        ast::Value::Proof(_) => generate_proof(value, resolver).map(|v| Value::Custom {
            value: ScryptoCustomValue::Proof(v),
        }),
        ast::Value::Expression(_) => generate_expression(value).map(|v| Value::Custom {
            value: ScryptoCustomValue::Expression(v),
        }),

        ast::Value::Hash(_) => generate_hash(value).map(|v| Value::Custom {
            value: ScryptoCustomValue::Hash(v),
        }),
        ast::Value::Decimal(_) => generate_decimal(value).map(|v| Value::Custom {
            value: ScryptoCustomValue::Decimal(v),
        }),
        ast::Value::PreciseDecimal(_) => generate_precise_decimal(value).map(|v| Value::Custom {
            value: ScryptoCustomValue::PreciseDecimal(v),
        }),
        ast::Value::EcdsaSecp256k1PublicKey(_) => {
            generate_ecdsa_secp256k1_public_key(value).map(|v| Value::Custom {
                value: ScryptoCustomValue::EcdsaSecp256k1PublicKey(v),
            })
        }
        ast::Value::EcdsaSecp256k1Signature(_) => {
            generate_ecdsa_secp256k1_signature(value).map(|v| Value::Custom {
                value: ScryptoCustomValue::EcdsaSecp256k1Signature(v),
            })
        }
        ast::Value::EddsaEd25519PublicKey(_) => {
            generate_eddsa_ed25519_public_key(value).map(|v| Value::Custom {
                value: ScryptoCustomValue::EddsaEd25519PublicKey(v),
            })
        }
        ast::Value::EddsaEd25519Signature(_) => {
            generate_eddsa_ed25519_signature(value).map(|v| Value::Custom {
                value: ScryptoCustomValue::EddsaEd25519Signature(v),
            })
        }
        ast::Value::NonFungibleLocalId(_) => {
            generate_non_fungible_local_id(value).map(|v| Value::Custom {
                value: ScryptoCustomValue::NonFungibleLocalId(v),
            })
        }
    }
}

fn generate_singletons(
    elements: &Vec<ast::Value>,
    expected_type: Option<ScryptoValueKind>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<Vec<ScryptoValue>, GeneratorError> {
    let mut result = vec![];
    for element in elements {
        result.push(generate_value(
            element,
            expected_type,
            resolver,
            bech32_decoder,
            blobs,
        )?);
    }
    Ok(result)
}

fn generate_kv_entries(
    elements: &Vec<ast::Value>,
    key_value_kind: ScryptoValueKind,
    value_value_kind: ScryptoValueKind,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<Vec<(ScryptoValue, ScryptoValue)>, GeneratorError> {
    if elements.len() % 2 != 0 {
        return Err(GeneratorError::OddNumberOfElements);
    }

    let mut result = vec![];
    for i in 0..elements.len() / 2 {
        let key = generate_value(
            &elements[i * 2],
            Some(key_value_kind),
            resolver,
            bech32_decoder,
            blobs,
        )?;
        let value = generate_value(
            &elements[i * 2 + 1],
            Some(value_value_kind),
            resolver,
            bech32_decoder,
            blobs,
        )?;
        result.push((key, value));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::lexer::tokenize;
    use crate::manifest::parser::Parser;
    use crate::signing::EcdsaSecp256k1PrivateKey;
    use radix_engine_interface::address::Bech32Decoder;
    use radix_engine_interface::args;
    use radix_engine_interface::node::NetworkDefinition;
    use radix_engine_interface::pdec;

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
                    &mut BTreeMap::new()
                ),
                Ok($expected)
            );
        }};
    }

    #[macro_export]
    macro_rules! generate_instruction_ok {
        ( $s:expr, $expected:expr, $($blob_hash: expr),* ) => {{
            // If you use the following output for test cases, make sure you've checked the diff
            // println!("{}", crate::manifest::decompile(&[$expected.clone()], &NetworkDefinition::simulator()).unwrap());
            let instruction = Parser::new(tokenize($s).unwrap())
                .parse_instruction()
                .unwrap();
            let mut id_validator = ManifestIdValidator::new();
            let mut resolver = NameResolver::new();
            assert_eq!(
                generate_instruction(
                    &instruction,
                    &mut id_validator,
                    &mut resolver,
                    &Bech32Decoder::new(&NetworkDefinition::simulator()),
                    &mut BTreeMap::from([
                        $(
                            (($blob_hash).parse().unwrap(), Vec::new()),
                        )*
                    ])
                ),
                Ok($expected)
            );
        }}
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
                &mut BTreeMap::new(),
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
        generate_value_ok!(r#"Tuple()"#, Value::Tuple { fields: vec![] });
        generate_value_ok!(r#"true"#, Value::Bool { value: true });
        generate_value_ok!(r#"false"#, Value::Bool { value: false });
        generate_value_ok!(r#"1i8"#, Value::I8 { value: 1 });
        generate_value_ok!(r#"1i128"#, Value::I128 { value: 1 });
        generate_value_ok!(r#"1u8"#, Value::U8 { value: 1 });
        generate_value_ok!(r#"1u128"#, Value::U128 { value: 1 });
        generate_value_ok!(
            r#"Tuple(Bucket(1u32), Proof(2u32), "bar")"#,
            Value::Tuple {
                fields: vec![
                    Value::Custom {
                        value: ScryptoCustomValue::Bucket(ManifestBucket(1))
                    },
                    Value::Custom {
                        value: ScryptoCustomValue::Proof(ManifestProof(2))
                    },
                    Value::String {
                        value: "bar".into()
                    }
                ]
            }
        );
        generate_value_ok!(
            r#"Tuple(Decimal("1"), Hash("aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c"))"#,
            Value::Tuple {
                fields: vec![
                    Value::Custom {
                        value: ScryptoCustomValue::Decimal(Decimal::from_str("1").unwrap())
                    },
                    Value::Custom {
                        value: ScryptoCustomValue::Hash(
                            Hash::from_str(
                                "aa37f5a71083a9aa044fb936678bfd74f848e930d2de482a49a73540ea72aa5c"
                            )
                            .unwrap()
                        )
                    },
                ]
            }
        );
        generate_value_ok!(r#"Tuple()"#, Value::Tuple { fields: vec![] });
        generate_value_ok!(
            r#"Enum(0u8, "abc")"#,
            Value::Enum {
                discriminator: 0,
                fields: vec![Value::String {
                    value: "abc".to_owned()
                }]
            }
        );
        generate_value_ok!(
            r#"Enum(1u8)"#,
            Value::Enum {
                discriminator: 1,
                fields: vec![]
            }
        );
        generate_value_ok!(
            r#"Enum("AccessRule::AllowAll")"#,
            Value::Enum {
                discriminator: 0,
                fields: vec![]
            }
        );
        generate_value_ok!(
            r#"Expression("ENTIRE_WORKTOP")"#,
            Value::Custom {
                value: ScryptoCustomValue::Expression(ManifestExpression::EntireWorktop)
            }
        );
    }

    #[test]
    fn test_failures() {
        generate_value_error!(
            r#"ComponentAddress(100u32)"#,
            GeneratorError::InvalidAstValue {
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
        let component = bech32_decoder
            .validate_and_decode_component_address(
                "component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum",
            )
            .unwrap();
        let resource = bech32_decoder
            .validate_and_decode_resource_address(
                "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak",
            )
            .unwrap();
        let owner_badge = NonFungibleGlobalId::new(resource, NonFungibleLocalId::integer(1));

        generate_instruction_ok!(
            r#"TAKE_FROM_WORKTOP_BY_AMOUNT  Decimal("1")  ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak")  Bucket("xrd_bucket");"#,
            BasicInstruction::TakeFromWorktopByAmount {
                amount: Decimal::from(1),
                resource_address: resource,
            },
        );
        generate_instruction_ok!(
            r#"TAKE_FROM_WORKTOP  ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak")  Bucket("xrd_bucket");"#,
            BasicInstruction::TakeFromWorktop {
                resource_address: resource
            },
        );
        generate_instruction_ok!(
            r#"ASSERT_WORKTOP_CONTAINS_BY_AMOUNT  Decimal("1")  ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak");"#,
            BasicInstruction::AssertWorktopContainsByAmount {
                amount: Decimal::from(1),
                resource_address: resource,
            },
        );
        generate_instruction_ok!(
            r#"CALL_FUNCTION  PackageAddress("package_sim1q8gl2qqsusgzmz92es68wy2fr7zjc523xj57eanm597qrz3dx7")  "Airdrop"  "new"  500u32  PreciseDecimal("120");"#,
            BasicInstruction::CallFunction {
                package_address: Bech32Decoder::for_simulator()
                    .validate_and_decode_package_address(
                        "package_sim1q8gl2qqsusgzmz92es68wy2fr7zjc523xj57eanm597qrz3dx7".into()
                    )
                    .unwrap(),
                blueprint_name: "Airdrop".into(),
                function_name: "new".to_string(),
                args: args!(500u32, pdec!("120"))
            },
        );
        generate_instruction_ok!(
            r#"CALL_METHOD  ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum")  "refill";"#,
            BasicInstruction::CallMethod {
                component_address: component,
                method_name: "refill".to_string(),
                args: args!()
            },
        );
        generate_instruction_ok!(
            r#"PUBLISH_PACKAGE Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d") Map<String, Tuple>() Map<String, String>() Tuple(Map<Enum, Enum>(), Map<String, Enum>(), Enum("AccessRule::DenyAll"), Map<Enum, Enum>(), Map<String, Enum>(), Enum("AccessRule::DenyAll"));"#,
            BasicInstruction::PublishPackage {
                code: ManifestBlobRef(
                    "36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618"
                        .parse()
                        .unwrap()
                ),
                abi: ManifestBlobRef(
                    "15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d"
                        .parse()
                        .unwrap()
                ),
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new()
            },
            "36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618",
            "15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d"
        );
        generate_instruction_ok!(
            r#"PUBLISH_PACKAGE_WITH_OWNER Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d") NonFungibleGlobalId("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak:#1#");"#,
            BasicInstruction::PublishPackageWithOwner {
                code: ManifestBlobRef(
                    "36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618"
                        .parse()
                        .unwrap()
                ),
                abi: ManifestBlobRef(
                    "15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d"
                        .parse()
                        .unwrap()
                ),
                owner_badge: owner_badge.clone()
            },
            "36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618",
            "15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d"
        );

        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE 18u8 Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll"))) Some(Decimal("500"));"#,
            BasicInstruction::CreateFungibleResource {
                divisibility: 18,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                access_rules: BTreeMap::from([
                    (
                        ResourceMethodAuthKey::Withdraw,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                    (
                        ResourceMethodAuthKey::Deposit,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                ]),
                initial_supply: Some("500".parse().unwrap())
            },
        );
        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE 18u8 Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll"))) None;"#,
            BasicInstruction::CreateFungibleResource {
                divisibility: 18,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                access_rules: BTreeMap::from([
                    (
                        ResourceMethodAuthKey::Withdraw,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                    (
                        ResourceMethodAuthKey::Deposit,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                ]),
                initial_supply: None
            },
        );
        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE_WITH_OWNER 18u8 Map<String, String>("name", "Token") NonFungibleGlobalId("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak:#1#") Some(Decimal("500"));"#,
            BasicInstruction::CreateFungibleResourceWithOwner {
                divisibility: 18,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                owner_badge: owner_badge.clone(),
                initial_supply: Some("500".parse().unwrap())
            },
        );
        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE_WITH_OWNER 18u8 Map<String, String>("name", "Token") NonFungibleGlobalId("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak:#1#") None;"#,
            BasicInstruction::CreateFungibleResourceWithOwner {
                divisibility: 18,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                owner_badge: owner_badge.clone(),
                initial_supply: None
            },
        );

        generate_instruction_ok!(
            r##"CREATE_NON_FUNGIBLE_RESOURCE Enum("NonFungibleIdType::Integer") Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll"))) Some(Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#1#"), Tuple(Tuple("Hello World", Decimal("12")), Tuple(12u8, 19u128))));"##,
            BasicInstruction::CreateNonFungibleResource {
                id_type: NonFungibleIdType::Integer,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                access_rules: BTreeMap::from([
                    (
                        ResourceMethodAuthKey::Withdraw,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                    (
                        ResourceMethodAuthKey::Deposit,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                ]),
                initial_supply: Some(BTreeMap::from([(
                    NonFungibleLocalId::integer(1),
                    (
                        args!(String::from("Hello World"), Decimal::from("12")),
                        args!(12u8, 19u128)
                    )
                )]))
            },
        );
        generate_instruction_ok!(
            r#"CREATE_NON_FUNGIBLE_RESOURCE Enum("NonFungibleIdType::Integer") Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll"))) None;"#,
            BasicInstruction::CreateNonFungibleResource {
                id_type: NonFungibleIdType::Integer,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                access_rules: BTreeMap::from([
                    (
                        ResourceMethodAuthKey::Withdraw,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                    (
                        ResourceMethodAuthKey::Deposit,
                        (AccessRule::AllowAll, AccessRule::DenyAll)
                    ),
                ]),
                initial_supply: None
            },
        );

        generate_instruction_ok!(
            r##"CREATE_NON_FUNGIBLE_RESOURCE_WITH_OWNER Enum("NonFungibleIdType::Integer") Map<String, String>("name", "Token") NonFungibleGlobalId("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak:#1#") Some(Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#1#"), Tuple(Tuple("Hello World", Decimal("12")), Tuple(12u8, 19u128))));"##,
            BasicInstruction::CreateNonFungibleResourceWithOwner {
                id_type: NonFungibleIdType::Integer,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                owner_badge: owner_badge.clone(),
                initial_supply: Some(BTreeMap::from([(
                    NonFungibleLocalId::integer(1),
                    (
                        args!(String::from("Hello World"), Decimal::from("12")),
                        args!(12u8, 19u128)
                    )
                )]))
            },
        );
        generate_instruction_ok!(
            r##"CREATE_NON_FUNGIBLE_RESOURCE_WITH_OWNER Enum("NonFungibleIdType::Integer") Map<String, String>("name", "Token") NonFungibleGlobalId("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak:#1#") None;"##,
            BasicInstruction::CreateNonFungibleResourceWithOwner {
                id_type: NonFungibleIdType::Integer,
                metadata: BTreeMap::from([("name".to_string(), "Token".to_string())]),
                owner_badge: owner_badge.clone(),
                initial_supply: None
            },
        );

        generate_instruction_ok!(
            r#"MINT_FUNGIBLE ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak") Decimal("100");"#,
            BasicInstruction::MintFungible {
                resource_address: resource,
                amount: Decimal::from_str("100").unwrap()
            },
        );
        generate_instruction_ok!(
            r##"MINT_NON_FUNGIBLE ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak") Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#1#"), Tuple(Tuple("Hello World", Decimal("12")), Tuple(12u8, 19u128)));"##,
            BasicInstruction::MintNonFungible {
                resource_address: resource,
                entries: BTreeMap::from([(
                    NonFungibleLocalId::integer(1),
                    (
                        args!(String::from("Hello World"), Decimal::from("12")),
                        args!(12u8, 19u128)
                    )
                )])
            },
        );
    }

    #[test]
    fn test_mint_uuid_non_fungible_instruction() {
        let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());
        let resource = bech32_decoder
            .validate_and_decode_resource_address(
                "resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak",
            )
            .unwrap();
        generate_instruction_ok!(
            r#"
            MINT_UUID_NON_FUNGIBLE
                ResourceAddress("resource_sim1qr9alp6h38ggejqvjl3fzkujpqj2d84gmqy72zuluzwsykwvak")
                Array<Tuple>(
                    Tuple(
                        Tuple("Hello World", Decimal("12")),
                        Tuple(12u8, 19u128)
                    )
                );
            "#,
            BasicInstruction::MintUuidNonFungible {
                resource_address: resource,
                entries: Vec::from([(
                    args!(String::from("Hello World"), Decimal::from("12")),
                    args!(12u8, 19u128)
                )])
            },
        );
    }

    #[test]
    fn test_create_validator_instruction() {
        generate_instruction_ok!(
            r#"
            CREATE_VALIDATOR EcdsaSecp256k1PublicKey("02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5") Enum("AccessRule::AllowAll");
            "#,
            BasicInstruction::CreateValidator {
                key: EcdsaSecp256k1PrivateKey::from_u64(2u64)
                    .unwrap()
                    .public_key(),
                owner_access_rule: AccessRule::AllowAll,
            },
        );
    }
}
