use crate::types::*;
use transaction::data::to_address;
use transaction::model::*;

pub fn extract_refs_from_manifest(
    instructions: &[Instruction],
) -> (BTreeSet<Address>, BTreeSet<InternalRef>) {
    let mut global_references = BTreeSet::new();
    let mut local_references = BTreeSet::new();

    for instruction in instructions {
        extract_refs_from_instruction(&instruction, &mut global_references, &mut local_references);
    }

    global_references.insert(RADIX_TOKEN.into());
    global_references.insert(PACKAGE_TOKEN.into());
    global_references.insert(EPOCH_MANAGER.into());
    global_references.insert(CLOCK.into());
    global_references.insert(ECDSA_SECP256K1_TOKEN.into());
    global_references.insert(EDDSA_ED25519_TOKEN.into());

    (global_references, local_references)
}

pub fn extract_refs_from_instruction(
    instruction: &Instruction,
    global_references: &mut BTreeSet<Address>,
    local_references: &mut BTreeSet<InternalRef>,
) {
    match instruction {
        Instruction::CallFunction {
            package_address,
            args,
            ..
        } => {
            global_references.insert(package_address.clone().into());
            extract_refs_from_value(&args, global_references, local_references);

            if package_address.eq(&EPOCH_MANAGER_PACKAGE) {
                global_references.insert(PACKAGE_TOKEN.clone().into());
            }
        }
        Instruction::CallMethod {
            component_address,
            args,
            ..
        } => {
            global_references.insert(component_address.clone().into());
            extract_refs_from_value(&args, global_references, local_references);
        }
        Instruction::MintUuidNonFungible {
            resource_address,
            args,
        } => {
            global_references.insert(resource_address.clone().into());
            extract_refs_from_value(&args, global_references, local_references);
        }
        Instruction::MintNonFungible {
            resource_address,
            args,
        } => {
            global_references.insert(resource_address.clone().into());
            extract_refs_from_value(&args, global_references, local_references);
        }
        Instruction::PublishPackageAdvanced { access_rules, .. } => {
            global_references.insert(PACKAGE_PACKAGE.clone().into());
            // TODO: Remove and cleanup
            let value: ManifestValue =
                manifest_decode(&manifest_encode(access_rules).unwrap()).unwrap();
            extract_refs_from_value(&value, global_references, local_references);
        }
        Instruction::SetMetadata {
            entity_address,
            value,
            ..
        } => {
            global_references.insert(to_address(entity_address.clone()).into());
            // TODO: Remove and cleanup
            let value: ManifestValue = manifest_decode(&manifest_encode(value).unwrap()).unwrap();
            extract_refs_from_value(&value, global_references, local_references);
        }
        Instruction::RemoveMetadata { entity_address, .. }
        | Instruction::SetMethodAccessRule { entity_address, .. } => {
            global_references.insert(to_address(entity_address.clone()).into());
        }
        Instruction::RecallResource { vault_id, .. } => {
            // TODO: This needs to be cleaned up
            // TODO: How does this relate to newly created vaults in the transaction frame?
            // TODO: Will probably want different spacing for refed vs. owned nodes
            local_references.insert(InternalRef(vault_id.clone()));
        }

        Instruction::SetPackageRoyaltyConfig {
            package_address, ..
        }
        | Instruction::ClaimPackageRoyalty {
            package_address, ..
        } => {
            global_references.insert(package_address.clone().into());
        }
        Instruction::SetComponentRoyaltyConfig {
            component_address, ..
        }
        | Instruction::ClaimComponentRoyalty {
            component_address, ..
        } => {
            global_references.insert(component_address.clone().into());
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
            global_references.insert(resource_address.clone().into());
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
        | Instruction::BurnResource { .. }
        | Instruction::AssertAccessRule { .. } => {}
    }
}

pub fn extract_refs_from_value(
    value: &ManifestValue,
    global_references: &mut BTreeSet<Address>,
    local_references: &mut BTreeSet<InternalRef>,
) {
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
                extract_refs_from_value(f, global_references, local_references);
            }
        }
        Value::Array { elements, .. } => {
            for f in elements {
                extract_refs_from_value(f, global_references, local_references);
            }
        }
        Value::Tuple { fields } => {
            for f in fields {
                extract_refs_from_value(f, global_references, local_references);
            }
        }
        Value::Map { entries, .. } => {
            for f in entries {
                extract_refs_from_value(&f.0, global_references, local_references);
                extract_refs_from_value(&f.1, global_references, local_references);
            }
        }
        Value::Custom { value } => match value {
            ManifestCustomValue::Address(a) => {
                global_references.insert(to_address(a.clone()));
            }
            _ => {}
        },
    }
}
