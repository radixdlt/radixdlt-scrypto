use crate::data::*;
use crate::errors::*;
use crate::model::*;
use crate::validation::*;
use radix_engine_interface::address::Bech32Encoder;
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
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
};
use radix_engine_interface::constants::{
    ACCESS_CONTROLLER_PACKAGE, ACCOUNT_PACKAGE, EPOCH_MANAGER, IDENTITY_PACKAGE,
    RESOURCE_MANAGER_PACKAGE,
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

    pub fn for_value_display(&'a self) -> ManifestValueDisplayContext<'a> {
        ManifestValueDisplayContext::with_bech32_and_names(
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
        Instruction::TakeFromWorktop { resource_address } => {
            let bucket_id = context
                .id_allocator
                .new_bucket_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            write!(
                f,
                "TAKE_FROM_WORKTOP\n    Address(\"{}\")\n    Bucket(\"{}\");",
                resource_address.display(context.bech32_encoder),
                name
            )?;
            context.bucket_names.insert(bucket_id, name);
        }
        Instruction::TakeFromWorktopByAmount {
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
                "TAKE_FROM_WORKTOP_BY_AMOUNT\n    Decimal(\"{}\")\n    Address(\"{}\")\n    Bucket(\"{}\");",
                amount,
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        Instruction::TakeFromWorktopByIds {
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
                "TAKE_FROM_WORKTOP_BY_IDS\n    Array<NonFungibleLocalId>({})\n    Address(\"{}\")\n    Bucket(\"{}\");",
                ids.iter()
                    .map(|k| ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(k.clone())).to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.display(context.bech32_encoder),
                name
            )?;
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
        Instruction::AssertWorktopContains { resource_address } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS\n    Address(\"{}\");",
                resource_address.display(context.bech32_encoder)
            )?;
        }
        Instruction::AssertWorktopContainsByAmount {
            amount,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT\n    Decimal(\"{}\")\n    Address(\"{}\");",
                amount,
                resource_address.display(context.bech32_encoder)
            )?;
        }
        Instruction::AssertWorktopContainsByIds {
            ids,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS_BY_IDS\n    Array<NonFungibleLocalId>({})\n    Address(\"{}\");",
                ids.iter()
                    .map(|k| ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(k.clone()))
                        .to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.display(context.bech32_encoder)
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
        Instruction::CreateProofFromAuthZoneByAmount {
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT\n    Decimal(\"{}\")\n    Address(\"{}\")\n    Proof(\"{}\");",
                amount,
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        Instruction::CreateProofFromAuthZoneByIds {
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS\n    Array<NonFungibleLocalId>({})\n    Address(\"{}\")\n    Proof(\"{}\");",ids.iter()
                .map(|k| ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(k.clone())).to_string(context.for_value_display()))
                .collect::<Vec<String>>()
                .join(", "),
                resource_address.display(context.bech32_encoder),
                name
            )?;
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
        Instruction::DropAllProofs => {
            f.write_str("DROP_ALL_PROOFS;")?;
        }
        Instruction::ClearSignatureProofs => {
            f.write_str("CLEAR_SIGNATURE_PROOFS;")?;
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
                    &RESOURCE_MANAGER_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => {
                    write!(f, "CREATE_FUNGIBLE_RESOURCE")?;
                }
                (
                    &RESOURCE_MANAGER_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) => {
                    write!(f, "CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY")?;
                }
                (
                    &RESOURCE_MANAGER_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => {
                    write!(f, "CREATE_NON_FUNGIBLE_RESOURCE")?;
                }
                (
                    &RESOURCE_MANAGER_PACKAGE,
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
            component_address,
            method_name,
            args,
        } => {
            match (component_address, method_name.as_str()) {
                (&EPOCH_MANAGER, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT) => {
                    write!(f, "CREATE_VALIDATOR")?;
                }
                _ => {
                    f.write_str(&format!(
                        "CALL_METHOD\n    Address(\"{}\")\n    \"{}\"",
                        component_address.display(context.bech32_encoder),
                        method_name
                    ))?;
                }
            }

            format_encoded_args(f, context, args)?;
            f.write_str(";")?;
        }
        Instruction::PublishPackage {
            code,
            schema,
            royalty_config,
            metadata,
        } => {
            f.write_str("PUBLISH_PACKAGE")?;
            format_typed_value(f, context, code)?;
            format_typed_value(f, context, schema)?;
            format_typed_value(f, context, royalty_config)?;
            format_typed_value(f, context, metadata)?;
            f.write_str(";")?;
        }
        Instruction::PublishPackageAdvanced {
            code,
            schema,
            royalty_config,
            metadata,
            access_rules,
        } => {
            f.write_str("PUBLISH_PACKAGE_ADVANCED")?;
            format_typed_value(f, context, code)?;
            format_typed_value(f, context, schema)?;
            format_typed_value(f, context, royalty_config)?;
            format_typed_value(f, context, metadata)?;
            format_typed_value(f, context, access_rules)?;
            f.write_str(";")?;
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
        Instruction::RecallResource { vault_id, amount } => {
            f.write_str("RECALL_RESOURCE")?;
            format_typed_value(f, context, vault_id)?;
            format_typed_value(f, context, amount)?;
            f.write_str(";")?;
        }
        Instruction::SetMetadata {
            entity_address,
            key,
            value,
        } => {
            f.write_str("SET_METADATA")?;
            format_typed_value(f, context, entity_address)?;
            format_typed_value(f, context, key)?;
            format_typed_value(f, context, value)?;
            f.write_str(";")?;
        }
        Instruction::RemoveMetadata {
            entity_address,
            key,
        } => {
            f.write_str("REMOVE_METADATA")?;
            format_typed_value(f, context, entity_address)?;
            format_typed_value(f, context, key)?;
            f.write_str(";")?;
        }
        Instruction::SetPackageRoyaltyConfig {
            package_address,
            royalty_config,
        } => {
            f.write_str("SET_PACKAGE_ROYALTY_CONFIG")?;
            format_typed_value(f, context, package_address)?;
            format_typed_value(f, context, royalty_config)?;
            f.write_str(";")?;
        }
        Instruction::SetComponentRoyaltyConfig {
            component_address,
            royalty_config,
        } => {
            f.write_str("SET_COMPONENT_ROYALTY_CONFIG")?;
            format_typed_value(f, context, component_address)?;
            format_typed_value(f, context, royalty_config)?;
            f.write_str(";")?;
        }
        Instruction::ClaimPackageRoyalty { package_address } => {
            f.write_str("CLAIM_PACKAGE_ROYALTY")?;
            format_typed_value(f, context, package_address)?;
            f.write_str(";")?;
        }
        Instruction::ClaimComponentRoyalty { component_address } => {
            f.write_str("CLAIM_COMPONENT_ROYALTY")?;
            format_typed_value(f, context, component_address)?;
            f.write_str(";")?;
        }
        Instruction::SetMethodAccessRule {
            entity_address,
            key,
            rule,
        } => {
            f.write_str("SET_METHOD_ACCESS_RULE")?;
            format_typed_value(f, context, entity_address)?;
            format_typed_value(f, context, key)?;
            format_typed_value(f, context, rule)?;
            f.write_str(";")?;
        }
        Instruction::MintFungible {
            resource_address,
            amount,
        } => {
            f.write_str("MINT_FUNGIBLE")?;
            format_typed_value(f, context, resource_address)?;
            format_typed_value(f, context, amount)?;
            f.write_str(";")?;
        }
        Instruction::MintNonFungible {
            resource_address,
            args,
        } => {
            f.write_str("MINT_NON_FUNGIBLE")?;
            format_typed_value(f, context, resource_address)?;
            f.write_str("\n    ")?;
            format_manifest_value(f, args, &context.for_value_display())?;
            f.write_str(";")?;
        }
        Instruction::MintUuidNonFungible {
            resource_address,
            args,
        } => {
            f.write_str("MINT_UUID_NON_FUNGIBLE")?;
            format_typed_value(f, context, resource_address)?;
            f.write_str("\n    ")?;
            format_manifest_value(f, args, &context.for_value_display())?;
            f.write_str(";")?;
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
