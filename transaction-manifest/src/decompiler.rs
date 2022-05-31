use radix_engine::engine::*;
use radix_engine::model::*;
use sbor::rust::collections::*;
use sbor::{encode_any, Value};
use scrypto::engine::types::*;
use scrypto::values::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    IdValidatorError(IdValidatorError),
    ParseScryptoValueError(ParseScryptoValueError),
}

pub fn decompile(tx: &Transaction) -> Result<String, DecompileError> {
    let mut buf = String::new();
    let mut id_validator = IdValidator::new();
    let mut buckets = HashMap::<BucketId, String>::new();
    let mut proofs = HashMap::<ProofId, String>::new();
    for inst in &tx.instructions {
        match inst.clone() {
            Instruction::TakeFromWorktop { resource_address } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP ResourceAddress(\"{}\") Bucket(\"{}\");\n",
                    resource_address, name
                ));
            }
            Instruction::TakeFromWorktopByAmount {
                amount,
                resource_address,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Bucket(\"{}\");\n",
                    amount, resource_address, name
                ));
            }
            Instruction::TakeFromWorktopByIds {
                ids,
                resource_address,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP_BY_IDS TreeSet<NonFungibleId>({}) ResourceAddress(\"{}\") Bucket(\"{}\");\n",
                    ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    resource_address, name
                ));
            }
            Instruction::ReturnToWorktop { bucket_id } => {
                id_validator
                    .drop_bucket(bucket_id)
                    .map_err(DecompileError::IdValidatorError)?;
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
                    resource_address
                ));
            }
            Instruction::AssertWorktopContainsByAmount {
                amount,
                resource_address,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\");\n",
                    amount, resource_address
                ));
            }
            Instruction::AssertWorktopContainsByIds {
                ids,
                resource_address,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS_BY_IDS TreeSet<NonFungibleId>({}) ResourceAddress(\"{}\");\n",
                    ids.iter()
                        .map(|k| format!("NonFungibleId(\"{}\")", k))
                        .collect::<Vec<String>>()
                        .join(", "),
                    resource_address
                ));
            }
            Instruction::PopFromAuthZone => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!("POP_FROM_AUTH_ZONE Proof(\"{}\");\n", name));
            }
            Instruction::PushToAuthZone { proof_id } => {
                id_validator
                    .drop_proof(proof_id)
                    .map_err(DecompileError::IdValidatorError)?;
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
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"{}\") Proof(\"{}\");\n",
                    resource_address, name
                ));
            }
            Instruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_address,
            } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT Decimal(\"{}\") ResourceAddress(\"{}\") Proof(\"{}\");\n",
                    amount,
                    resource_address, name
                ));
            }
            Instruction::CreateProofFromAuthZoneByIds {
                ids,
                resource_address,
            } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS TreeSet<NonFungibleId>({}) ResourceAddress(\"{}\") Proof(\"{}\");\n",ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    resource_address, name
                ));
            }
            Instruction::CreateProofFromBucket { bucket_id } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::BucketProof(bucket_id))
                    .map_err(DecompileError::IdValidatorError)?;
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
                    .map_err(DecompileError::IdValidatorError)?;
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
                    .map_err(DecompileError::IdValidatorError)?;
                buf.push_str(&format!(
                    "DROP_PROOF Proof({});\n",
                    proofs
                        .get(&proof_id)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", proof_id)),
                ));
            }
            Instruction::CallFunction {
                package_address,
                blueprint_name,
                method_name,
                arg,
            } => {
                buf.push_str(&format!(
                    "CALL_FUNCTION PackageAddress(\"{}\") \"{}\" \"{}\"",
                    package_address, blueprint_name, method_name
                ));
                let validated_arg = ScryptoValue::from_slice(&arg)
                    .map_err(DecompileError::ParseScryptoValueError)?;
                if let Value::Struct { fields } = validated_arg.dom {
                    for field in fields {
                        let bytes = encode_any(&field);
                        let validated_arg = ScryptoValue::from_slice(&bytes)
                            .map_err(DecompileError::ParseScryptoValueError)?;
                        id_validator
                            .move_resources(&validated_arg)
                            .map_err(DecompileError::IdValidatorError)?;

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
                arg,
            } => {
                buf.push_str(&format!(
                    "CALL_METHOD ComponentAddress(\"{}\") \"{}\"",
                    component_address, method_name
                ));

                let validated_arg = ScryptoValue::from_slice(&arg)
                    .map_err(DecompileError::ParseScryptoValueError)?;
                if let Value::Struct { fields } = validated_arg.dom {
                    for field in fields {
                        let bytes = encode_any(&field);
                        let validated_arg = ScryptoValue::from_slice(&bytes)
                            .map_err(DecompileError::ParseScryptoValueError)?;
                        id_validator
                            .move_resources(&validated_arg)
                            .map_err(DecompileError::IdValidatorError)?;

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
                    .move_all_resources()
                    .map_err(DecompileError::IdValidatorError)?;
                buf.push_str(&format!(
                    "CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"{}\") \"{}\";\n",
                    component_address, method
                ));
            }
            Instruction::PublishPackage { package } => {
                buf.push_str(&format!(
                    "PUBLISH_PACKAGE Bytes(\"{}\");\n",
                    hex::encode(&package)
                ));
            }
            Instruction::Nonce { .. } => {
                // TODO: add support for this
            }
        }
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;

    #[test]
    fn test_decompile() {
        let tx = compile(include_str!("../examples/complex.rtm")).unwrap();

        let manifest = &decompile(&tx).unwrap();
        println!("{}", manifest);

        assert_eq!(compile(manifest).unwrap(), tx);
    }
}
