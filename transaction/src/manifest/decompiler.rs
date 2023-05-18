use crate::data::*;
use crate::errors::*;
use crate::model::*;
use crate::validation::*;
use radix_engine_common::native_addresses::PACKAGE_PACKAGE;
use radix_engine_interface::address::Bech32Encoder;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_SET_AUTHORITY_MUTABILITY_IDENT;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_SET_AUTHORITY_RULE_IDENT;
use radix_engine_interface::api::node_modules::metadata::METADATA_REMOVE_IDENT;
use radix_engine_interface::api::node_modules::metadata::METADATA_SET_IDENT;
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::access_controller::{
    ACCESS_CONTROLLER_BLUEPRINT, ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
};
use radix_engine_interface::blueprints::account::{
    ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_ADVANCED_IDENT, ACCOUNT_CREATE_IDENT,
};
use radix_engine_interface::blueprints::epoch_manager::EPOCH_MANAGER_CREATE_VALIDATOR_IDENT;
use radix_engine_interface::blueprints::identity::{
    IDENTITY_BLUEPRINT, IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_CREATE_IDENT,
};
use radix_engine_interface::blueprints::package::PACKAGE_BLUEPRINT;
use radix_engine_interface::blueprints::package::PACKAGE_PUBLISH_WASM_ADVANCED_IDENT;
use radix_engine_interface::blueprints::package::PACKAGE_PUBLISH_WASM_IDENT;
use radix_engine_interface::blueprints::package::{
    PACKAGE_CLAIM_ROYALTY_IDENT, PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT,
};
use radix_engine_interface::constants::{
    ACCESS_CONTROLLER_PACKAGE, ACCOUNT_PACKAGE, IDENTITY_PACKAGE, RESOURCE_PACKAGE,
};
use radix_engine_interface::data::manifest::model::*;
use radix_engine_interface::data::manifest::*;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::*;
use utils::ContextualDisplay;

#[derive(Debug, Clone)]
pub enum DecompileError {
    InvalidArguments,
    EncodeError(EncodeError),
    DecodeError(DecodeError),
    IdAllocationError(ManifestIdAllocationError),
    FormattingError(fmt::Error),
}

impl From<EncodeError> for DecompileError {
    fn from(error: EncodeError) -> Self {
        Self::EncodeError(error)
    }
}

impl From<DecodeError> for DecompileError {
    fn from(error: DecodeError) -> Self {
        Self::DecodeError(error)
    }
}

impl From<fmt::Error> for DecompileError {
    fn from(error: fmt::Error) -> Self {
        Self::FormattingError(error)
    }
}

pub struct DecompilationContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
    pub id_allocator: ManifestIdAllocator,
    pub bucket_names: NonIterMap<ManifestBucket, String>,
    pub proof_names: NonIterMap<ManifestProof, String>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(bech32_encoder: &'a Bech32Encoder) -> Self {
        Self {
            bech32_encoder: Some(bech32_encoder),
            id_allocator: ManifestIdAllocator::new(),
            bucket_names: NonIterMap::<ManifestBucket, String>::new(),
            proof_names: NonIterMap::<ManifestProof, String>::new(),
        }
    }

    pub fn new_with_optional_network(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            id_allocator: ManifestIdAllocator::new(),
            bucket_names: NonIterMap::<ManifestBucket, String>::new(),
            proof_names: NonIterMap::<ManifestProof, String>::new(),
        }
    }

    pub fn for_value_display(&'a self) -> ManifestDecompilationDisplayContext<'a> {
        ManifestDecompilationDisplayContext::with_bech32_and_names(
            self.bech32_encoder,
            &self.bucket_names,
            &self.proof_names,
        )
    }
}

/// Contract: if the instructions are from a validated notarized transaction, no error
/// should be returned.
pub fn decompile(
    instructions: &[Instruction],
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    let bech32_encoder = Bech32Encoder::new(network);
    let mut buf = String::new();
    let mut context = DecompilationContext::new(&bech32_encoder);
    for inst in instructions {
        decompile_instruction(&mut buf, inst, &mut context)?;
        buf.push('\n');
    }

    Ok(buf)
}

pub fn decompile_instruction<F: fmt::Write>(
    f: &mut F,
    instruction: &Instruction,
    context: &mut DecompilationContext,
) -> Result<(), DecompileError> {
    match instruction {
        Instruction::TakeFromWorktop {
            amount,
            resource_address,
        } => {
            let bucket_id = context
                .id_allocator
                .new_bucket_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            context.bucket_names.insert(bucket_id, name.clone());
            write!(
                f,
                "TAKE_FROM_WORKTOP\n    Address(\"{}\")\n    Decimal(\"{}\")\n    Bucket(\"{}\");",
                resource_address.display(context.bech32_encoder),
                amount,
                name
            )?;
        }
        Instruction::TakeNonFungiblesFromWorktop {
            ids,
            resource_address,
        } => {
            let bucket_id = context
                .id_allocator
                .new_bucket_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            context.bucket_names.insert(bucket_id, name.clone());
            write!(
                f,
                "TAKE_NON_FUNGIBLES_FROM_WORKTOP\n    Address(\"{}\")\n    Array<NonFungibleLocalId>({})\n    Bucket(\"{}\");",
                resource_address.display(context.bech32_encoder),
                ids.iter()
                    .map(|k| ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(k.clone())).to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", "),
                name
            )?;
        }
        Instruction::TakeAllFromWorktop { resource_address } => {
            let bucket_id = context
                .id_allocator
                .new_bucket_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            write!(
                f,
                "TAKE_ALL_FROM_WORKTOP\n    Address(\"{}\")\n    Bucket(\"{}\");",
                resource_address.display(context.bech32_encoder),
                name
            )?;
            context.bucket_names.insert(bucket_id, name);
        }
        Instruction::ReturnToWorktop { bucket_id } => {
            write!(
                f,
                "RETURN_TO_WORKTOP\n    Bucket({});",
                context
                    .bucket_names
                    .get(bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id.0))
            )?;
        }
        Instruction::AssertWorktopContains {
            amount,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS\n    Address(\"{}\")\n    Decimal(\"{}\");",
                resource_address.display(context.bech32_encoder),
                amount
            )?;
        }
        Instruction::AssertWorktopContainsNonFungibles {
            ids,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES\n    Address(\"{}\")\n    Array<NonFungibleLocalId>({});",
                resource_address.display(context.bech32_encoder),
                ids.iter()
                    .map(|k| ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(k.clone()))
                        .to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
        Instruction::PopFromAuthZone => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(f, "POP_FROM_AUTH_ZONE\n    Proof(\"{}\");", name)?;
        }
        Instruction::PushToAuthZone { proof_id } => {
            write!(
                f,
                "PUSH_TO_AUTH_ZONE\n    Proof({});",
                context
                    .proof_names
                    .get(proof_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", proof_id.0))
            )?;
        }
        Instruction::ClearAuthZone => {
            f.write_str("CLEAR_AUTH_ZONE;")?;
        }
        Instruction::CreateProofFromAuthZone { resource_address } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE\n    Address(\"{}\")\n    Proof(\"{}\");",
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        Instruction::CreateProofFromAuthZoneOfAmount {
            amount,
            resource_address,
        } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT\n    Address(\"{}\")\n    Decimal(\"{}\")\n    Proof(\"{}\");",
                resource_address.display(context.bech32_encoder),
                amount,
                name
            )?;
        }
        Instruction::CreateProofFromAuthZoneOfNonFungibles {
            ids,
            resource_address,
        } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES\n    Address(\"{}\")\n    Array<NonFungibleLocalId>({})\n    Proof(\"{}\");",
                resource_address.display(context.bech32_encoder),ids.iter()
                .map(|k| ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(k.clone())).to_string(context.for_value_display()))
                .collect::<Vec<String>>()
                .join(", "),
                name
            )?;
        }
        Instruction::CreateProofFromAuthZoneOfAll { resource_address } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL\n    Address(\"{}\")\n    Proof(\"{}\");",
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }

        Instruction::ClearSignatureProofs => {
            f.write_str("CLEAR_SIGNATURE_PROOFS;")?;
        }

        Instruction::CreateProofFromBucket { bucket_id } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_BUCKET\n    Bucket({})\n    Proof(\"{}\");",
                context
                    .bucket_names
                    .get(bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id.0)),
                name
            )?;
        }

        Instruction::CreateProofFromBucketOfAmount { bucket_id, amount } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_BUCKET_OF_AMOUNT\n    Bucket({})\n    Decimal(\"{}\")\n    Proof(\"{}\");",
                context
                    .bucket_names
                    .get(bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id.0)), 
                amount,
                name
            )?;
        }
        Instruction::CreateProofFromBucketOfNonFungibles { bucket_id, ids } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES\n    Bucket({})\n    Array<NonFungibleLocalId>({})\n    Proof(\"{}\");",
                context
                    .bucket_names
                    .get(bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id.0)),
                ids.iter()
                .map(|k| ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(k.clone())).to_string(context.for_value_display()))
                .collect::<Vec<String>>()
                .join(", "), 
                name
            )?;
        }
        Instruction::CreateProofFromBucketOfAll { bucket_id } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_BUCKET_OF_ALL\n    Bucket({})\n    Proof(\"{}\");",
                context
                    .bucket_names
                    .get(bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id.0)),
                name
            )?;
        }
        Instruction::BurnResource { bucket_id } => {
            write!(
                f,
                "BURN_RESOURCE\n    Bucket({});",
                context
                    .bucket_names
                    .get(bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id.0)),
            )?;
        }
        Instruction::CloneProof { proof_id } => {
            let proof_id2 = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id2, name.clone());
            write!(
                f,
                "CLONE_PROOF\n    Proof({})\n    Proof(\"{}\");",
                context
                    .proof_names
                    .get(proof_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", proof_id.0)),
                name
            )?;
        }
        Instruction::DropProof { proof_id } => {
            write!(
                f,
                "DROP_PROOF\n    Proof({});",
                context
                    .proof_names
                    .get(proof_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", proof_id.0)),
            )?;
        }
        Instruction::CallFunction {
            package_address,
            blueprint_name,
            function_name,
            args,
        } => {
            match (
                package_address,
                blueprint_name.as_str(),
                function_name.as_str(),
            ) {
                (&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_IDENT) => {
                    write!(f, "PUBLISH_PACKAGE")?;
                }
                (&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_ADVANCED_IDENT) => {
                    write!(f, "PUBLISH_PACKAGE_ADVANCED")?;
                }
                (&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_ADVANCED_IDENT) => {
                    write!(f, "CREATE_ACCOUNT_ADVANCED")?;
                }
                (&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_IDENT) => {
                    write!(f, "CREATE_ACCOUNT")?;
                }
                (&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT, IDENTITY_CREATE_ADVANCED_IDENT) => {
                    write!(f, "CREATE_IDENTITY_ADVANCED")?;
                }
                (&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT, IDENTITY_CREATE_IDENT) => {
                    write!(f, "CREATE_IDENTITY")?;
                }
                (
                    &ACCESS_CONTROLLER_PACKAGE,
                    ACCESS_CONTROLLER_BLUEPRINT,
                    ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
                ) => {
                    write!(f, "CREATE_ACCESS_CONTROLLER")?;
                }
                (
                    &RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => {
                    write!(f, "CREATE_FUNGIBLE_RESOURCE")?;
                }
                (
                    &RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) => {
                    write!(f, "CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY")?;
                }
                (
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => {
                    write!(f, "CREATE_NON_FUNGIBLE_RESOURCE")?;
                }
                (
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) => {
                    write!(f, "CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY")?;
                }
                _ => {
                    write!(
                        f,
                        "CALL_FUNCTION\n    Address(\"{}\")\n    \"{}\"\n    \"{}\"",
                        package_address.display(context.bech32_encoder),
                        blueprint_name,
                        function_name,
                    )?;
                }
            }

            format_encoded_args(f, context, args)?;
            f.write_str(";")?;
        }
        Instruction::CallMethod {
            address,
            method_name,
            args,
        } => {
            match (address, method_name.as_str()) {
                // Nb - For Main method call, we also check the address type to avoid name clashing.

                /* Package */
                (address, PACKAGE_SET_ROYALTY_CONFIG_IDENT)
                    if address.as_node_id().is_global_package() =>
                {
                    f.write_str(&format!(
                        "SET_PACKAGE_ROYALTY_CONFIG\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }
                (address, PACKAGE_CLAIM_ROYALTY_IDENT)
                    if address.as_node_id().is_global_package() =>
                {
                    f.write_str(&format!(
                        "CLAIM_PACKAGE_ROYALTY\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }

                /* Resource manager */
                (address, FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address.as_node_id().is_global_fungible_resource_manager() =>
                {
                    f.write_str(&format!(
                        "MINT_FUNGIBLE\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address
                        .as_node_id()
                        .is_global_non_fungible_resource_manager() =>
                {
                    f.write_str(&format!(
                        "MINT_NON_FUNGIBLE\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT)
                    if address
                        .as_node_id()
                        .is_global_non_fungible_resource_manager() =>
                {
                    f.write_str(&format!(
                        "MINT_UUID_NON_FUNGIBLE\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }

                /* Validator */
                (address, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT)
                    if address.as_node_id().is_global_epoch_manager() =>
                {
                    write!(f, "CREATE_VALIDATOR")?;
                }

                /* Default */
                _ => {
                    f.write_str(&format!(
                        "CALL_METHOD\n    Address(\"{}\")\n    \"{}\"",
                        address.display(context.bech32_encoder),
                        method_name
                    ))?;
                }
            }

            format_encoded_args(f, context, args)?;
            f.write_str(";")?;
        }
        Instruction::CallRoyaltyMethod {
            address,
            method_name,
            args,
        } => {
            match (address, method_name.as_str()) {
                /* Component royalty */
                (address, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT) => {
                    f.write_str(&format!(
                        "SET_COMPONENT_ROYALTY_CONFIG\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }
                (address, COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT) => {
                    f.write_str(&format!(
                        "CLAIM_COMPONENT_ROYALTY\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }

                /* Default */
                _ => {
                    f.write_str(&format!(
                        "CALL_ROYALTY_METHOD\n    Address(\"{}\")\n    \"{}\"",
                        address.display(context.bech32_encoder),
                        method_name
                    ))?;
                }
            }

            format_encoded_args(f, context, args)?;
            f.write_str(";")?;
        }
        Instruction::CallMetadataMethod {
            address,
            method_name,
            args,
        } => {
            match (address, method_name.as_str()) {
                /* Metadata */
                (address, METADATA_SET_IDENT) => {
                    f.write_str(&format!(
                        "SET_METADATA\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }
                (address, METADATA_REMOVE_IDENT) => {
                    f.write_str(&format!(
                        "REMOVE_METADATA\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }

                /* Default */
                _ => {
                    f.write_str(&format!(
                        "CALL_METADATA_METHOD\n    Address(\"{}\")\n    \"{}\"",
                        address.display(context.bech32_encoder),
                        method_name
                    ))?;
                }
            }

            format_encoded_args(f, context, args)?;
            f.write_str(";")?;
        }
        Instruction::CallAccessRulesMethod {
            address,
            method_name,
            args,
        } => {
            match (address, method_name.as_str()) {
                /* Access rules */
                (address, ACCESS_RULES_SET_AUTHORITY_RULE_IDENT) => {
                    f.write_str(&format!(
                        "SET_AUTHORITY_ACCESS_RULE\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }
                (address, ACCESS_RULES_SET_AUTHORITY_MUTABILITY_IDENT) => {
                    f.write_str(&format!(
                        "SET_AUTHORITY_MUTABILITY\n    Address(\"{}\")",
                        address.display(context.bech32_encoder),
                    ))?;
                }

                /* Default */
                _ => {
                    // TODO: add compiler support
                    f.write_str(&format!(
                        "CALL_ACCESS_RULES_METHOD\n    Address(\"{}\")\n    \"{}\"",
                        address.display(context.bech32_encoder),
                        method_name
                    ))?;
                }
            }

            format_encoded_args(f, context, args)?;
            f.write_str(";")?;
        }
        Instruction::RecallResource { vault_id, amount } => {
            f.write_str("RECALL_RESOURCE")?;
            format_typed_value(f, context, vault_id)?;
            format_typed_value(f, context, amount)?;
            f.write_str(";")?;
        }

        Instruction::DropAllProofs => {
            f.write_str("DROP_ALL_PROOFS;")?;
        }
    }

    Ok(())
}

pub fn format_typed_value<F: fmt::Write, T: ManifestEncode>(
    f: &mut F,
    context: &mut DecompilationContext,
    value: &T,
) -> Result<(), DecompileError> {
    f.write_str("\n    ")?;
    let value: ManifestValue = to_manifest_value(value);

    format_manifest_value(f, &value, &context.for_value_display())?;
    Ok(())
}

pub fn format_encoded_args<F: fmt::Write>(
    f: &mut F,
    context: &mut DecompilationContext,
    value: &ManifestValue,
) -> Result<(), DecompileError> {
    if let Value::Tuple { fields } = value {
        for field in fields {
            f.write_str("\n    ")?;
            format_manifest_value(f, &field, &context.for_value_display())?;
        }
    } else {
        return Err(DecompileError::InvalidArguments);
    }

    Ok(())
}
