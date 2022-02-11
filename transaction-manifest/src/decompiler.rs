use radix_engine::engine::*;
use radix_engine::model::*;
use scrypto::rust::collections::*;
use scrypto::types::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    IdValidatorError(IdValidatorError),
    DataValidationError(DataValidationError),
}

pub fn decompile(tx: &Transaction) -> Result<String, DecompileError> {
    let mut buf = String::new();
    let mut id_validator = IdValidator::new();
    let mut buckets = HashMap::<Bid, String>::new();
    let mut bucket_refs = HashMap::<Rid, String>::new();
    for inst in &tx.instructions {
        match inst.clone() {
            Instruction::TakeFromWorktop {
                amount,
                resource_address,
            } => {
                let bid = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bid, name.clone());
                buf.push_str(&format!(
                    "TAKE_FROM_WORKTOP Decimal(\"{}\") Address(\"{}\") Bucket(\"{}\");\n",
                    amount, resource_address, name
                ));
            }
            Instruction::TakeAllFromWorktop { resource_address } => {
                let bid = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bid, name.clone());
                buf.push_str(&format!(
                    "TAKE_ALL_FROM_WORKTOP Address(\"{}\") Bucket(\"{}\");\n",
                    resource_address, name
                ));
            }
            Instruction::TakeNonFungiblesFromWorktop {
                keys,
                resource_address,
            } => {
                let bid = id_validator
                    .new_bucket()
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("bucket{}", buckets.len() + 1);
                buckets.insert(bid, name.clone());
                buf.push_str(&format!(
                    "TAKE_NON_FUNGIBLES_FROM_WORKTOP TreeSet<NonFungibleKey>({}) Address(\"{}\") Bucket(\"{}\");\n",
                    keys.iter()
                    .map(|k| format!("NonFungibleKey(\"{}\")", k))
                    .collect::<Vec<String>>()
                    .join(", "),
                    resource_address, name
                ));
            }
            Instruction::ReturnToWorktop { bid } => {
                id_validator
                    .drop_bucket(bid)
                    .map_err(DecompileError::IdValidatorError)?;
                buf.push_str(&format!(
                    "RETURN_TO_WORKTOP Bucket({});\n",
                    buckets
                        .get(&bid)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", bid.0))
                ));
            }
            Instruction::AssertWorktopContains {
                amount,
                resource_address,
            } => {
                buf.push_str(&format!(
                    "ASSERT_WORKTOP_CONTAINS Decimal(\"{}\") Address(\"{}\");\n",
                    amount, resource_address
                ));
            }
            Instruction::CreateBucketRef { bid } => {
                let rid = id_validator
                    .new_bucket_ref(bid)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("badge{}", bucket_refs.len() + 1);
                bucket_refs.insert(rid, name.clone());
                buf.push_str(&format!(
                    "CREATE_BUCKET_REF Bucket({}) BucketRef(\"{}\");\n",
                    buckets
                        .get(&bid)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", bid.0)),
                    name
                ));
            }
            Instruction::CloneBucketRef { rid } => {
                let rid2 = id_validator
                    .clone_bucket_ref(rid)
                    .map_err(DecompileError::IdValidatorError)?;
                let name = format!("badge{}", bucket_refs.len() + 1);
                bucket_refs.insert(rid2, name.clone());
                buf.push_str(&format!(
                    "CLONE_BUCKET_REF BucketRef({}) BucketRef(\"{}\");\n",
                    bucket_refs
                        .get(&rid)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", rid.0)),
                    name
                ));
            }
            Instruction::DropBucketRef { rid } => {
                id_validator
                    .drop_bucket_ref(rid)
                    .map_err(DecompileError::IdValidatorError)?;
                buf.push_str(&format!(
                    "DROP_BUCKET_REF BucketRef({});\n",
                    bucket_refs
                        .get(&rid)
                        .map(|name| format!("\"{}\"", name))
                        .unwrap_or(format!("{}u32", rid.0)),
                ));
            }
            Instruction::CallFunction {
                package_address,
                blueprint_name,
                function,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_FUNCTION Address(\"{}\") \"{}\" \"{}\"",
                    package_address, blueprint_name, function
                ));
                for arg in args {
                    let validated_arg =
                        validate_data(&arg).map_err(DecompileError::DataValidationError)?;
                    id_validator
                        .move_resources(&validated_arg)
                        .map_err(DecompileError::IdValidatorError)?;
                    buf.push(' ');
                    buf.push_str(&format_value(&validated_arg.dom, &buckets, &bucket_refs));
                }
                buf.push_str(";\n");
            }
            Instruction::CallMethod {
                component_address,
                method,
                args,
            } => {
                buf.push_str(&format!(
                    "CALL_METHOD Address(\"{}\") \"{}\"",
                    component_address, method
                ));
                for arg in args {
                    let validated_arg =
                        validate_data(&arg).map_err(DecompileError::DataValidationError)?;
                    id_validator
                        .move_resources(&validated_arg)
                        .map_err(DecompileError::IdValidatorError)?;
                    buf.push(' ');
                    buf.push_str(&format_value(&validated_arg.dom, &buckets, &bucket_refs));
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
                    "CALL_METHOD_WITH_ALL_RESOURCES Address(\"{}\") \"{}\";\n",
                    component_address, method
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
