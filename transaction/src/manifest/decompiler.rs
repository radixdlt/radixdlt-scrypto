use radix_engine_interface::address::{AddressError, Bech32Encoder};
use radix_engine_interface::api::types::{BucketId, GlobalAddress, ProofId};
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
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
    InvalidScryptoValue(ScryptoValueDecodeError),
    InvalidSborValue(EncodeError),
    IdAllocationError(IdAllocationError),
    FormattingError(fmt::Error),
}

impl From<ScryptoValueDecodeError> for DecompileError {
    fn from(error: ScryptoValueDecodeError) -> Self {
        Self::InvalidScryptoValue(error)
    }
}

impl From<EncodeError> for DecompileError {
    fn from(error: EncodeError) -> Self {
        Self::InvalidSborValue(error)
    }
}

impl From<fmt::Error> for DecompileError {
    fn from(error: fmt::Error) -> Self {
        Self::FormattingError(error)
    }
}

pub struct DecompilationContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
    pub id_allocator: IdAllocator,
    pub bucket_names: HashMap<BucketId, String>,
    pub proof_names: HashMap<ProofId, String>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(bech32_encoder: &'a Bech32Encoder) -> Self {
        Self {
            bech32_encoder: Some(bech32_encoder),
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            bucket_names: HashMap::<BucketId, String>::new(),
            proof_names: HashMap::<ProofId, String>::new(),
        }
    }

    pub fn new_with_optional_network(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            bucket_names: HashMap::<BucketId, String>::new(),
            proof_names: HashMap::<ProofId, String>::new(),
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
                "TAKE_FROM_WORKTOP ResourceAddress(\"{}\") Bucket(\"{}\");",
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
                "TAKE_FROM_WORKTOP_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Bucket(\"{}\");",
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
                "TAKE_FROM_WORKTOP_BY_IDS Array<NonFungibleId>({}) ResourceAddress(\"{}\") Bucket(\"{}\");",
                ids.iter()
                    .map(|k| ScryptoCustomValue::NonFungibleId(k.clone()).to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        BasicInstruction::ReturnToWorktop { bucket_id } => {
            write!(
                f,
                "RETURN_TO_WORKTOP Bucket({});",
                context
                    .bucket_names
                    .get(&bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id))
            )?;
        }
        BasicInstruction::AssertWorktopContains { resource_address } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS ResourceAddress(\"{}\");",
                resource_address.display(context.bech32_encoder)
            )?;
        }
        BasicInstruction::AssertWorktopContainsByAmount {
            amount,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\");",
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
                "ASSERT_WORKTOP_CONTAINS_BY_IDS Array<NonFungibleId>({}) ResourceAddress(\"{}\");",
                ids.iter()
                    .map(|k| ScryptoCustomValue::NonFungibleId(k.clone())
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
            write!(f, "POP_FROM_AUTH_ZONE Proof(\"{}\");", name)?;
        }
        BasicInstruction::PushToAuthZone { proof_id } => {
            write!(
                f,
                "PUSH_TO_AUTH_ZONE Proof({});",
                context
                    .proof_names
                    .get(&proof_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", proof_id))
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
                "CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"{}\") Proof(\"{}\");",
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Proof(\"{}\");",
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS Array<NonFungibleId>({}) ResourceAddress(\"{}\") Proof(\"{}\");",ids.iter()
                .map(|k| ScryptoCustomValue::NonFungibleId(k.clone()).to_string(context.for_value_display()))
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
                "CREATE_PROOF_FROM_BUCKET Bucket({}) Proof(\"{}\");",
                context
                    .bucket_names
                    .get(&bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id)),
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
                "CLONE_PROOF Proof({}) Proof(\"{}\");",
                context
                    .proof_names
                    .get(&proof_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", proof_id)),
                name
            )?;
        }
        BasicInstruction::DropProof { proof_id } => {
            write!(
                f,
                "DROP_PROOF Proof({});",
                context
                    .proof_names
                    .get(&proof_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", proof_id)),
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
                "CALL_FUNCTION PackageAddress(\"{}\") \"{}\" \"{}\"",
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
                "CALL_METHOD ComponentAddress(\"{}\") \"{}\"",
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
        BasicInstruction::BurnResource { bucket_id } => {
            write!(
                f,
                "BURN_RESOURCE Bucket({});",
                context
                    .bucket_names
                    .get(&bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id)),
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
            f.write_str("MINT_NON_FUNGIBLE")?;
            format_typed_value(f, context, resource_address)?;
            format_typed_value(f, context, entries)?;
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
        BasicInstruction::CreateNonFungibleResource {
            id_type,
            metadata,
            access_rules,
            initial_supply,
        } => {
            f.write_str("CREATE_NON_FUNGIBLE_RESOURCE")?;
            format_typed_value(f, context, id_type)?;
            format_typed_value(f, context, metadata)?;
            format_typed_value(f, context, access_rules)?;
            format_typed_value(f, context, initial_supply)?;
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
    let bytes = scrypto_encode(value).map_err(DecompileError::InvalidSborValue)?;
    let value =
        IndexedScryptoValue::from_slice(&bytes).map_err(DecompileError::InvalidScryptoValue)?;
    f.write_char(' ')?;
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
        GlobalAddress::System(address) => {
            write!(
                f,
                "SystemAddress(\"{}\")",
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
    if let SborValue::Tuple { fields } = value.dom {
        for field in fields {
            let bytes = scrypto_encode(&field)?;
            let arg = IndexedScryptoValue::from_slice(&bytes)
                .map_err(|_| DecompileError::InvalidArguments)?;
            f.write_char(' ')?;
            write!(f, "{}", &arg.display(context.for_value_display()))?;
        }
    } else {
        return Err(DecompileError::InvalidArguments);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::*;
    use radix_engine_interface::core::NetworkDefinition;

    #[test]
    fn test_resource_move() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/resource_move.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/test-cases/code.blob").to_vec(),
                include_bytes!("../../examples/test-cases/abi.blob").to_vec(),
            ],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "withdraw_by_amount" Decimal("5") ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
TAKE_FROM_WORKTOP_BY_AMOUNT Decimal("2") ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket1");
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "buy_gumball" Bucket("bucket1");
ASSERT_WORKTOP_CONTAINS_BY_AMOUNT Decimal("3") ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
ASSERT_WORKTOP_CONTAINS ResourceAddress("resource_sim1qzhdk7tq68u8msj38r6v6yqa5myc64ejx3ud20zlh9gseqtux6");
TAKE_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket2");
CREATE_PROOF_FROM_BUCKET Bucket("bucket2") Proof("proof1");
CLONE_PROOF Proof("proof1") Proof("proof2");
DROP_PROOF Proof("proof1");
DROP_PROOF Proof("proof2");
CALL_METHOD ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "create_proof_by_amount" Decimal("5") ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
POP_FROM_AUTH_ZONE Proof("proof3");
DROP_PROOF Proof("proof3");
RETURN_TO_WORKTOP Bucket("bucket2");
TAKE_FROM_WORKTOP_BY_IDS Array<NonFungibleId>(NonFungibleId(1u32)) ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket3");
DROP_ALL_PROOFS;
CALL_METHOD ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "deposit_batch" Expression("ENTIRE_WORKTOP");
"#
        );
    }

    #[test]
    fn test_resource_manipulate() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/resource_manipulate.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/test-cases/code.blob").to_vec(),
                include_bytes!("../../examples/test-cases/abi.blob").to_vec(),
            ],
        );

        assert_eq!(
            canonical_manifest,
            r#"CREATE_FUNGIBLE_RESOURCE 0u8 Array<Tuple>() Array<Tuple>() Some(Decimal("1"));
TAKE_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket1");
BURN_RESOURCE Bucket("bucket1");
MINT_FUNGIBLE ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Decimal("5");
RECALL_RESOURCE Bytes("49cd9235ba62b2c217e32e5b4754c08219ef16389761356eaccbf6f6bdbfa44d00000000") Decimal("1.2");
"#
        );
    }

    #[test]
    fn test_publish_package() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/publish_package.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/test-cases/code.blob").to_vec(),
                include_bytes!("../../examples/test-cases/abi.blob").to_vec(),
            ],
        );

        assert_eq!(
            canonical_manifest,
            r#"PUBLISH_PACKAGE Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d") Array<Tuple>() Array<Tuple>() Array<Tuple>(Tuple(Enum("SetMetadata"), Tuple(Enum("DenyAll"), Enum("DenyAll"))), Tuple(Enum("GetMetadata"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("SetRoyaltyConfig"), Tuple(Enum("DenyAll"), Enum("DenyAll"))), Tuple(Enum("ClaimRoyalty"), Tuple(Enum("DenyAll"), Enum("DenyAll"))));
"#
        );
    }

    #[test]
    fn test_invocation() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/invocation.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/test-cases/code.blob").to_vec(),
                include_bytes!("../../examples/test-cases/abi.blob").to_vec(),
            ],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_FUNCTION PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe") "BlueprintName" "f" "string";
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "complicated_method" Decimal("1") PreciseDecimal("2");
"#
        );
    }

    #[test]
    fn test_royalty() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/royalty.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/test-cases/code.blob").to_vec(),
                include_bytes!("../../examples/test-cases/abi.blob").to_vec(),
            ],
        );

        assert_eq!(
            canonical_manifest,
            r#"SET_PACKAGE_ROYALTY_CONFIG PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe") Array<Tuple>(Tuple("Blueprint", Tuple(Array<Tuple>(Tuple("method", 1u32)), 0u32)));
SET_COMPONENT_ROYALTY_CONFIG ComponentAddress("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt") Tuple(Array<Tuple>(Tuple("method", 1u32)), 0u32);
CLAIM_PACKAGE_ROYALTY PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe");
CLAIM_COMPONENT_ROYALTY ComponentAddress("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt");
"#
        );
    }

    #[test]
    fn test_metadata() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/metadata.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/test-cases/code.blob").to_vec(),
                include_bytes!("../../examples/test-cases/abi.blob").to_vec(),
            ],
        );

        assert_eq!(
            canonical_manifest,
            r#"SET_METADATA PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe") "k" "v";
SET_METADATA ComponentAddress("component_sim1qg2jwzl3hxnkqye8tfj5v3p2wp7cv9xdcjv4nl63refs785pvt") "k" "v";
SET_METADATA ResourceAddress("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v") "k" "v";
"#
        );
    }

    #[test]
    fn test_values() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/values.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/test-cases/code.blob").to_vec(),
                include_bytes!("../../examples/test-cases/abi.blob").to_vec(),
            ],
        );

        assert_eq!(
            canonical_manifest,
            r#"TAKE_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket1");
CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Proof("proof1");
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "with_aliases" None None Some("hello") Some("hello") Ok("test") Ok("test") Err("test123") Err("test123") Bytes("050aff") Bytes("050aff");
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "with_all_types" PackageAddress("package_sim1qyqzcexvnyg60z7lnlwauh66nhzg3m8tch2j8wc0e70qkydk8r") ComponentAddress("account_sim1q0u9gxewjxj8nhxuaschth2mgencma2hpkgwz30s9wlslthace") ResourceAddress("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v") SystemAddress("system_sim1qne8qu4seyvzfgd94p3z8rjcdl3v0nfhv84judpum2lq7x4635") Component("000000000000000000000000000000000000000000000000000000000000000005000000") KeyValueStore("000000000000000000000000000000000000000000000000000000000000000005000000") Bucket("bucket1") Proof("proof1") Vault("000000000000000000000000000000000000000000000000000000000000000005000000") Expression("ALL_WORKTOP_RESOURCES") Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", "value") NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", 123u32) NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", 456u64) NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", Bytes("031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f")) NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", 1234567890u128) Hash("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824") EcdsaSecp256k1PublicKey("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798") EcdsaSecp256k1Signature("0079224ea514206706298d8d620f660828f7987068d6d02757e6f3cbbf4a51ab133395db69db1bc9b2726dd99e34efc252d8258dcb003ebaba42be349f50f7765e") EddsaEd25519PublicKey("4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29") EddsaEd25519Signature("ce993adc51111309a041faa65cbcf1154d21ed0ecdc2d54070bc90b9deb744aa8605b3f686fa178fba21070b4a4678e54eee3486a881e0e328251cd37966de09") Decimal("1.2") PreciseDecimal("1.2") NonFungibleId(Bytes("031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f")) NonFungibleId(12u32) NonFungibleId(12345u64) NonFungibleId(1234567890u128) NonFungibleId("SomeId");
"#
        );
    }

    #[test]
    fn test_access_rule() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/test-cases/access_rule.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"SET_METHOD_ACCESS_RULE ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") 0u32 Enum("ScryptoMethod", "test") Enum("AllowAll");
"#
        );
    }

    #[test]
    fn test_create_fungible_resource_with_initial_supply() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/creation/fungible/with_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "lock_fee" Decimal("10");
CREATE_FUNGIBLE_RESOURCE 18u8 Array<Tuple>(Tuple("description", "A very innovative and important resource"), Tuple("name", "MyResource"), Tuple("symbol", "RSRC")) Array<Tuple>(Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) Some(Decimal("12"));
CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "deposit_batch" Expression("ENTIRE_WORKTOP");
"#
        );
    }

    #[test]
    fn test_create_fungible_resource_with_no_initial_supply() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/creation/fungible/no_initial_supply.rtm")
                    .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "lock_fee" Decimal("10");
CREATE_FUNGIBLE_RESOURCE 18u8 Array<Tuple>(Tuple("description", "A very innovative and important resource"), Tuple("name", "MyResource"), Tuple("symbol", "RSRC")) Array<Tuple>(Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) None;
"#
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_initial_supply() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/non_fungible/with_initial_supply.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "lock_fee" Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE Enum("U32") Array<Tuple>(Tuple("description", "A very innovative and important resource"), Tuple("name", "MyResource"), Tuple("symbol", "RSRC")) Array<Tuple>(Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) Some(Array<Tuple>(Tuple(NonFungibleId(1u32), Tuple(Bytes("5c2100"), Bytes("5c2100")))));
CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "deposit_batch" Expression("ENTIRE_WORKTOP");
"#
        );
    }

    #[test]
    fn test_create_non_fungible_resource_with_no_initial_supply() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!(
                    "../../examples/resources/creation/non_fungible/no_initial_supply.rtm"
                )
                .to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "lock_fee" Decimal("10");
CREATE_NON_FUNGIBLE_RESOURCE Enum("U32") Array<Tuple>(Tuple("description", "A very innovative and important resource"), Tuple("name", "MyResource"), Tuple("symbol", "RSRC")) Array<Tuple>(Tuple(Enum("Withdraw"), Tuple(Enum("AllowAll"), Enum("DenyAll"))), Tuple(Enum("Deposit"), Tuple(Enum("AllowAll"), Enum("DenyAll")))) None;
"#
        );
    }

    #[test]
    fn test_mint_fungible() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/mint/fungible/mint.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "lock_fee" Decimal("10");
CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "create_proof_by_amount" Decimal("1") ResourceAddress("resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02");
MINT_FUNGIBLE ResourceAddress("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx") Decimal("12");
CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "deposit_batch" Expression("ENTIRE_WORKTOP");
"#
        );
    }

    #[test]
    fn test_mint_non_fungible() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            &apply_replacements_to_manifest(
                include_str!("../../examples/resources/mint/non_fungible/mint.rtm").to_string(),
            ),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "lock_fee" Decimal("10");
CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "create_proof_by_amount" Decimal("1") ResourceAddress("resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02");
MINT_NON_FUNGIBLE ResourceAddress("resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx") Array<Tuple>(Tuple(NonFungibleId(12u32), Tuple(Bytes("5c2100"), Bytes("5c2100"))));
CALL_METHOD ComponentAddress("account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na") "deposit_batch" Expression("ENTIRE_WORKTOP");
"#
        );
    }

    #[test]
    fn test_recompile_many_blobs() {
        // This test is mostly to prevent a regression whereby the blobs were re-ordered at compilation
        // Which made the manifest compilation process non-deterministic (when including blobs)
        compile_and_decompile_with_inversion_test(
            "",
            &NetworkDefinition::simulator(),
            vec![
                vec![0],
                vec![1],
                vec![2],
                vec![3],
                vec![4],
                vec![5],
                vec![6],
                vec![7],
                vec![8],
                vec![9],
            ],
        );
    }

    fn compile_and_decompile_with_inversion_test(
        manifest: &str,
        network: &NetworkDefinition,
        blobs: Vec<Vec<u8>>,
    ) -> String {
        let compiled1 = compile(manifest, network, blobs.clone()).unwrap();
        let decompiled1 = decompile(&compiled1.instructions, network).unwrap();

        // Whilst we're here - let's test that compile/decompile are inverses...
        let compiled2 = compile(manifest, network, blobs).unwrap();
        let decompiled2 = decompile(&compiled2.instructions, network).unwrap();

        // The manifest argument is not necessarily in canonical decompiled string representation,
        // therefore we can't assert that decompiled1 == manifest ...
        // So instead we assert that decompiled1 and decompiled2 match :)
        assert_eq!(
            compiled1, compiled2,
            "Compile(Decompile(compiled_manifest)) != compiled_manifest"
        );
        assert_eq!(
            decompiled1, decompiled2,
            "Decompile(Compile(canonical_manifest_str)) != canonical_manifest_str"
        );

        return decompiled2;
    }

    fn apply_replacements_to_manifest(mut manifest: String) -> String {
        let replacement_vectors = BTreeMap::from([
            (
                "{xrd_resource_address}",
                "resource_sim1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95sqjjpwr",
            ),
            (
                "{account_component_address}",
                "account_sim1qwskd4q5jdywfw6f7jlwmcyp2xxq48uuwruc003x2kcskxh3na",
            ),
            (
                "{other_account_component_address}",
                "account_sim1qdy4jqfpehf8nv4n7680cw0vhxqvhgh5lf3ae8jkjz6q5hmzed",
            ),
            (
                "{minter_badge_resource_address}",
                "resource_sim1qp075qmn6389pkq30ppzzsuadd55ry04mjx69v86r4wq0feh02",
            ),
            (
                "{mintable_resource_address}",
                "resource_sim1qqgvpz8q7ypeueqcv4qthsv7ezt8h9m3depmqqw7pc4sfmucfx",
            ),
            ("{initial_supply}", "12"),
            ("{mint_amount}", "12"),
            ("{non_fungible_id}", "12u32"),
        ]);
        for (of, with) in replacement_vectors.into_iter() {
            manifest = manifest.replace(of, with);
        }
        manifest
    }
}
