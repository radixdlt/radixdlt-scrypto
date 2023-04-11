use crate::data::*;
use crate::errors::*;
use crate::manifest::ast;
use crate::model::*;
use crate::validation::*;
use radix_engine_interface::address::Bech32Decoder;
use radix_engine_interface::blueprints::access_controller::RuleSet;
use radix_engine_interface::blueprints::access_controller::{
    ACCESS_CONTROLLER_BLUEPRINT, ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
};
use radix_engine_interface::blueprints::account::{
    AccountCreateAdvancedInput, AccountCreateInput, ACCOUNT_BLUEPRINT,
    ACCOUNT_CREATE_ADVANCED_IDENT, ACCOUNT_CREATE_IDENT,
};
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerCreateValidatorInput, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT,
};
use radix_engine_interface::blueprints::identity::{
    IdentityCreateAdvancedInput, IdentityCreateInput, IDENTITY_BLUEPRINT,
    IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_CREATE_IDENT,
};
use radix_engine_interface::blueprints::resource::{
    AccessRulesConfig, FungibleResourceManagerCreateInput,
    FungibleResourceManagerCreateWithInitialSupplyInput, NonFungibleResourceManagerCreateInput,
    NonFungibleResourceManagerCreateWithInitialSupplyManifestInput,
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
};
use radix_engine_interface::blueprints::resource::{
    NonFungibleGlobalId, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
};
use radix_engine_interface::constants::{
    ACCESS_CONTROLLER_PACKAGE, ACCOUNT_PACKAGE, EPOCH_MANAGER, IDENTITY_PACKAGE,
    RESOURCE_MANAGER_PACKAGE,
};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::manifest::model::*;
use radix_engine_interface::data::manifest::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::manifest_args;
use radix_engine_interface::math::{Decimal, PreciseDecimal};
use radix_engine_interface::types::ComponentAddress;
use radix_engine_interface::types::GlobalAddress;
use radix_engine_interface::types::LocalAddress;
use radix_engine_interface::types::PackageAddress;
use radix_engine_interface::types::ResourceAddress;
use sbor::rust::borrow::Borrow;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::str::FromStr;
use sbor::rust::vec;
use sbor::*;

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
        expected_type: ManifestValueKind,
        actual: ast::Value,
    },
    InvalidPackageAddress(String),
    InvalidComponentAddress(String),
    InvalidResourceAddress(String),
    InvalidDecimal(String),
    InvalidPreciseDecimal(String),
    InvalidHash(String),
    InvalidNodeId(String),
    InvalidVaultId(String),
    InvalidNonFungibleLocalId(String),
    InvalidNonFungibleGlobalId,
    InvalidExpression(String),
    InvalidComponent(String),
    InvalidKeyValueStore(String),
    InvalidBucket(String),
    InvalidProof(String),
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
    InvalidGlobalAddress(String),
    InvalidLocalAddress(String),
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
    let mut id_validator = ManifestValidator::new();
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
    id_validator: &mut ManifestValidator,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
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
                ids: generate_non_fungible_local_ids(ids)?,
                resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            }
        }
        ast::Instruction::ReturnToWorktop { bucket } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(&bucket_id)
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
            ids: generate_non_fungible_local_ids(ids)?,
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
                .drop_proof(&proof_id)
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
            let ids = generate_non_fungible_local_ids(ids)?;
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
                .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id)?;

            Instruction::CreateProofFromBucket { bucket_id }
        }
        ast::Instruction::CloneProof { proof, new_proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            let proof_id2 = id_validator
                .clone_proof(&proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            declare_proof(new_proof, resolver, proof_id2)?;

            Instruction::CloneProof { proof_id }
        }
        ast::Instruction::DropProof { proof } => {
            let proof_id = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(&proof_id)
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::DropProof { proof_id }
        }
        ast::Instruction::DropAllProofs => {
            id_validator
                .drop_all_proofs()
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::DropAllProofs
        }
        ast::Instruction::ClearSignatureProofs => {
            id_validator
                .drop_all_proofs()
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::ClearSignatureProofs
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
            id_validator
                .process_call_data(&args)
                .map_err(GeneratorError::IdValidationError)?;

            Instruction::CallFunction {
                package_address,
                blueprint_name,
                function_name,
                args: to_manifest_value(&args),
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
            id_validator
                .process_call_data(&args)
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::CallMethod {
                component_address,
                method_name,
                args,
            }
        }
        ast::Instruction::PublishPackage {
            code,
            schema,
            royalty_config,
            metadata,
        } => Instruction::PublishPackage {
            code: generate_blob(code, blobs)?,
            schema: generate_blob(schema, blobs)?,
            royalty_config: generate_typed_value(royalty_config, resolver, bech32_decoder, blobs)?,
            metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::PublishPackageAdvanced {
            code,
            schema,
            royalty_config,
            metadata,
            access_rules,
        } => Instruction::PublishPackageAdvanced {
            code: generate_blob(code, blobs)?,
            schema: generate_blob(schema, blobs)?,
            royalty_config: generate_typed_value(royalty_config, resolver, bech32_decoder, blobs)?,
            metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
            access_rules: generate_typed_value(access_rules, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::BurnResource { bucket } => {
            let bucket_id = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(&bucket_id)
                .map_err(GeneratorError::IdValidationError)?;
            Instruction::BurnResource { bucket_id }
        }
        ast::Instruction::RecallResource { vault_id, amount } => Instruction::RecallResource {
            vault_id: generate_local_address(vault_id, bech32_decoder)?,
            amount: generate_decimal(amount)?,
        },
        ast::Instruction::SetMetadata {
            entity_address,
            key,
            value,
        } => Instruction::SetMetadata {
            entity_address: generate_global_address(entity_address, bech32_decoder)?,
            key: generate_string(key)?,
            value: generate_typed_value(value, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::RemoveMetadata {
            entity_address,
            key,
        } => Instruction::RemoveMetadata {
            entity_address: generate_global_address(entity_address, bech32_decoder)?,
            key: generate_string(key)?,
        },
        ast::Instruction::SetPackageRoyaltyConfig {
            package_address,
            royalty_config,
        } => Instruction::SetPackageRoyaltyConfig {
            package_address: generate_package_address(package_address, bech32_decoder)?,
            royalty_config: generate_typed_value(royalty_config, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::SetComponentRoyaltyConfig {
            component_address,
            royalty_config,
        } => Instruction::SetComponentRoyaltyConfig {
            component_address: generate_component_address(component_address, bech32_decoder)?,
            royalty_config: generate_typed_value(royalty_config, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::ClaimPackageRoyalty { package_address } => {
            Instruction::ClaimPackageRoyalty {
                package_address: generate_package_address(package_address, bech32_decoder)?,
            }
        }
        ast::Instruction::ClaimComponentRoyalty { component_address } => {
            Instruction::ClaimComponentRoyalty {
                component_address: generate_component_address(component_address, bech32_decoder)?,
            }
        }
        ast::Instruction::SetMethodAccessRule {
            entity_address,
            key,
            rule,
        } => Instruction::SetMethodAccessRule {
            entity_address: generate_global_address(entity_address, bech32_decoder)?,
            key: generate_typed_value(key, resolver, bech32_decoder, blobs)?,
            rule: generate_typed_value(rule, resolver, bech32_decoder, blobs)?,
        },

        ast::Instruction::MintFungible {
            resource_address,
            amount,
        } => Instruction::MintFungible {
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            amount: generate_decimal(amount)?,
        },
        ast::Instruction::MintNonFungible {
            resource_address,
            args,
        } => Instruction::MintNonFungible {
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            args: generate_value(args, None, resolver, bech32_decoder, blobs)?,
        },
        ast::Instruction::MintUuidNonFungible {
            resource_address,
            args,
        } => Instruction::MintUuidNonFungible {
            resource_address: generate_resource_address(resource_address, bech32_decoder)?,
            args: generate_value(args, None, resolver, bech32_decoder, blobs)?,
        },

        ast::Instruction::CreateValidator { key } => Instruction::CallMethod {
            component_address: EPOCH_MANAGER,
            method_name: EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            args: to_manifest_value(&EpochManagerCreateValidatorInput {
                key: generate_typed_value(key, resolver, bech32_decoder, blobs)?,
            }),
        },
        ast::Instruction::CreateFungibleResource {
            divisibility,
            metadata,
            access_rules,
        } => Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value(&FungibleResourceManagerCreateInput {
                divisibility: generate_u8(divisibility)?,
                metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
                access_rules: generate_typed_value(access_rules, resolver, bech32_decoder, blobs)?,
            }),
        },
        ast::Instruction::CreateFungibleResourceWithInitialSupply {
            divisibility,
            metadata,
            access_rules,
            initial_supply,
        } => Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
            args: to_manifest_value(&FungibleResourceManagerCreateWithInitialSupplyInput {
                divisibility: generate_u8(divisibility)?,
                metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
                access_rules: generate_typed_value(access_rules, resolver, bech32_decoder, blobs)?,
                initial_supply: generate_decimal(initial_supply)?,
            }),
        },
        ast::Instruction::CreateNonFungibleResource {
            id_type,
            schema,
            metadata,
            access_rules,
        } => Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerCreateInput {
                id_type: generate_typed_value(id_type, resolver, bech32_decoder, blobs)?,
                non_fungible_schema: generate_typed_value(schema, resolver, bech32_decoder, blobs)?,
                metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
                access_rules: generate_typed_value(access_rules, resolver, bech32_decoder, blobs)?,
            }),
        },
        ast::Instruction::CreateNonFungibleResourceWithInitialSupply {
            id_type,
            schema,
            metadata,
            access_rules,
            initial_supply,
        } => Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                .to_string(),
            args: to_manifest_value(
                &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                    id_type: generate_typed_value(id_type, resolver, bech32_decoder, blobs)?,
                    non_fungible_schema: generate_typed_value(
                        schema,
                        resolver,
                        bech32_decoder,
                        blobs,
                    )?,
                    metadata: generate_typed_value(metadata, resolver, bech32_decoder, blobs)?,
                    access_rules: generate_typed_value(
                        access_rules,
                        resolver,
                        bech32_decoder,
                        blobs,
                    )?,
                    entries: generate_typed_value(initial_supply, resolver, bech32_decoder, blobs)?,
                },
            ),
        },
        ast::Instruction::CreateAccessController {
            controlled_asset,
            rule_set,
            timed_recovery_delay_in_minutes,
        } => Instruction::CallFunction {
            package_address: ACCESS_CONTROLLER_PACKAGE,
            blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
            function_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            args: manifest_args!(
                generate_typed_value::<ManifestBucket>(
                    controlled_asset,
                    resolver,
                    bech32_decoder,
                    blobs
                )?,
                generate_typed_value::<RuleSet>(rule_set, resolver, bech32_decoder, blobs)?,
                generate_typed_value::<Option<u32>>(
                    timed_recovery_delay_in_minutes,
                    resolver,
                    bech32_decoder,
                    blobs
                )?
            ),
        },
        ast::Instruction::CreateIdentity {} => Instruction::CallFunction {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_IDENT.to_string(),
            args: to_manifest_value(&IdentityCreateInput {}),
        },
        ast::Instruction::CreateIdentityAdvanced { config } => Instruction::CallFunction {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&IdentityCreateAdvancedInput {
                config: generate_typed_value::<AccessRulesConfig>(
                    config,
                    resolver,
                    bech32_decoder,
                    blobs,
                )?,
            }),
        },

        ast::Instruction::CreateAccount {} => Instruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_IDENT.to_string(),
            args: to_manifest_value(&AccountCreateInput {}),
        },
        ast::Instruction::CreateAccountAdvanced { config } => Instruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&AccountCreateAdvancedInput {
                config: generate_typed_value(config, resolver, bech32_decoder, blobs)?,
            }),
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

fn generate_typed_value<T: ManifestDecode>(
    value: &ast::Value,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<T, GeneratorError> {
    let value = generate_value(value, None, resolver, bech32_decoder, blobs)?;
    let encoded = manifest_encode(&value).map_err(GeneratorError::ArgumentEncodingError)?;
    let decoded: T =
        manifest_decode(&encoded).map_err(|e| GeneratorError::ArgumentDecodingError(e))?;
    Ok(decoded)
}

fn generate_args(
    values: &Vec<ast::Value>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<ManifestValue, GeneratorError> {
    let mut fields = Vec::new();
    for v in values {
        fields.push(generate_value(v, None, resolver, bech32_decoder, blobs)?);
    }

    Ok(ManifestValue::Tuple { fields })
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

fn generate_package_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<PackageAddress, GeneratorError> {
    match value {
        ast::Value::Address(inner) => match &**inner {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = bech32_decoder.validate_and_decode(&s) {
                    if let Ok(address) = PackageAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError::InvalidGlobalAddress(s.into()));
            }
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
        ast::Value::Address(inner) => match &**inner {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = bech32_decoder.validate_and_decode(&s) {
                    if let Ok(address) = ComponentAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError::InvalidGlobalAddress(s.into()));
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::ComponentAddress, ast::Type::Address),
    }
}

fn generate_resource_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<ResourceAddress, GeneratorError> {
    match value {
        ast::Value::Address(inner) => match inner.borrow() {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = bech32_decoder.validate_and_decode(&s) {
                    if let Ok(address) = ResourceAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError::InvalidGlobalAddress(s.into()));
            }
            v => invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(v, ast::Type::ResourceAddress),
    }
}

fn generate_global_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<GlobalAddress, GeneratorError> {
    match value {
        ast::Value::Address(value) => match value.borrow() {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = bech32_decoder.validate_and_decode(&s) {
                    if let Ok(address) = GlobalAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError::InvalidGlobalAddress(s.into()));
            }
            v => return invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(
            v,
            ast::Type::Address,
            ast::Type::PackageAddress,
            ast::Type::ResourceAddress,
            ast::Type::ComponentAddress
        ),
    }
}

fn generate_local_address(
    value: &ast::Value,
    bech32_decoder: &Bech32Decoder,
) -> Result<LocalAddress, GeneratorError> {
    match value {
        ast::Value::Address(value) => match value.borrow() {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = bech32_decoder.validate_and_decode(&s) {
                    if let Ok(address) = LocalAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError::InvalidLocalAddress(s.into()));
            }
            v => return invalid_type!(v, ast::Type::String),
        },
        v => invalid_type!(
            v,
            ast::Type::Address,
            ast::Type::PackageAddress,
            ast::Type::ResourceAddress,
            ast::Type::ComponentAddress
        ),
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
                Ok(ManifestBlobRef(hash.0))
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

pub fn generate_value(
    value: &ast::Value,
    expected_type: Option<ManifestValueKind>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<ManifestValue, GeneratorError> {
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
                        value: ManifestCustomValue::Address(ManifestAddress(
                            global_id.resource_address().into(),
                        )),
                    },
                    Value::Custom {
                        value: ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(
                            global_id.local_id().clone(),
                        )),
                    },
                ],
            })
        }
        // ==============
        // Custom Types
        // ==============
        ast::Value::Address(_) => {
            generate_global_address(value, bech32_decoder).map(|v| Value::Custom {
                value: ManifestCustomValue::Address(ManifestAddress(v.into())),
            })
        }
        ast::Value::Bucket(_) => generate_bucket(value, resolver).map(|v| Value::Custom {
            value: ManifestCustomValue::Bucket(v),
        }),
        ast::Value::Proof(_) => generate_proof(value, resolver).map(|v| Value::Custom {
            value: ManifestCustomValue::Proof(v),
        }),
        ast::Value::Expression(_) => generate_expression(value).map(|v| Value::Custom {
            value: ManifestCustomValue::Expression(v),
        }),
        ast::Value::Blob(_) => generate_blob(value, blobs).map(|v| Value::Custom {
            value: ManifestCustomValue::Blob(v),
        }),
        ast::Value::Decimal(_) => generate_decimal(value).map(|v| Value::Custom {
            value: ManifestCustomValue::Decimal(from_decimal(v)),
        }),
        ast::Value::PreciseDecimal(_) => generate_precise_decimal(value).map(|v| Value::Custom {
            value: ManifestCustomValue::PreciseDecimal(from_precise_decimal(v)),
        }),
        ast::Value::NonFungibleLocalId(_) => {
            generate_non_fungible_local_id(value).map(|v| Value::Custom {
                value: ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(v)),
            })
        }
    }
}

fn generate_singletons(
    elements: &Vec<ast::Value>,
    expected_type: Option<ManifestValueKind>,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<Vec<ManifestValue>, GeneratorError> {
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
    key_value_kind: ManifestValueKind,
    value_value_kind: ManifestValueKind,
    resolver: &mut NameResolver,
    bech32_decoder: &Bech32Decoder,
    blobs: &BTreeMap<Hash, Vec<u8>>,
) -> Result<Vec<(ManifestValue, ManifestValue)>, GeneratorError> {
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
    use crate::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
    use crate::manifest::lexer::tokenize;
    use crate::manifest::parser::Parser;
    use radix_engine_interface::address::Bech32Decoder;
    use radix_engine_interface::blueprints::resource::{
        AccessRule, AccessRulesConfig, NonFungibleDataSchema,
        NonFungibleResourceManagerMintManifestInput,
        NonFungibleResourceManagerMintUuidManifestInput, ResourceMethodAuthKey,
    };
    use radix_engine_interface::network::NetworkDefinition;
    use radix_engine_interface::types::NonFungibleData;
    use radix_engine_interface::{dec, pdec, ScryptoSbor};

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
            let mut id_validator = ManifestValidator::new();
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
                        value: ManifestCustomValue::Bucket(ManifestBucket(1))
                    },
                    Value::Custom {
                        value: ManifestCustomValue::Proof(ManifestProof(2))
                    },
                    Value::String {
                        value: "bar".into()
                    }
                ]
            }
        );
        generate_value_ok!(
            r#"Tuple(Decimal("1"))"#,
            Value::Tuple {
                fields: vec![Value::Custom {
                    value: ManifestCustomValue::Decimal(from_decimal(
                        Decimal::from_str("1").unwrap()
                    ))
                },]
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
                value: ManifestCustomValue::Expression(ManifestExpression::EntireWorktop)
            }
        );
    }

    #[test]
    fn test_failures() {
        generate_value_error!(
            r#"Address(100u32)"#,
            GeneratorError::InvalidAstValue {
                expected_type: vec![ast::Type::String],
                actual: ast::Value::U32(100),
            }
        );
        generate_value_error!(
            r#"Address("invalid_package_address")"#,
            GeneratorError::InvalidGlobalAddress("invalid_package_address".into())
        );
        generate_value_error!(
            r#"Decimal("invalid_decimal")"#,
            GeneratorError::InvalidDecimal("invalid_decimal".into())
        );
    }

    #[test]
    fn test_instructions() {
        let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());
        let package_address = PackageAddress::try_from_bech32(
            &bech32_decoder,
            "package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv".into(),
        )
        .unwrap();
        let component = ComponentAddress::try_from_bech32(
            &bech32_decoder,
            "component_sim1p8xzs5t032p03afg4p6kzyfuxgllj8uumk7st7dn869qs6vdzq",
        )
        .unwrap();
        let resource = ResourceAddress::try_from_bech32(
            &bech32_decoder,
            "resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje",
        )
        .unwrap();

        generate_instruction_ok!(
            r#"TAKE_FROM_WORKTOP_BY_AMOUNT  Decimal("1")  Address("resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje")  Bucket("xrd_bucket");"#,
            Instruction::TakeFromWorktopByAmount {
                amount: Decimal::from(1),
                resource_address: resource,
            },
        );
        generate_instruction_ok!(
            r#"TAKE_FROM_WORKTOP  Address("resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje")  Bucket("xrd_bucket");"#,
            Instruction::TakeFromWorktop {
                resource_address: resource
            },
        );
        generate_instruction_ok!(
            r#"ASSERT_WORKTOP_CONTAINS_BY_AMOUNT  Decimal("1")  Address("resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje");"#,
            Instruction::AssertWorktopContainsByAmount {
                amount: Decimal::from(1),
                resource_address: resource,
            },
        );
        generate_instruction_ok!(
            r#"CALL_FUNCTION  Address("package_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9qwvvetv")  "Airdrop"  "new"  500u32  PreciseDecimal("120");"#,
            Instruction::CallFunction {
                package_address,
                blueprint_name: "Airdrop".into(),
                function_name: "new".to_string(),
                args: manifest_args!(500u32, pdec!("120"))
            },
        );
        generate_instruction_ok!(
            r#"CALL_METHOD  Address("component_sim1p8xzs5t032p03afg4p6kzyfuxgllj8uumk7st7dn869qs6vdzq")  "refill";"#,
            Instruction::CallMethod {
                component_address: component,
                method_name: "refill".to_string(),
                args: manifest_args!()
            },
        );
        generate_instruction_ok!(
            r#"MINT_FUNGIBLE Address("resource_sim1qgyx3fwettpx9pwkgnxapfx6f8u87vdven8h6ptkwj2sfvqsje") Decimal("100");"#,
            Instruction::MintFungible {
                resource_address: resource,
                amount: dec!("100")
            },
        );
    }

    #[test]
    fn test_publish_instruction() {
        generate_instruction_ok!(
            r#"PUBLISH_PACKAGE_ADVANCED Blob("a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0") Blob("554d6e3a49e90d3be279e7ff394a01d9603cc13aa701c11c1f291f6264aa5791") Map<String, Tuple>() Map<String, String>() Tuple(Map<Tuple, Enum>(), Map<Tuple, Enum>(), Map<String, Enum>(), Enum("AccessRuleEntry::AccessRule", Enum("AccessRule::DenyAll")), Map<Tuple, Enum>(), Map<String, Enum>(), Enum("AccessRuleEntry::AccessRule", Enum("AccessRule::DenyAll")));"#,
            Instruction::PublishPackageAdvanced {
                code: ManifestBlobRef(
                    hex::decode("a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0")
                        .unwrap()
                        .try_into()
                        .unwrap()
                ),
                schema: ManifestBlobRef(
                    hex::decode("554d6e3a49e90d3be279e7ff394a01d9603cc13aa701c11c1f291f6264aa5791")
                        .unwrap()
                        .try_into()
                        .unwrap()
                ),
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new()
            },
            "a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0",
            "554d6e3a49e90d3be279e7ff394a01d9603cc13aa701c11c1f291f6264aa5791"
        );
    }

    #[test]
    fn test_create_non_fungible_instruction() {
        generate_instruction_ok!(
            r#"CREATE_NON_FUNGIBLE_RESOURCE Enum("NonFungibleIdType::Integer") Tuple(Tuple(Array<Enum>(), Array<Tuple>(), Array<Enum>()), Enum(0u8, 66u8), Array<String>()) Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")));"#,
            Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value(&NonFungibleResourceManagerCreateInput {
                    id_type: NonFungibleIdType::Integer,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
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
                }),
            },
        );
    }

    #[derive(ScryptoSbor)]
    struct MyNonFungibleData {
        name: String,
        description: String,
        stored_number: Decimal,
    }

    // Because we can't import the derive trait
    impl NonFungibleData for MyNonFungibleData {
        const MUTABLE_FIELDS: &'static [&'static str] = &["description", "stored_number"];
    }

    #[test]
    fn test_generate_non_fungible_instruction_with_specific_data() {
        // This test is mostly to assist with generating manifest instructions for the testing harness
        println!(
            "{}",
            crate::manifest::decompile(
                &[Instruction::CallFunction {
                    package_address: RESOURCE_MANAGER_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                    function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                    args: to_manifest_value(&NonFungibleResourceManagerCreateInput {
                        id_type: NonFungibleIdType::Integer,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<MyNonFungibleData>(
                        ),
                        metadata: BTreeMap::new(),
                        access_rules: BTreeMap::new(),
                    }),
                }],
                &NetworkDefinition::simulator()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_create_non_fungible_with_initial_supply_instruction() {
        generate_instruction_ok!(
            r##"CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY Enum("NonFungibleIdType::Integer") Tuple(Tuple(Array<Enum>(), Array<Tuple>(), Array<Enum>()), Enum(0u8, 66u8), Array<String>()) Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll"))) Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#1#"), Tuple(Tuple("Hello World", Decimal("12"))));"##,
            Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value(
                    &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        id_type: NonFungibleIdType::Integer,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
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
                        entries: BTreeMap::from([(
                            NonFungibleLocalId::integer(1),
                            (to_manifest_value(&(
                                String::from("Hello World"),
                                dec!("12")
                            )),),
                        )]),
                    }
                ),
            },
        );
    }

    #[test]
    fn test_create_fungible_instruction() {
        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE 18u8 Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")));"#,
            Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value(&FungibleResourceManagerCreateInput {
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
                }),
            },
        );
    }

    #[test]
    fn test_create_fungible_with_initial_supply_instruction() {
        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY 18u8 Map<String, String>("name", "Token") Map<Enum, Tuple>(Enum("ResourceMethodAuthKey::Withdraw"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll")), Enum("ResourceMethodAuthKey::Deposit"), Tuple(Enum("AccessRule::AllowAll"), Enum("AccessRule::DenyAll"))) Decimal("500");"#,
            Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value(&FungibleResourceManagerCreateWithInitialSupplyInput {
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
                    initial_supply: "500".parse().unwrap()
                })
            },
        );
    }

    #[test]
    fn test_mint_non_fungible_instruction() {
        let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());
        let resource = ResourceAddress::try_from_bech32(
            &bech32_decoder,
            "resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe",
        )
        .unwrap();

        generate_instruction_ok!(
            r##"
            MINT_NON_FUNGIBLE
                Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
                Tuple(
                    Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#1#"), Tuple(Tuple("Hello World", Decimal("12"))))
                );
            "##,
            Instruction::MintNonFungible {
                resource_address: resource,
                args: to_manifest_value(&NonFungibleResourceManagerMintManifestInput {
                    entries: BTreeMap::from([(
                        NonFungibleLocalId::integer(1),
                        (to_manifest_value(&(
                            String::from("Hello World"),
                            dec!("12")
                        )),)
                    )])
                })
            },
        );
    }

    #[test]
    fn test_mint_uuid_non_fungible_instruction() {
        let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());
        let resource = ResourceAddress::try_from_bech32(
            &bech32_decoder,
            "resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe",
        )
        .unwrap();

        generate_instruction_ok!(
            r#"
            MINT_UUID_NON_FUNGIBLE
                Address("resource_sim1q2ym536cwvvf3cy9p777t4qjczqwf79hagp3wn93srvsgvqtwe")
                Tuple(
                    Array<Tuple>(
                        Tuple(Tuple("Hello World", Decimal("12")))
                    )
                );
            "#,
            Instruction::MintUuidNonFungible {
                resource_address: resource,
                args: to_manifest_value(&NonFungibleResourceManagerMintUuidManifestInput {
                    entries: Vec::from([(to_manifest_value(&(
                        String::from("Hello World"),
                        dec!("12")
                    )),),])
                }),
            },
        );
    }

    #[test]
    fn test_create_validator_instruction() {
        generate_instruction_ok!(
            r#"
            CREATE_VALIDATOR Bytes("02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5");
            "#,
            Instruction::CallMethod {
                component_address: EPOCH_MANAGER,
                method_name: EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
                args: to_manifest_value(&EpochManagerCreateValidatorInput {
                    key: EcdsaSecp256k1PrivateKey::from_u64(2u64)
                        .unwrap()
                        .public_key(),
                }),
            },
        );
    }
}
