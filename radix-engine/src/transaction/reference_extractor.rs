use crate::types::*;
use transaction::model::*;

pub fn extract_refs_from_manifest(instructions: &[Instruction]) -> BTreeSet<Reference> {
    let mut references = BTreeSet::<Reference>::new();

    for instruction in instructions {
        extract_refs_from_instruction(&instruction, &mut references);
    }

    references.insert(RADIX_TOKEN.into());
    references.insert(PACKAGE_TOKEN.into());
    references.insert(EPOCH_MANAGER.into());
    references.insert(CLOCK.into());
    references.insert(ECDSA_SECP256K1_TOKEN.into());
    references.insert(EDDSA_ED25519_TOKEN.into());

    references
}

pub fn extract_refs_from_instruction(
    instruction: &Instruction,
    references: &mut BTreeSet<Reference>,
) {
    match instruction {
        Instruction::CallFunction {
            package_address,
            args,
            ..
        } => {
            references.insert(package_address.clone().into());
            extract_refs_from_value(&args, references);

            if package_address.eq(&EPOCH_MANAGER_PACKAGE) {
                references.insert(PACKAGE_TOKEN.clone().into());
            }
        }
        Instruction::CallMethod {
            component_address,
            args,
            ..
        } => {
            references.insert(component_address.clone().into());
            extract_refs_from_value(&args, references);
        }
        Instruction::MintUuidNonFungible {
            resource_address,
            args,
        } => {
            references.insert(resource_address.clone().into());
            extract_refs_from_value(&args, references);
        }
        Instruction::MintNonFungible {
            resource_address,
            args,
        } => {
            references.insert(resource_address.clone().into());
            extract_refs_from_value(&args, references);
        }
        Instruction::PublishPackage { .. } => {
            references.insert(PACKAGE_PACKAGE.clone().into());
        }
        Instruction::PublishPackageAdvanced { access_rules, .. } => {
            references.insert(PACKAGE_PACKAGE.clone().into());
            // TODO: Remove and cleanup
            let value: ManifestValue =
                manifest_decode(&manifest_encode(access_rules).unwrap()).unwrap();
            extract_refs_from_value(&value, references);
        }
        Instruction::SetMetadata {
            entity_address,
            value,
            ..
        } => {
            references.insert(entity_address.clone().into());
            // TODO: Remove and cleanup
            let value: ManifestValue = manifest_decode(&manifest_encode(value).unwrap()).unwrap();
            extract_refs_from_value(&value, references);
        }
        Instruction::RemoveMetadata { entity_address, .. } => {
            references.insert(entity_address.clone().into());
        }
        Instruction::SetMethodAccessRule {
            entity_address,
            rule,
            ..
        } => {
            references.insert(entity_address.clone().into());
            // TODO: Remove and cleanup
            let value: ManifestValue = manifest_decode(&manifest_encode(rule).unwrap()).unwrap();
            extract_refs_from_value(&value, references);
        }
        Instruction::RecallResource { vault_id, .. } => {
            // TODO: This needs to be cleaned up
            // TODO: How does this relate to newly created vaults in the transaction frame?
            // TODO: Will probably want different spacing for refed vs. owned nodes
            references.insert(vault_id.clone().into());
        }

        Instruction::SetPackageRoyaltyConfig {
            package_address, ..
        }
        | Instruction::ClaimPackageRoyalty {
            package_address, ..
        } => {
            references.insert(package_address.clone().into());
        }
        Instruction::SetComponentRoyaltyConfig {
            component_address, ..
        }
        | Instruction::ClaimComponentRoyalty {
            component_address, ..
        } => {
            references.insert(component_address.clone().into());
        }
        Instruction::TakeFromWorktop {
            resource_address, ..
        }
        | Instruction::TakeFromWorktopByAmount {
            resource_address, ..
        }
        | Instruction::TakeFromWorktopByIds {
            resource_address, ..
        }
        | Instruction::AssertWorktopContains {
            resource_address, ..
        }
        | Instruction::AssertWorktopContainsByAmount {
            resource_address, ..
        }
        | Instruction::AssertWorktopContainsByIds {
            resource_address, ..
        }
        | Instruction::CreateProofFromAuthZone {
            resource_address, ..
        }
        | Instruction::CreateProofFromAuthZoneByAmount {
            resource_address, ..
        }
        | Instruction::CreateProofFromAuthZoneByIds {
            resource_address, ..
        }
        | Instruction::MintFungible {
            resource_address, ..
        } => {
            references.insert(resource_address.clone().into());
        }
        Instruction::ReturnToWorktop { .. }
        | Instruction::PopFromAuthZone { .. }
        | Instruction::PushToAuthZone { .. }
        | Instruction::ClearAuthZone { .. }
        | Instruction::CreateProofFromBucket { .. }
        | Instruction::CloneProof { .. }
        | Instruction::DropProof { .. }
        | Instruction::DropAllProofs { .. }
        | Instruction::ClearSignatureProofs { .. }
        | Instruction::BurnResource { .. } => {}
    }
}

pub fn extract_refs_from_value(value: &ManifestValue, references: &mut BTreeSet<Reference>) {
    match value {
        Value::Bool { .. }
        | Value::I8 { .. }
        | Value::I16 { .. }
        | Value::I32 { .. }
        | Value::I64 { .. }
        | Value::I128 { .. }
        | Value::U8 { .. }
        | Value::U16 { .. }
        | Value::U32 { .. }
        | Value::U64 { .. }
        | Value::U128 { .. }
        | Value::String { .. } => {}
        Value::Enum { fields, .. } => {
            for f in fields {
                extract_refs_from_value(f, references);
            }
        }
        Value::Array { elements, .. } => {
            for f in elements {
                extract_refs_from_value(f, references);
            }
        }
        Value::Tuple { fields } => {
            for f in fields {
                extract_refs_from_value(f, references);
            }
        }
        Value::Map { entries, .. } => {
            for f in entries {
                extract_refs_from_value(&f.0, references);
                extract_refs_from_value(&f.1, references);
            }
        }
        Value::Custom { value } => match value {
            ManifestCustomValue::Address(a) => {
                references.insert(Reference(a.0.clone()));
            }
            _ => {}
        },
    }
}
