use radix_engine_interface::address::{AddressError, Bech32Encoder};
use radix_engine_interface::api::types::{
    BucketId, GlobalAddress, NativeFunctionIdent, NativeMethodIdent, ProofId, RENodeId,
    ScryptoFunctionIdent, ScryptoMethodIdent, ScryptoPackage, ScryptoReceiver,
};
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, IndexedScryptoValue, ScryptoCustomValue,
    ScryptoValueDecodeError, ValueFormattingContext,
};
use radix_engine_interface::model::*;
use sbor::rust::collections::*;
use sbor::rust::fmt;
use sbor::{EncodeError, SborValue};
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
                "TAKE_FROM_WORKTOP ResourceAddress(\"{}\") Bucket(\"{}\");",
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
                "TAKE_FROM_WORKTOP_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Bucket(\"{}\");",
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
                "TAKE_FROM_WORKTOP_BY_IDS Array<NonFungibleId>({}) ResourceAddress(\"{}\") Bucket(\"{}\");",
                ids.iter()
                    .map(|k| ScryptoCustomValue::NonFungibleId(k.clone()).to_string(context.for_value_display()))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.display(context.bech32_encoder),
                name
            )?;
        }
        Instruction::ReturnToWorktop { bucket_id } => {
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
        Instruction::AssertWorktopContains { resource_address } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS ResourceAddress(\"{}\");",
                resource_address.display(context.bech32_encoder)
            )?;
        }
        Instruction::AssertWorktopContainsByAmount {
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
        Instruction::AssertWorktopContainsByIds {
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
        Instruction::PopFromAuthZone => {
            let proof_id = context
                .id_allocator
                .new_proof_id()
                .map_err(DecompileError::IdAllocationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(f, "POP_FROM_AUTH_ZONE Proof(\"{}\");", name)?;
        }
        Instruction::PushToAuthZone { proof_id } => {
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
                "CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"{}\") Proof(\"{}\");",
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Proof(\"{}\");",
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
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS Array<NonFungibleId>({}) ResourceAddress(\"{}\") Proof(\"{}\");",ids.iter()
                .map(|k| ScryptoCustomValue::NonFungibleId(k.clone()).to_string(context.for_value_display()))
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
                "CREATE_PROOF_FROM_BUCKET Bucket({}) Proof(\"{}\");",
                context
                    .bucket_names
                    .get(&bucket_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", bucket_id)),
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
                "CLONE_PROOF Proof({}) Proof(\"{}\");",
                context
                    .proof_names
                    .get(&proof_id)
                    .map(|name| format!("\"{}\"", name))
                    .unwrap_or(format!("{}u32", proof_id)),
                name
            )?;
        }
        Instruction::DropProof { proof_id } => {
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
        Instruction::DropAllProofs => {
            f.write_str("DROP_ALL_PROOFS;")?;
        }
        Instruction::CallFunction {
            function_ident,
            args,
        } => decompile_call_function(f, context, function_ident, args)?,
        Instruction::CallMethod { method_ident, args } => {
            decompile_call_scrypto_method(f, context, method_ident, args)?
        }
        Instruction::CallNativeFunction {
            function_ident,
            args,
        } => decompile_call_native_function(f, context, function_ident, args)?,
        Instruction::CallNativeMethod { method_ident, args } => {
            decompile_call_native_method(f, context, method_ident, args)?
        }
        Instruction::PublishPackageWithOwner {
            code,
            abi,
            owner_badge,
        } => {
            write!(
                f,
                "PUBLISH_PACKAGE_WITH_OWNER Blob(\"{}\") Blob(\"{}\") {};",
                code,
                abi,
                ScryptoCustomValue::NonFungibleAddress(owner_badge.clone())
                    .display(context.for_value_display()),
            )?;
        }
    }
    Ok(())
}

pub fn decompile_call_function<F: fmt::Write>(
    f: &mut F,
    context: &mut DecompilationContext,
    function_ident: &ScryptoFunctionIdent,
    args: &Vec<u8>,
) -> Result<(), DecompileError> {
    write!(
        f,
        "CALL_FUNCTION PackageAddress(\"{}\") \"{}\" \"{}\"",
        match &function_ident.package {
            ScryptoPackage::Global(package_address) => {
                package_address.display(context.bech32_encoder)
            }
        },
        function_ident.blueprint_name,
        function_ident.function_name,
    )?;
    format_args(f, context, args)?;
    f.write_str(";")?;
    Ok(())
}

pub fn decompile_call_native_function<F: fmt::Write>(
    f: &mut F,
    context: &mut DecompilationContext,
    function_ident: &NativeFunctionIdent,
    args: &Vec<u8>,
) -> Result<(), DecompileError> {
    // Try to recognize the invocation
    let blueprint_name = &function_ident.blueprint_name;
    let function_name = &function_ident.function_name;
    match (blueprint_name.as_str(), function_name.as_ref()) {
        ("ResourceManager", "burn") => {
            if let Ok(input) = scrypto_decode::<ResourceManagerBurnInvocation>(&args) {
                write!(
                    f,
                    "BURN_BUCKET Bucket({});",
                    context
                        .bucket_names
                        .get(&input.bucket.0)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", input.bucket.0)),
                )?;
                return Ok(());
            }
        }
        ("ResourceManager", "create") => {
            if let Ok(input) = scrypto_decode::<ResourceManagerCreateInvocation>(&args) {
                f.write_str(&format!(
                    "CREATE_RESOURCE {} {} {} {};",
                    IndexedScryptoValue::from_typed(&input.resource_type)
                        .display(context.for_value_display()),
                    IndexedScryptoValue::from_typed(&input.metadata)
                        .display(context.for_value_display()),
                    IndexedScryptoValue::from_typed(&input.access_rules)
                        .display(context.for_value_display()),
                    IndexedScryptoValue::from_typed(&input.mint_params)
                        .display(context.for_value_display()),
                ))?;
                return Ok(());
            }
        }
        _ => {}
    }

    // Fall back to generic representation
    write!(
        f,
        "CALL_NATIVE_FUNCTION \"{}\" \"{}\"",
        blueprint_name, function_name,
    )?;
    format_args(f, context, args)?;
    f.write_str(";")?;
    Ok(())
}

pub fn decompile_call_scrypto_method<F: fmt::Write>(
    f: &mut F,
    context: &mut DecompilationContext,
    method_ident: &ScryptoMethodIdent,
    args: &Vec<u8>,
) -> Result<(), DecompileError> {
    let receiver = match method_ident.receiver {
        ScryptoReceiver::Global(address) => {
            format!(
                "ComponentAddress(\"{}\")",
                address.display(context.bech32_encoder)
            )
        }
        ScryptoReceiver::Component(id) => {
            format!("Component(\"{}\")", format_id(&id))
        }
    };
    f.write_str(&format!(
        "CALL_METHOD {} \"{}\"",
        receiver, method_ident.method_name
    ))?;
    format_args(f, context, args)?;
    f.write_str(";")?;
    Ok(())
}

pub fn decompile_call_native_method<F: fmt::Write>(
    f: &mut F,
    context: &mut DecompilationContext,
    method_ident: &NativeMethodIdent,
    args: &Vec<u8>,
) -> Result<(), DecompileError> {
    // Try to recognize the invocation
    match (method_ident.receiver, method_ident.method_name.as_ref()) {
        (RENodeId::Global(GlobalAddress::Resource(resource_address)), "mint") => {
            if let Ok(input) = scrypto_decode::<ResourceManagerMintInvocation>(&args) {
                if let MintParams::Fungible { amount } = input.mint_params {
                    write!(
                        f,
                        "MINT_FUNGIBLE ResourceAddress(\"{}\") Decimal(\"{}\");",
                        resource_address.display(context.bech32_encoder),
                        amount,
                    )?;
                }
                return Ok(());
            }
        }
        _ => {}
    }

    // Fall back to generic representation
    let receiver = format_node_id(&method_ident.receiver, context);
    f.write_str(&format!(
        "CALL_NATIVE_METHOD {} \"{}\"",
        receiver, method_ident.method_name
    ))?;
    format_args(f, context, args)?;
    f.write_str(";")?;
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

fn format_node_id(node_id: &RENodeId, context: &mut DecompilationContext) -> String {
    match node_id {
        RENodeId::Global(global_address) => match global_address {
            GlobalAddress::Component(address) => {
                format!("Global(\"{}\")", address.display(context.bech32_encoder))
            }
            GlobalAddress::Package(address) => {
                format!("Global(\"{}\")", address.display(context.bech32_encoder))
            }
            GlobalAddress::Resource(address) => {
                format!("Global(\"{}\")", address.display(context.bech32_encoder))
            }
            GlobalAddress::System(address) => {
                format!("Global(\"{}\")", address.display(context.bech32_encoder))
            }
        },
        RENodeId::Bucket(id) => match context.bucket_names.get(id) {
            Some(name) => format!("Bucket(\"{}\")", name),
            None => format!("Bucket({}u32)", id),
        },
        RENodeId::Proof(id) => match context.proof_names.get(id) {
            Some(name) => format!("Proof(\"{}\")", name),
            None => format!("Proof({}u32)", id),
        },
        RENodeId::AuthZoneStack(id) => format!("AuthZoneStack({}u32)", id),
        RENodeId::Worktop => "Worktop".to_owned(),
        RENodeId::KeyValueStore(id) => format!("KeyValueStore(\"{}\")", format_id(id)),
        RENodeId::NonFungibleStore(id) => format!("NonFungibleStore(\"{}\")", format_id(id)),
        RENodeId::Component(id) => format!("Component(\"{}\")", format_id(id)),
        RENodeId::EpochManager(id) => format!("EpochManager(\"{}\")", format_id(id)),
        RENodeId::Clock(id) => format!("Clock(\"{}\")", format_id(id)),
        RENodeId::Vault(id) => format!("Vault(\"{}\")", format_id(id)),
        RENodeId::ResourceManager(id) => format!("ResourceManager(\"{}\")", format_id(id)),
        RENodeId::Package(id) => format!("Package(\"{}\")", format_id(id)),
        RENodeId::FeeReserve(id) => format!("FeeReserve({}u32)", id),
    }
}

fn format_id(id: &[u8; 36]) -> String {
    hex::encode(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::*;
    use radix_engine_interface::api::types::ResourceManagerFunction;
    use radix_engine_interface::core::NetworkDefinition;
    use radix_engine_interface::data::scrypto_encode;
    use radix_engine_interface::scrypto;

    #[scrypto(TypeId, Encode, Decode)]
    struct BadResourceManagerCreateInput {
        pub resource_type: ResourceType,
        pub metadata: HashMap<String, String>,
        pub access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        // pub mint_params: Option<MintParams>,
    }

    #[test]
    fn test_decompile_create_resource_with_invalid_arguments() {
        let network = NetworkDefinition::simulator();
        let manifest = decompile(
            &[Instruction::CallNativeFunction {
                function_ident: NativeFunctionIdent {
                    blueprint_name: "ResourceManager".to_owned(),
                    function_name: ResourceManagerFunction::Create.to_string(),
                },
                args: scrypto_encode(&BadResourceManagerCreateInput {
                    resource_type: ResourceType::NonFungible {
                        id_type: NonFungibleIdType::default(),
                    },
                    metadata: HashMap::new(),
                    access_rules: HashMap::new(),
                })
                .unwrap(),
            }],
            &network,
        )
        .unwrap();

        assert_eq!(manifest, "CALL_NATIVE_FUNCTION \"ResourceManager\" \"create\" Enum(\"NonFungible\", Enum(\"UUID\")) Array<Tuple>() Array<Tuple>();\n");
        compile_and_decompile_with_inversion_test(&manifest, &network, vec![]);
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

    #[test]
    fn test_decompile_complex() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/complex.rtm"),
            &NetworkDefinition::simulator(),
            vec![
                include_bytes!("../../examples/code.blob").to_vec(),
                include_bytes!("../../examples/abi.blob").to_vec(),
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
TAKE_FROM_WORKTOP_BY_IDS Array<NonFungibleId>(NonFungibleId(Bytes("031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f"))) ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket3");
CREATE_RESOURCE Enum("Fungible", 0u8) Array<Tuple>() Array<Tuple>() Some(Enum("Fungible", Decimal("1")));
CALL_METHOD ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "deposit_batch" Expression("ENTIRE_WORKTOP");
DROP_ALL_PROOFS;
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "complicated_method" Decimal("1") PreciseDecimal("2");
PUBLISH_PACKAGE_WITH_OWNER Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d") NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", Bytes("031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f"));
"#
        )
    }

    #[test]
    fn test_decompile_call_function() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/call_function.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
        );

        assert_eq!(
            canonical_manifest,
            r#"CALL_FUNCTION PackageAddress("package_sim1qy4hrp8a9apxldp5cazvxgwdj80cxad4u8cpkaqqnhlsa3lfpe") "Blueprint" "function";
CALL_NATIVE_FUNCTION "EpochManager" "create";
CALL_NATIVE_FUNCTION "ResourceManager" "create";
CALL_NATIVE_FUNCTION "Package" "publish";
CALL_NATIVE_FUNCTION "TransactionProcessor" "run";
"#
        )
    }

    #[test]
    fn test_decompile_call_method() {
        let network = NetworkDefinition::simulator();

        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/call_method.rtm"),
            &network,
            vec![],
        );
        assert_eq!(
            canonical_manifest,
            r#"CALL_METHOD ComponentAddress("component_sim1qgvyxt5rrjhwctw7krgmgkrhv82zuamcqkq75tkkrwgs00m736") "free_xrd";
CALL_METHOD Component("000000000000000000000000000000000000000000000000000000000000000005000000") "free_xrd";
TAKE_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket1");
CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Proof("proof1");
CALL_NATIVE_METHOD Bucket("bucket1") "get_resource_address";
CALL_NATIVE_METHOD Bucket(1u32) "get_resource_address";
CALL_NATIVE_METHOD Bucket(513u32) "get_resource_address";
CALL_NATIVE_METHOD Bucket(1u32) "get_resource_address";
CALL_NATIVE_METHOD AuthZoneStack(1u32) "drain";
CALL_NATIVE_METHOD Worktop "drain";
CALL_NATIVE_METHOD KeyValueStore("000000000000000000000000000000000000000000000000000000000000000005000000") "method";
CALL_NATIVE_METHOD NonFungibleStore("000000000000000000000000000000000000000000000000000000000000000005000000") "method";
CALL_NATIVE_METHOD Component("000000000000000000000000000000000000000000000000000000000000000005000000") "add_access_check";
CALL_NATIVE_METHOD EpochManager("000000000000000000000000000000000000000000000000000000000000000005000000") "get_transaction_hash";
CALL_NATIVE_METHOD Vault("000000000000000000000000000000000000000000000000000000000000000005000000") "get_resource_address";
CALL_NATIVE_METHOD ResourceManager("000000000000000000000000000000000000000000000000000000000000000000000005") "burn";
CALL_NATIVE_METHOD Package("000000000000000000000000000000000000000000000000000000000000000000000005") "method";
CALL_NATIVE_METHOD Global("resource_sim1qrc4s082h9trka3yrghwragylm3sdne0u668h2sy6c9sckkpn6") "method";
"#
        )
    }

    #[test]
    fn test_decompile_any_value() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/any_value.rtm"),
            &NetworkDefinition::simulator(),
            vec![include_bytes!("../../examples/code.blob").to_vec()],
        );

        assert_eq!(
            canonical_manifest,
            r#"TAKE_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket1");
CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Proof("proof1");
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "with_aliases" None None Some("hello") Some("hello") Ok("test") Ok("test") Err("test123") Err("test123") Bytes("050aff") Bytes("050aff");
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "with_all_types" PackageAddress("package_sim1qyqzcexvnyg60z7lnlwauh66nhzg3m8tch2j8wc0e70qkydk8r") ComponentAddress("account_sim1q0u9gxewjxj8nhxuaschth2mgencma2hpkgwz30s9wlslthace") ResourceAddress("resource_sim1qq8cays25704xdyap2vhgmshkkfyr023uxdtk59ddd4qs8cr5v") SystemAddress("system_sim1qne8qu4seyvzfgd94p3z8rjcdl3v0nfhv84judpum2lq7x4635") Component("000000000000000000000000000000000000000000000000000000000000000005000000") KeyValueStore("000000000000000000000000000000000000000000000000000000000000000005000000") Bucket("bucket1") Proof("proof1") Vault("000000000000000000000000000000000000000000000000000000000000000005000000") Expression("ALL_WORKTOP_RESOURCES") Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", "value") NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", 123u32) NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", 456u64) NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", Bytes("031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f")) NonFungibleAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", 1234567890u128) Hash("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824") EcdsaSecp256k1PublicKey("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798") EcdsaSecp256k1Signature("0079224ea514206706298d8d620f660828f7987068d6d02757e6f3cbbf4a51ab133395db69db1bc9b2726dd99e34efc252d8258dcb003ebaba42be349f50f7765e") EddsaEd25519PublicKey("4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29") EddsaEd25519Signature("ce993adc51111309a041faa65cbcf1154d21ed0ecdc2d54070bc90b9deb744aa8605b3f686fa178fba21070b4a4678e54eee3486a881e0e328251cd37966de09") Decimal("1.2") PreciseDecimal("1.2") NonFungibleId(Bytes("031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f")) NonFungibleId(12u32) NonFungibleId(12345u64) NonFungibleId(1234567890u128) NonFungibleId("SomeId");
"#
        )
    }

    #[test]
    fn test_decompile_non_fungible_ids() {
        let canonical_manifest = compile_and_decompile_with_inversion_test(
            include_str!("../../examples/non_fungible_ids_canonical.rtm"),
            &NetworkDefinition::simulator(),
            vec![],
        );

        // Act
        compile_and_decompile_with_inversion_test(
            &canonical_manifest,
            &NetworkDefinition::simulator(),
            vec![],
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
}
