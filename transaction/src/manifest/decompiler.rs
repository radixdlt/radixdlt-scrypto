use sbor::rust::collections::*;
use sbor::{encode_any, DecodeError, Value};
use scrypto::address::{AddressError, Bech32Encoder};
use scrypto::core::NetworkDefinition;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::errors::*;
use crate::model::*;
use crate::validation::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    IdValidationError(IdValidationError),
    DecodeError(DecodeError),
    AddressError(AddressError),
}

pub fn decompile(
    manifest: &TransactionManifest,
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    let bech32_encoder = Bech32Encoder::new(network);
    let mut buf = String::new();
    let mut id_validator = IdValidator::new();
    let mut buckets = HashMap::<BucketId, String>::new();
    let mut proofs = HashMap::<ProofId, String>::new();
    for inst in &manifest.instructions {
        match inst.clone() {
            Instruction::TakeFromWorktop { resource_address } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP ResourceAddress(\"{}\") Bucket(\"{}\");\n",
                    bech32_encoder.encode_resource_address(&resource_address),
                    name
                ));
            }
            Instruction::TakeFromWorktopByAmount {
                amount,
                resource_address,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Bucket(\"{}\");\n",
                    amount, bech32_encoder.encode_resource_address(&resource_address), name
                ));
            }
            Instruction::TakeFromWorktopByIds {
                ids,
                resource_address,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP_BY_IDS Set<NonFungibleId>({}) ResourceAddress(\"{}\") Bucket(\"{}\");\n",
                    ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    bech32_encoder.encode_resource_address(&resource_address), name
                ));
            }
            Instruction::ReturnToWorktop { bucket_id } => {
                id_validator
                    .drop_bucket(bucket_id)
                    .map_err(DecompileError::IdValidationError)?;
                buf.push_str(&format!(
                    "RETURN_TO_WORKTOP Bucket({});\n",
                    buckets
                        .get(&bucket_id)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", bucket_id))
                ));
            }
            Instruction::AssertWorktopContains { resource_address } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS ResourceAddress(\"{}\");\n",
                    bech32_encoder.encode_resource_address(&resource_address)
                ));
            }
            Instruction::AssertWorktopContainsByAmount {
                amount,
                resource_address,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\");\n",
                    amount,
                    bech32_encoder.encode_resource_address(&resource_address)
                ));
            }
            Instruction::AssertWorktopContainsByIds {
                ids,
                resource_address,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS_BY_IDS Set<NonFungibleId>({}) ResourceAddress(\"{}\");\n",
                    ids.iter()
                        .map(|k| format!("NonFungibleId(\"{}\")", k))
                        .collect::<Vec<String>>()
                        .join(", "),
                    bech32_encoder.encode_resource_address(&resource_address)
                ));
            }
            Instruction::PopFromAuthZone => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!("POP_FROM_AUTH_ZONE Proof(\"{}\");\n", name));
            }
            Instruction::PushToAuthZone { proof_id } => {
                id_validator
                    .drop_proof(proof_id)
                    .map_err(DecompileError::IdValidationError)?;
                buf.push_str(&format!(
                    "PUSH_TO_AUTH_ZONE Proof({});\n",
                    proofs
                        .get(&proof_id)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", proof_id))
                ));
            }
            Instruction::ClearAuthZone => {
                buf.push_str("CLEAR_AUTH_ZONE;\n");
            }
            Instruction::CreateProofFromAuthZone { resource_address } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"{}\") Proof(\"{}\");\n",
                    bech32_encoder.encode_resource_address(&resource_address),
                    name
                ));
            }
            Instruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_address,
            } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Proof(\"{}\");\n",
                    amount,
                    bech32_encoder.encode_resource_address(&resource_address), name
                ));
            }
            Instruction::CreateProofFromAuthZoneByIds {
                ids,
                resource_address,
            } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS Set<NonFungibleId>({}) ResourceAddress(\"{}\") Proof(\"{}\");\n",ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    bech32_encoder.encode_resource_address(&resource_address), name
                ));
            }
            Instruction::CreateProofFromBucket { bucket_id } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::BucketProof(bucket_id))
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_BUCKET Bucket({}) Proof(\"{}\");\n",
                    buckets
                        .get(&bucket_id)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", bucket_id)),
                    name
                ));
            }
            Instruction::CloneProof { proof_id } => {
                let proof_id2 = id_validator
                    .clone_proof(proof_id)
                    .map_err(DecompileError::IdValidationError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id2, name.clone());
                buf.push_str(&format!(
                    "CLONE_PROOF Proof({}) Proof(\"{}\");\n",
                    proofs
                        .get(&proof_id)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", proof_id)),
                    name
                ));
            }
            Instruction::DropProof { proof_id } => {
                id_validator
                    .drop_proof(proof_id)
                    .map_err(DecompileError::IdValidationError)?;
                buf.push_str(&format!(
                    "DROP_PROOF Proof({});\n",
                    proofs
                        .get(&proof_id)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", proof_id)),
                ));
            }
            Instruction::DropAllProofs => {
                id_validator
                    .drop_all_proofs()
                    .map_err(DecompileError::IdValidationError)?;
                buf.push_str("DROP_ALL_PROOFS;\n");
            }
            Instruction::CallFunction {
                package_address,
                blueprint_name,
                method_name,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_FUNCTION PackageAddress(\"{}\") \"{}\" \"{}\"",
                    bech32_encoder.encode_package_address(&package_address),
                    blueprint_name,
                    method_name
                ));
                let validated_arg =
                    ScryptoValue::from_slice(&args).map_err(DecompileError::DecodeError)?;
                if let Value::Struct { fields } = validated_arg.dom {
                    for field in fields {
                        let bytes = encode_any(&field);
                        let validated_arg = ScryptoValue::from_slice(&bytes)
                            .map_err(DecompileError::DecodeError)?;
                        id_validator
                            .move_resources(&validated_arg)
                            .map_err(DecompileError::IdValidationError)?;

                        buf.push(' ');
                        buf.push_str(&validated_arg.to_string_with_context(&buckets, &proofs));
                    }
                } else {
                    panic!("Should not get here.");
                }
                buf.push_str(";\n");
            }
            Instruction::CallMethod {
                component_address,
                method_name,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_METHOD ComponentAddress(\"{}\") \"{}\"",
                    bech32_encoder.encode_component_address(&component_address),
                    method_name
                ));

                let validated_arg =
                    ScryptoValue::from_slice(&args).map_err(DecompileError::DecodeError)?;
                if let Value::Struct { fields } = validated_arg.dom {
                    for field in fields {
                        let bytes = encode_any(&field);
                        let validated_arg = ScryptoValue::from_slice(&bytes)
                            .map_err(DecompileError::DecodeError)?;
                        id_validator
                            .move_resources(&validated_arg)
                            .map_err(DecompileError::IdValidationError)?;

                        buf.push(' ');
                        buf.push_str(&validated_arg.to_string_with_context(&buckets, &proofs));
                    }
                } else {
                    panic!("Should not get here.");
                }

                buf.push_str(";\n");
            }
            Instruction::PublishPackage { code, abi } => {
                buf.push_str(&format!(
                    "PUBLISH_PACKAGE Blob(\"{}.data\") Blob(\"{}.data\");\n",
                    code, abi
                ));
            }
        }
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::*;
    use scrypto::core::NetworkDefinition;

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn test_decompile() {
        let network = NetworkDefinition::local_simulator();
        let mut blob_loader = FileBlobLoader::new("./examples/");
        let manifest_str = include_str!("../../examples/complex.rtm");
        let manifest = compile(manifest_str, &network, &mut blob_loader).unwrap();

        let manifest2 = decompile(&manifest, &network).unwrap();
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
CALL_METHOD ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "deposit_batch" Expression("ENTIRE_WORKTOP");
DROP_ALL_PROOFS;
CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "complicated_method" Decimal("1") PreciseDecimal("2");
PUBLISH_PACKAGE Blob("36dae540b7889956f1f1d8d46ba23e5e44bf5723aef2a8e6b698686c02583618.data") Blob("15e8699a6d63a96f66f6feeb609549be2688b96b02119f260ae6dfd012d16a5d.data");
"#
        )
    }
}
