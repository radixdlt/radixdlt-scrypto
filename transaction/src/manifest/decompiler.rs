use radix_engine_interface::address::{AddressError, Bech32Encoder};
use radix_engine_interface::api::types::GlobalAddress;
use radix_engine_interface::data::types::{ManifestBucket, ManifestProof};
use radix_engine_interface::data::*;
use radix_engine_interface::model::NonFungibleLocalId;
use radix_engine_interface::node::NetworkDefinition;
use sbor::rust::collections::*;
use sbor::rust::fmt;
use sbor::*;
use utils::ContextualDisplay;

use crate::errors::*;
use crate::model::*;
use crate::validation::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    InvalidAddress(AddressError),
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
    pub bucket_names: HashMap<ManifestBucket, String>,
    pub proof_names: HashMap<ManifestProof, String>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(bech32_encoder: &'a Bech32Encoder) -> Self {
        Self {
            bech32_encoder: Some(bech32_encoder),
            id_allocator: ManifestIdAllocator::new(),
            bucket_names: HashMap::<ManifestBucket, String>::new(),
            proof_names: HashMap::<ManifestProof, String>::new(),
        }
    }

    pub fn new_with_optional_network(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            id_allocator: ManifestIdAllocator::new(),
            bucket_names: HashMap::<ManifestBucket, String>::new(),
            proof_names: HashMap::<ManifestProof, String>::new(),
        }
    }

    pub fn for_value_display(&'a self) -> ValueFormattingContext<'a> {
        ValueFormattingContext::with_manifest_context(
            self.bech32_encoder,
            &self.bucket_names,
            &self.proof_names,
        )
    }
}

/// Contract: if the instructions are from a validated notarized transaction, no error
/// should be returned.
pub fn decompile(
    instructions: &[BasicInstruction],
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
    instruction: &BasicInstruction,
    context: &mut DecompilationContext,
) -> Result<(), DecompileError> {
    match instruction {
        BasicInstruction::TakeFromWorktop { resource_address } => {
            let bucket_id = context
                .id_allocator
                .new_bucket_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            write!(
                f,
                "TAKE_FROM_WORKTOP\n    ResourceAddress(\"{}\")\n    Bucket(\"{}\");",
                resource_address.display(context.bech32_encoder),
                name
            )?;
            context.bucket_names.insert(bucket_id, name);
        }
        BasicInstruction::TakeFromWorktopByAmount {
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
                "TAKE_FROM_WORKTOP_BY_AMOUNT\n    Decimal(\"{}\")\n    ResourceAddress(\"{}\")\n    Bucket(\"{}\");",
                amount,
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        BasicInstruction::TakeFromWorktopByIds {
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
                "TAKE_FROM_WORKTOP_BY_IDS\n    Array<NonFungibleLocalId>({})\n    ResourceAddress(\"{}\")\n    Bucket(\"{}\");",
                ids.iter()
                    .map(|k| ScryptoCustomValue::NonFungibleLocalId(k.clone()).to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        BasicInstruction::ReturnToWorktop { bucket_id } => {
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
        BasicInstruction::AssertWorktopContains { resource_address } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS\n    ResourceAddress(\"{}\");",
                resource_address.display(context.bech32_encoder)
            )?;
        }
        BasicInstruction::AssertWorktopContainsByAmount {
            amount,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT\n    Decimal(\"{}\")\n    ResourceAddress(\"{}\");",
                amount,
                resource_address.display(context.bech32_encoder)
            )?;
        }
        BasicInstruction::AssertWorktopContainsByIds {
            ids,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS_BY_IDS\n    Array<NonFungibleLocalId>({})\n    ResourceAddress(\"{}\");",
                ids.iter()
                    .map(|k| ScryptoCustomValue::NonFungibleLocalId(k.clone())
                        .to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.display(context.bech32_encoder)
            )?;
        }
        BasicInstruction::PopFromAuthZone => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(f, "POP_FROM_AUTH_ZONE\n    Proof(\"{}\");", name)?;
        }
        BasicInstruction::PushToAuthZone { proof_id } => {
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
        BasicInstruction::ClearAuthZone => {
            f.write_str("CLEAR_AUTH_ZONE;")?;
        }
        BasicInstruction::CreateProofFromAuthZone { resource_address } => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE\n    ResourceAddress(\"{}\")\n    Proof(\"{}\");",
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        BasicInstruction::CreateProofFromAuthZoneByAmount {
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT\n    Decimal(\"{}\")\n    ResourceAddress(\"{}\")\n    Proof(\"{}\");",
                amount,
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        BasicInstruction::CreateProofFromAuthZoneByIds {
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS\n    Array<NonFungibleLocalId>({})\n    ResourceAddress(\"{}\")\n    Proof(\"{}\");",ids.iter()
                .map(|k| ScryptoCustomValue::NonFungibleLocalId(k.clone()).to_string(context.for_value_display()))
                .collect::<Vec<String>>()
                .join(", "),
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        BasicInstruction::CreateProofFromBucket { bucket_id } => {
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
        BasicInstruction::CloneProof { proof_id } => {
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
        BasicInstruction::DropProof { proof_id } => {
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
        BasicInstruction::DropAllProofs => {
            f.write_str("DROP_ALL_PROOFS;")?;
        }
        BasicInstruction::CallFunction {
            package_address,
            blueprint_name,
            function_name,
            args,
        } => {
            write!(
                f,
                "CALL_FUNCTION\n    PackageAddress(\"{}\")\n    \"{}\"\n    \"{}\"",
                package_address.display(context.bech32_encoder),
                blueprint_name,
                function_name,
            )?;
            format_args(f, context, args)?;
            f.write_str(";")?;
        }
        BasicInstruction::CallMethod {
            component_address,
            method_name,
            args,
        } => {
            f.write_str(&format!(
                "CALL_METHOD\n    ComponentAddress(\"{}\")\n    \"{}\"",
                component_address.display(context.bech32_encoder),
                method_name
            ))?;
            format_args(f, context, args)?;
            f.write_str(";")?;
        }
        BasicInstruction::PublishPackage {
            code,
            abi,
            royalty_config,
            metadata,
            access_rules,
        } => {
            f.write_str("PUBLISH_PACKAGE")?;
            format_typed_value(f, context, code)?;
            format_typed_value(f, context, abi)?;
            format_typed_value(f, context, royalty_config)?;
            format_typed_value(f, context, metadata)?;
            format_typed_value(f, context, access_rules)?;
            f.write_str(";")?;
        }
        BasicInstruction::PublishPackageWithOwner {
            code,
            abi,
            owner_badge,
        } => {
            f.write_str("PUBLISH_PACKAGE_WITH_OWNER")?;
            format_typed_value(f, context, code)?;
            format_typed_value(f, context, abi)?;
            format_typed_value(f, context, owner_badge)?;
            f.write_str(";")?;
        }
        BasicInstruction::BurnResource { bucket_id } => {
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
        BasicInstruction::RecallResource { vault_id, amount } => {
            f.write_str("RECALL_RESOURCE")?;
            format_typed_value(f, context, vault_id)?;
            format_typed_value(f, context, amount)?;
            f.write_str(";")?;
        }
        BasicInstruction::SetMetadata {
            entity_address,
            key,
            value,
        } => {
            f.write_str("SET_METADATA")?;
            format_entity_address(f, context, entity_address)?;
            format_typed_value(f, context, key)?;
            format_typed_value(f, context, value)?;
            f.write_str(";")?;
        }
        BasicInstruction::SetPackageRoyaltyConfig {
            package_address,
            royalty_config,
        } => {
            f.write_str("SET_PACKAGE_ROYALTY_CONFIG")?;
            format_typed_value(f, context, package_address)?;
            format_typed_value(f, context, royalty_config)?;
            f.write_str(";")?;
        }
        BasicInstruction::SetComponentRoyaltyConfig {
            component_address,
            royalty_config,
        } => {
            f.write_str("SET_COMPONENT_ROYALTY_CONFIG")?;
            format_typed_value(f, context, component_address)?;
            format_typed_value(f, context, royalty_config)?;
            f.write_str(";")?;
        }
        BasicInstruction::ClaimPackageRoyalty { package_address } => {
            f.write_str("CLAIM_PACKAGE_ROYALTY")?;
            format_typed_value(f, context, package_address)?;
            f.write_str(";")?;
        }
        BasicInstruction::ClaimComponentRoyalty { component_address } => {
            f.write_str("CLAIM_COMPONENT_ROYALTY")?;
            format_typed_value(f, context, component_address)?;
            f.write_str(";")?;
        }
        BasicInstruction::SetMethodAccessRule {
            entity_address,
            index,
            key,
            rule,
        } => {
            f.write_str("SET_METHOD_ACCESS_RULE")?;
            format_entity_address(f, context, entity_address)?;
            format_typed_value(f, context, index)?;
            format_typed_value(f, context, key)?;
            format_typed_value(f, context, rule)?;
            f.write_str(";")?;
        }
        BasicInstruction::MintFungible {
            resource_address,
            amount,
        } => {
            f.write_str("MINT_FUNGIBLE")?;
            format_typed_value(f, context, resource_address)?;
            format_typed_value(f, context, amount)?;
            f.write_str(";")?;
        }
        BasicInstruction::MintNonFungible {
            resource_address,
            entries,
        } => {
            let entries = transform_non_fungible_mint_params(entries)?;

            f.write_str("MINT_NON_FUNGIBLE")?;
            format_typed_value(f, context, resource_address)?;
            format_typed_value(f, context, &entries)?;
            f.write_str(";")?;
        }
        BasicInstruction::MintUuidNonFungible {
            resource_address,
            entries,
        } => {
            let entries = transform_uuid_non_fungible_mint_params(entries)?;

            f.write_str("MINT_UUID_NON_FUNGIBLE")?;
            format_typed_value(f, context, resource_address)?;
            format_typed_value(f, context, &entries)?;
            f.write_str(";")?;
        }
        BasicInstruction::CreateFungibleResource {
            divisibility,
            metadata,
            access_rules,
            initial_supply,
        } => {
            f.write_str("CREATE_FUNGIBLE_RESOURCE")?;
            format_typed_value(f, context, divisibility)?;
            format_typed_value(f, context, metadata)?;
            format_typed_value(f, context, access_rules)?;
            format_typed_value(f, context, initial_supply)?;
            f.write_str(";")?;
        }
        BasicInstruction::CreateFungibleResourceWithOwner {
            divisibility,
            metadata,
            owner_badge,
            initial_supply,
        } => {
            f.write_str("CREATE_FUNGIBLE_RESOURCE_WITH_OWNER")?;
            format_typed_value(f, context, divisibility)?;
            format_typed_value(f, context, metadata)?;
            format_typed_value(f, context, owner_badge)?;
            format_typed_value(f, context, initial_supply)?;
            f.write_str(";")?;
        }
        BasicInstruction::CreateNonFungibleResource {
            id_type,
            metadata,
            access_rules,
            initial_supply,
        } => {
            let initial_supply = {
                match initial_supply {
                    Some(initial_supply) => {
                        transform_non_fungible_mint_params(initial_supply).map(Some)?
                    }
                    None => None,
                }
            };

            f.write_str("CREATE_NON_FUNGIBLE_RESOURCE")?;
            format_typed_value(f, context, id_type)?;
            format_typed_value(f, context, metadata)?;
            format_typed_value(f, context, access_rules)?;
            format_typed_value(f, context, &initial_supply)?;
            f.write_str(";")?;
        }
        BasicInstruction::CreateNonFungibleResourceWithOwner {
            id_type,
            metadata,
            owner_badge,
            initial_supply,
        } => {
            let initial_supply = {
                match initial_supply {
                    Some(initial_supply) => {
                        transform_non_fungible_mint_params(initial_supply).map(Some)?
                    }
                    None => None,
                }
            };

            f.write_str("CREATE_NON_FUNGIBLE_RESOURCE_WITH_OWNER")?;
            format_typed_value(f, context, id_type)?;
            format_typed_value(f, context, metadata)?;
            format_typed_value(f, context, owner_badge)?;
            format_typed_value(f, context, &initial_supply)?;
            f.write_str(";")?;
        }
        BasicInstruction::CreateValidator {
            key,
            owner_access_rule,
        } => {
            f.write_str("CREATE_VALIDATOR")?;
            format_typed_value(f, context, key)?;
            format_typed_value(f, context, owner_access_rule)?;
            f.write_str(";")?;
        }
        BasicInstruction::CreateAccessController {
            controlled_asset,
            primary_role,
            recovery_role,
            confirmation_role,
            timed_recovery_delay_in_minutes,
        } => {
            f.write_str("CREATE_ACCESS_CONTROLLER")?;
            format_typed_value(f, context, controlled_asset)?;
            format_typed_value(f, context, primary_role)?;
            format_typed_value(f, context, recovery_role)?;
            format_typed_value(f, context, confirmation_role)?;
            format_typed_value(f, context, timed_recovery_delay_in_minutes)?;
            f.write_str(";")?;
        }
        BasicInstruction::CreateIdentity { access_rule } => {
            f.write_str("CREATE_IDENTITY")?;
            format_typed_value(f, context, access_rule)?;
            f.write_str(";")?;
        }
        BasicInstruction::AssertAccessRule { access_rule } => {
            f.write_str("ASSERT_ACCESS_RULE")?;
            format_typed_value(f, context, access_rule)?;
            f.write_str(";")?;
        }
    }
    Ok(())
}

pub fn format_typed_value<F: fmt::Write, T: ScryptoEncode>(
    f: &mut F,
    context: &mut DecompilationContext,
    value: &T,
) -> Result<(), DecompileError> {
    let value = IndexedScryptoValue::from_typed(value);
    f.write_str("\n    ")?;
    write!(f, "{}", &value.display(context.for_value_display()))?;
    Ok(())
}

pub fn format_entity_address<F: fmt::Write>(
    f: &mut F,
    context: &mut DecompilationContext,
    address: &GlobalAddress,
) -> Result<(), DecompileError> {
    f.write_char(' ')?;
    match address {
        GlobalAddress::Component(address) => {
            write!(
                f,
                "ComponentAddress(\"{}\")",
                &address.display(context.bech32_encoder)
            )?;
        }
        GlobalAddress::Package(address) => {
            write!(
                f,
                "PackageAddress(\"{}\")",
                &address.display(context.bech32_encoder)
            )?;
        }
        GlobalAddress::Resource(address) => {
            write!(
                f,
                "ResourceAddress(\"{}\")",
                &address.display(context.bech32_encoder)
            )?;
        }
    }

    Ok(())
}

pub fn format_args<F: fmt::Write>(
    f: &mut F,
    context: &mut DecompilationContext,
    args: &Vec<u8>,
) -> Result<(), DecompileError> {
    let value =
        IndexedScryptoValue::from_slice(&args).map_err(|_| DecompileError::InvalidArguments)?;
    if let Value::Tuple { fields } = value.as_value() {
        for field in fields {
            let bytes = scrypto_encode(&field)?;
            let arg = IndexedScryptoValue::from_slice(&bytes)
                .map_err(|_| DecompileError::InvalidArguments)?;
            f.write_str("\n    ")?;
            write!(f, "{}", &arg.display(context.for_value_display()))?;
        }
    } else {
        return Err(DecompileError::InvalidArguments);
    }

    Ok(())
}

fn transform_non_fungible_mint_params(
    mint_params: &BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
) -> Result<BTreeMap<NonFungibleLocalId, (ScryptoValue, ScryptoValue)>, DecodeError> {
    let mut mint_params_scrypto_value =
        BTreeMap::<NonFungibleLocalId, (ScryptoValue, ScryptoValue)>::new();
    for (id, (immutable_data, mutable_data)) in mint_params.into_iter() {
        mint_params_scrypto_value.insert(
            id.clone(),
            (
                scrypto_decode(&immutable_data)?,
                scrypto_decode(&mutable_data)?,
            ),
        );
    }
    Ok(mint_params_scrypto_value)
}

fn transform_uuid_non_fungible_mint_params(
    mint_params: &Vec<(Vec<u8>, Vec<u8>)>,
) -> Result<Vec<(ScryptoValue, ScryptoValue)>, DecodeError> {
    let mut mint_params_scrypto_value = Vec::<(ScryptoValue, ScryptoValue)>::new();
    for (immutable_data, mutable_data) in mint_params.into_iter() {
        mint_params_scrypto_value.push((
            scrypto_decode(&immutable_data)?,
            scrypto_decode(&mutable_data)?,
        ));
    }
    Ok(mint_params_scrypto_value)
}
