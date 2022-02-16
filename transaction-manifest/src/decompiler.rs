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
            Instruction::TakeFromWorktop {
                amount,
                resource_def_ref,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP Decimal(\"{}\") ResourceDefRef(\"{}\") Bucket(\"{}\");\n",
                    amount, resource_def_ref, name
                ));
            }
            Instruction::TakeAllFromWorktop { resource_def_ref } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_ALL_FROM_WORKTOP ResourceDefRef(\"{}\") Bucket(\"{}\");\n",
                    resource_def_ref, name
                ));
            }
            Instruction::TakeNonFungiblesFromWorktop {
                keys,
                resource_def_ref,
            } => {
                let bucket_id = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bucket_id, name.clone());
                buf.push_str(&format!(
                    "TAKE_NON_FUNGIBLES_FROM_WORKTOP TreeSet<NonFungibleKey>({}) ResourceDefRef(\"{}\") Bucket(\"{}\");\n",
                    keys.iter()
                    .map(|k| format!("NonFungibleKey(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    resource_def_ref, name
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
            Instruction::AssertWorktopContains {
                amount,
                resource_def_ref,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS Decimal(\"{}\") ResourceDefRef(\"{}\");\n",
                    amount, resource_def_ref
                ));
            }
            Instruction::CreateProof { bucket_id } => {
                let proof_id = id_validator
                    .new_proof(bucket_id)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("badge{}", proofs.len() + 1);
                proofs.insert(proof_id, name.clone());
                buf.push_str(&format!(
                    "CREATE_PROOF Bucket({}) Proof(\"{}\");\n",
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
                let name = format!("badge{}", proofs.len() + 1);
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
                package_ref,
                blueprint_name,
                function,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_FUNCTION PackageRef(\"{}\") \"{}\" \"{}\"",
                    package_ref, blueprint_name, function
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
                component_ref,
                method,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_METHOD ComponentRef(\"{}\") \"{}\"",
                    component_ref, method
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
                component_ref,
                method,
            } => {
                id_validator
                    .move_all_resources()
                    .map_err(DecompileError::IdValidatorError)?;
                buf.push_str(&format!(
                    "CALL_METHOD_WITH_ALL_RESOURCES ComponentRef(\"{}\") \"{}\";\n",
                    component_ref, method
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
