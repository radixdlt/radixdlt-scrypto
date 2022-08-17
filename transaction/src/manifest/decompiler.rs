use sbor::rust::collections::*;
use sbor::{encode_any, DecodeError, Value};
use scrypto::address::{AddressError, Bech32Encoder};
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
    bech32_encoder: &Bech32Encoder,
) -> Result<String, DecompileError> {
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
                    bech32_encoder
                        .encode_resource_address(&resource_address)
                        .map_err(|err| DecompileError::AddressError(err))?,
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
                    amount, bech32_encoder.encode_resource_address(&resource_address).map_err(|err| DecompileError::AddressError(err))?, name
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
                    bech32_encoder.encode_resource_address(&resource_address).map_err(|err| DecompileError::AddressError(err))?, name
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
                    bech32_encoder
                        .encode_resource_address(&resource_address)
                        .map_err(|err| DecompileError::AddressError(err))?
                ));
            }
            Instruction::AssertWorktopContainsByAmount {
                amount,
                resource_address,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\");\n",
                    amount,
                    bech32_encoder
                        .encode_resource_address(&resource_address)
                        .map_err(|err| DecompileError::AddressError(err))?
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
                    bech32_encoder.encode_resource_address(&resource_address).map_err(|err| DecompileError::AddressError(err))?
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
                    bech32_encoder
                        .encode_resource_address(&resource_address)
                        .map_err(|err| DecompileError::AddressError(err))?,
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
                    bech32_encoder.encode_resource_address(&resource_address).map_err(|err| DecompileError::AddressError(err))?, name
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
                    bech32_encoder.encode_resource_address(&resource_address).map_err(|err| DecompileError::AddressError(err))?, name
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
                    bech32_encoder
                        .encode_package_address(&package_address)
                        .map_err(|err| DecompileError::AddressError(err))?,
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
                    bech32_encoder
                        .encode_component_address(&component_address)
                        .map_err(|err| DecompileError::AddressError(err))?,
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
            Instruction::CallMethodWithAllResources {
                component_address,
                method,
            } => {
                id_validator
                    .move_all_buckets()
                    .map_err(DecompileError::IdValidationError)?;
                buf.push_str(&format!(
                    "CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"{}\") \"{}\";\n",
                    bech32_encoder
                        .encode_component_address(&component_address)
                        .map_err(|err| DecompileError::AddressError(err))?,
                    method
                ));
            }
            Instruction::PublishPackage { package } => {
                buf.push_str(&format!(
                    "PUBLISH_PACKAGE Bytes(\"{}\");\n",
                    hex::encode(&package)
                ));
            }
        }
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::compile;
    use scrypto::core::Network;

    #[test]
    fn test_decompile() {
        let tx = compile(
            include_str!("../../examples/complex.rtm"),
            &Network::LocalSimulator,
        )
        .unwrap();

        let bech32_encoder = Bech32Encoder::new_from_network(&Network::LocalSimulator);
        let manifest = &decompile(&tx, &bech32_encoder).unwrap();
        println!("{}", manifest);

        assert_eq!(compile(manifest, &Network::LocalSimulator).unwrap(), tx);
    }
}
