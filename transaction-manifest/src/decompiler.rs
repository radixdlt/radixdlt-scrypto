use radix_engine::engine::*;
use radix_engine::errors::*;
use radix_engine::model::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    IdValidatorError(IdValidatorError),
    DataValidationError(DataValidationError),
}

pub fn decompile(tx: &Transaction) -> Result<String, DecompileError> {
    let mut buf = String::new();
    let mut id_validator = IdValidator::new();
    let mut buckets = HashMap::<BucketId, String>::new();
    let mut proofs = HashMap::<ProofId, String>::new();
    for inst in &tx.instructions {
        match inst.clone() {
            Instruction::TakeFromWorktop { resource_def_id } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP ResourceDefId(\"{}\") Bucket(\"{}\");\n",
                    resource_def_id, name
                ));
            }
            Instruction::TakeFromWorktopByAmount {
                amount,
                resource_def_id,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP_BY_AMOUNT Decimal(\"{}\") ResourceDefId(\"{}\") Bucket(\"{}\");\n",
                    amount, resource_def_id, name
                ));
            }
            Instruction::TakeFromWorktopByIds {
                ids,
                resource_def_id,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP_BY_IDS TreeSet<NonFungibleId>({}) ResourceDefId(\"{}\") Bucket(\"{}\");\n",
                    ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    resource_def_id, name
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
            Instruction::AssertWorktopContains { resource_def_id } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS ResourceDefId(\"{}\");\n",
                    resource_def_id
                ));
            }
            Instruction::AssertWorktopContainsByAmount {
                amount,
                resource_def_id,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS_BY_AMOUNT Decimal(\"{}\") ResourceDefId(\"{}\");\n",
                    amount, resource_def_id
                ));
            }
            Instruction::AssertWorktopContainsByIds {
                ids,
                resource_def_id,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS_BY_IDS TreeSet<NonFungibleId>({}) ResourceDefId(\"{}\");\n",
                    ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    resource_def_id
                ));
            }
            Instruction::TakeFromAuthZone => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!("TAKE_FROM_AUTH_ZONE Proof(\"{}\");\n", name));
            }
            Instruction::ReturnToAuthZone { proof_id } => {
                id_validator
                    .drop_proof(proof_id)
                    .map_err(DecompileError::IdValidatorError)?;
                buf.push_str(&format!(
                    "RETURN_TO_AUTH_ZONE Proof({});\n",
                    proofs
                        .get(&proof_id)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", proof_id))
                ));
            }
            Instruction::ClearAuthZone => {
                buf.push_str("CLEAR_AUTH_ZONE;\n");
            }
            Instruction::CreateProofFromAuthZone { resource_def_id } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE ResourceDefId(\"{}\") Proof(\"{}\");\n",
                    resource_def_id, name
                ));
            }
            Instruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_def_id,
            } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT Decimal(\"{}\") ResourceDefId(\"{}\") Proof(\"{}\");\n",
                    amount,
                    resource_def_id, name
                ));
            }
            Instruction::CreateProofFromAuthZoneByIds {
                ids,
                resource_def_id,
            } => {
                let proof_id = id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("proof{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS TreeSet<NonFungibleId>({}) ResourceDefId(\"{}\") Proof(\"{}\");\n",ids.iter()
                    .map(|k| format!("NonFungibleId(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    resource_def_id, name
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
                package_id,
                blueprint_name,
                function,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_FUNCTION PackageId(\"{}\") \"{}\" \"{}\"",
                    package_id, blueprint_name, function
                ));
                for arg in args {
                    let validated_arg = ValidatedData::from_slice(&arg)
                        .map_err(DecompileError::DataValidationError)?;
                    id_validator
                        .move_resources(&validated_arg)
                        .map_err(DecompileError::IdValidatorError)?;
                    buf.push(' ');
                    buf.push_str(&format_value(&validated_arg.dom, &buckets, &proofs));
                }
                buf.push_str(";\n");
            }
            Instruction::CallMethod {
                component_id,
                method,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_METHOD ComponentId(\"{}\") \"{}\"",
                    component_id, method
                ));
                for arg in args {
                    let validated_arg = ValidatedData::from_slice(&arg)
                        .map_err(DecompileError::DataValidationError)?;
                    id_validator
                        .move_resources(&validated_arg)
                        .map_err(DecompileError::IdValidatorError)?;
                    buf.push(' ');
                    buf.push_str(&format_value(&validated_arg.dom, &buckets, &proofs));
                }
                buf.push_str(";\n");
            }
            Instruction::CallMethodWithAllResources {
                component_id,
                method,
            } => {
                id_validator
                    .move_all_resources()
                    .map_err(DecompileError::IdValidatorError)?;
                buf.push_str(&format!(
                    "CALL_METHOD_WITH_ALL_RESOURCES ComponentId(\"{}\") \"{}\";\n",
                    component_id, method
                ));
            }
            Instruction::PublishPackage { code } => {
                buf.push_str(&format!(
                    "PUBLISH_PACKAGE Blob(\"{}\");\n",
                    base64::encode(&code)
                ));
            }
            Instruction::End { .. } => {}
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
        let tx = compile(include_str!("../examples/call.rtm")).unwrap();

        let manifest = &decompile(&tx).unwrap();
        println!("{}", manifest);

        assert_eq!(compile(manifest).unwrap(), tx);
    }
}
