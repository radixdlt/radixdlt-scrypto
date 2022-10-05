use sbor::rust::collections::*;
use sbor::rust::fmt;
use sbor::{encode_any, DecodeError, Value};
use scrypto::address::{AddressError, Bech32Encoder};
use scrypto::buffer::scrypto_decode;
use scrypto::core::{
    BucketFnIdentifier, FnIdentifier, NativeFnIdentifier, NetworkDefinition, Receiver,
    ResourceManagerFnIdentifier,
};
use scrypto::engine::types::*;
use scrypto::resource::{
    ConsumingBucketBurnInput, MintParams, ResourceManagerCreateInput, ResourceManagerMintInput,
};
use scrypto::values::*;

use crate::errors::*;
use crate::model::*;
use crate::validation::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    IdValidationError(IdValidationError),
    DecodeError(DecodeError),
    AddressError(AddressError),
    InvalidValue(ScryptoValueFormatterError),
    FormattingError(fmt::Error),
    UnrecognizedNativeFunction,
}

impl From<ScryptoValueFormatterError> for DecompileError {
    fn from(error: ScryptoValueFormatterError) -> Self {
        Self::InvalidValue(error)
    }
}

impl From<fmt::Error> for DecompileError {
    fn from(error: fmt::Error) -> Self {
        Self::FormattingError(error)
    }
}

pub struct DecompilationContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
    pub id_validator: IdValidator,
    pub bucket_names: HashMap<BucketId, String>,
    pub proof_names: HashMap<ProofId, String>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(bech32_encoder: &'a Bech32Encoder) -> Self {
        Self {
            bech32_encoder: Some(bech32_encoder),
            id_validator: IdValidator::new(),
            bucket_names: HashMap::<BucketId, String>::new(),
            proof_names: HashMap::<ProofId, String>::new(),
        }
    }

    pub fn new_with_optional_network(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            id_validator: IdValidator::new(),
            bucket_names: HashMap::<BucketId, String>::new(),
            proof_names: HashMap::<ProofId, String>::new(),
        }
    }
}

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
                .id_validator
                .new_bucket()
                .map_err(DecompileError::IdValidationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            write!(
                f,
                "TAKE_FROM_WORKTOP ResourceAddress(\"{}\") Bucket(\"{}\");",
                resource_address.displayable(context.bech32_encoder),
                name
            )?;
            context.bucket_names.insert(bucket_id, name);
        }
        Instruction::TakeFromWorktopByAmount {
            amount,
            resource_address,
        } => {
            let bucket_id = context
                .id_validator
                .new_bucket()
                .map_err(DecompileError::IdValidationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            context.bucket_names.insert(bucket_id, name.clone());
            write!(
                f,
                "TAKE_FROM_WORKTOP_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Bucket(\"{}\");",
                amount,
                resource_address.displayable(context.bech32_encoder),
                name
            )?;
        }
        Instruction::TakeFromWorktopByIds {
            ids,
            resource_address,
        } => {
            let bucket_id = context
                .id_validator
                .new_bucket()
                .map_err(DecompileError::IdValidationError)?;
            let name = format!("bucket{}", context.bucket_names.len() + 1);
            context.bucket_names.insert(bucket_id, name.clone());
            write!(
                f,
                "TAKE_FROM_WORKTOP_BY_IDS Set<NonFungibleId>({}) ResourceAddress(\"{}\") Bucket(\"{}\");",
                ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.displayable(context.bech32_encoder),
                name
            )?;
        }
        Instruction::ReturnToWorktop { bucket_id } => {
            context
                .id_validator
                .drop_bucket(*bucket_id)
                .map_err(DecompileError::IdValidationError)?;
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
                resource_address.displayable(context.bech32_encoder)
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
                resource_address.displayable(context.bech32_encoder)
            )?;
        }
        Instruction::AssertWorktopContainsByIds {
            ids,
            resource_address,
        } => {
            write!(
                f,
                "ASSERT_WORKTOP_CONTAINS_BY_IDS Set<NonFungibleId>({}) ResourceAddress(\"{}\");",
                ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource_address.displayable(context.bech32_encoder)
            )?;
        }
        Instruction::PopFromAuthZone => {
            let proof_id = context
                .id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(DecompileError::IdValidationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(f, "POP_FROM_AUTH_ZONE Proof(\"{}\");", name)?;
        }
        Instruction::PushToAuthZone { proof_id } => {
            context
                .id_validator
                .drop_proof(*proof_id)
                .map_err(DecompileError::IdValidationError)?;
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
                .id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(DecompileError::IdValidationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"{}\") Proof(\"{}\");",
                resource_address.displayable(context.bech32_encoder),
                name
            )?;
        }
        Instruction::CreateProofFromAuthZoneByAmount {
            amount,
            resource_address,
        } => {
            let proof_id = context
                .id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(DecompileError::IdValidationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Proof(\"{}\");",
                amount,
                resource_address.displayable(context.bech32_encoder),
                name
            )?;
        }
        Instruction::CreateProofFromAuthZoneByIds {
            ids,
            resource_address,
        } => {
            let proof_id = context
                .id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(DecompileError::IdValidationError)?;
            let name = format!("proof{}", context.proof_names.len() + 1);
            context.proof_names.insert(proof_id, name.clone());
            write!(
                f,
                "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS Set<NonFungibleId>({}) ResourceAddress(\"{}\") Proof(\"{}\");",ids.iter()
                .map(|k| format!("NonFungibleId(\"{}\")", k))
                .collect::<Vec<String>>()
                .join(", "),
                resource_address.displayable(context.bech32_encoder),
                name
            )?;
        }
        Instruction::CreateProofFromBucket { bucket_id } => {
            let proof_id = context
                .id_validator
                .new_proof(ProofKind::BucketProof(*bucket_id))
                .map_err(DecompileError::IdValidationError)?;
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
                .id_validator
                .clone_proof(*proof_id)
                .map_err(DecompileError::IdValidationError)?;
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
            context
                .id_validator
                .drop_proof(*proof_id)
                .map_err(DecompileError::IdValidationError)?;
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
            context
                .id_validator
                .drop_all_proofs()
                .map_err(DecompileError::IdValidationError)?;
            f.write_str("DROP_ALL_PROOFS;")?;
        }
        Instruction::CallFunction {
            fn_identifier,

            args,
        } => match fn_identifier {
            FnIdentifier::Scrypto {
                package_address,
                blueprint_name,
                ident,
            } => {
                f.write_str(&format!(
                    "CALL_FUNCTION PackageAddress(\"{}\") \"{}\" \"{}\"",
                    package_address.displayable(context.bech32_encoder),
                    blueprint_name,
                    ident
                ))?;
                let validated_arg =
                    ScryptoValue::from_slice(&args).map_err(DecompileError::DecodeError)?;
                if let Value::Struct { fields } = validated_arg.dom {
                    for field in fields {
                        let bytes = encode_any(&field);
                        let validated_arg = ScryptoValue::from_slice(&bytes)
                            .map_err(DecompileError::DecodeError)?;
                        context
                            .id_validator
                            .move_resources(&validated_arg)
                            .map_err(DecompileError::IdValidationError)?;

                        f.write_char(' ')?;
                        f.write_str(&validated_arg.to_string_with_fixed_context(
                            context.bech32_encoder,
                            &context.bucket_names,
                            &context.proof_names,
                        )?)?;
                    }
                } else {
                    panic!("Should not get here.");
                }
                f.write_str(";")?;
            }
            FnIdentifier::Native(native_fn_identifier) => match native_fn_identifier {
                NativeFnIdentifier::ResourceManager(ResourceManagerFnIdentifier::Create) => {
                    let input: ResourceManagerCreateInput =
                        scrypto_decode(&args).map_err(DecompileError::DecodeError)?;

                    f.write_str(&format!(
                        "CREATE_RESOURCE {} {} {} {};",
                        ScryptoValue::from_typed(&input.resource_type)
                            .to_string_with_fixed_context(
                                context.bech32_encoder,
                                &context.bucket_names,
                                &context.proof_names,
                            )?,
                        ScryptoValue::from_typed(&input.metadata).to_string_with_fixed_context(
                            context.bech32_encoder,
                            &context.bucket_names,
                            &context.proof_names,
                        )?,
                        ScryptoValue::from_typed(&input.access_rules)
                            .to_string_with_fixed_context(
                                context.bech32_encoder,
                                &context.bucket_names,
                                &context.proof_names,
                            )?,
                        ScryptoValue::from_typed(&input.mint_params).to_string_with_fixed_context(
                            context.bech32_encoder,
                            &context.bucket_names,
                            &context.proof_names,
                        )?,
                    ))?;
                }
                _ => return Err(DecompileError::UnrecognizedNativeFunction),
            },
        },
        Instruction::CallMethod {
            method_identifier,
            args,
        } => match method_identifier {
            MethodIdentifier::Scrypto {
                component_address,
                ident,
            } => {
                f.write_str(&format!(
                    "CALL_METHOD ComponentAddress(\"{}\") \"{}\"",
                    component_address.displayable(context.bech32_encoder),
                    ident
                ))?;

                let validated_arg =
                    ScryptoValue::from_slice(&args).map_err(DecompileError::DecodeError)?;
                if let Value::Struct { fields } = validated_arg.dom {
                    for field in fields {
                        let bytes = encode_any(&field);
                        let validated_arg = ScryptoValue::from_slice(&bytes)
                            .map_err(DecompileError::DecodeError)?;
                        context
                            .id_validator
                            .move_resources(&validated_arg)
                            .map_err(DecompileError::IdValidationError)?;

                        f.write_char(' ')?;
                        f.write_str(&validated_arg.to_string_with_fixed_context(
                            context.bech32_encoder,
                            &context.bucket_names,
                            &context.proof_names,
                        )?)?;
                    }
                } else {
                    panic!("Should not get here.");
                }

                f.write_str(";")?;
            }
            MethodIdentifier::Native {
                native_fn_identifier,
                receiver,
            } => match (native_fn_identifier, receiver) {
                (
                    NativeFnIdentifier::Bucket(BucketFnIdentifier::Burn),
                    Receiver::Consumed(RENodeId::Bucket(bucket_id)),
                ) => {
                    let _input: ConsumingBucketBurnInput =
                        scrypto_decode(&args).map_err(DecompileError::DecodeError)?;

                    write!(
                        f,
                        "BURN_BUCKET Bucket({});",
                        context
                            .bucket_names
                            .get(&bucket_id)
                            .map(|name| format!("\"{}\"", name))
                            .unwrap_or(format!("{}u32", bucket_id)),
                    )?;
                }
                (
                    NativeFnIdentifier::ResourceManager(ResourceManagerFnIdentifier::Mint),
                    Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                ) => {
                    let input: ResourceManagerMintInput =
                        scrypto_decode(&args).map_err(DecompileError::DecodeError)?;
                    match input.mint_params {
                        MintParams::Fungible { amount } => {
                            write!(
                                f,
                                "MINT_FUNGIBLE ResourceAddress(\"{}\") Decimal(\"{}\");",
                                resource_address.displayable(context.bech32_encoder),
                                amount,
                            )?;
                        }
                        _ => return Err(DecompileError::UnrecognizedNativeFunction),
                    }
                }
                _ => return Err(DecompileError::UnrecognizedNativeFunction),
            },
        },
        Instruction::PublishPackage { code, abi } => {
            write!(f, "PUBLISH_PACKAGE Blob(\"{}\") Blob(\"{}\");", code, abi)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::*;
    use scrypto::core::NetworkDefinition;

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn test_decompile() {
        let network = NetworkDefinition::simulator();
        let manifest_str = include_str!("../../examples/complex.rtm");
        let blobs = vec![
            include_bytes!("../../examples/code.blob").to_vec(),
            include_bytes!("../../examples/abi.blob").to_vec(),
        ];
        let manifest = compile(manifest_str, &network, blobs).unwrap();

        let manifest2 = decompile(&manifest.instructions, &network).unwrap();
        assert_eq!(
            manifest2,
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
TAKE_FROM_WORKTOP_BY_IDS Set<NonFungibleId>(NonFungibleId("0905000000"), NonFungibleId("0907000000")) ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket3");
CREATE_RESOURCE Enum("Fungible", 0u8) Map<String, String>() Map<Enum, Tuple>() Some(Enum("Fungible", Decimal("1")));
CALL_METHOD ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "deposit_batch" Expression("ENTIRE_WORKTOP");
DROP_ALL_PROOFS;
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "complicated_method" Decimal("1") PreciseDecimal("2");
PUBLISH_PACKAGE Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d");
"#
        )
    }
}
