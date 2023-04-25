use super::*;
use crate::constants::*;
use crate::types::{GlobalAddress, NodeId, PackageAddress};
use crate::*;
use sbor::rust::prelude::*;
use sbor::*;

impl ValidatableCustomTypeExtension<()> for ScryptoCustomTypeExtension {
    fn validate_custom_value<'de, L: SchemaTypeLink>(
        _custom_value_ref: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        _custom_type_kind: &Self::CustomTypeKind<L>,
        _context: &mut (),
    ) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// A slightly duplicated model to break circular dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInfo {
    Object {
        package_address: PackageAddress,
        blueprint_name: String,
        global: bool,
        type_parent: Option<GlobalAddress>,
    },
    KeyValueStore,
    SortedStore,
}

pub trait NodeTypeInfoContext {
    fn get_node_type_info(&mut self, reference: &NodeId) -> Option<TypeInfo>;
}

impl<T> ValidatableCustomTypeExtension<T> for ScryptoCustomTypeExtension
where
    T: NodeTypeInfoContext,
{
    fn validate_custom_value<'de, L: SchemaTypeLink>(
        custom_value_ref: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        custom_type_kind: &Self::CustomTypeKind<L>,
        context: &mut T,
    ) -> Result<(), ValidationError> {
        match &custom_value_ref.0 {
            ScryptoCustomValue::Reference(reference) => {
                if let Some(type_info) = context.get_node_type_info(reference.as_node_id()) {
                    // Alternately, we can just use the type info for reference check.
                    // Using node entity type is to avoid massive list of blueprints for each type.
                    if match custom_type_kind {
                        ScryptoCustomTypeKind::Reference => true,
                        ScryptoCustomTypeKind::GlobalAddress => reference.as_node_id().is_global(),
                        ScryptoCustomTypeKind::LocalAddress => reference.as_node_id().is_local(),
                        ScryptoCustomTypeKind::PackageAddress => reference.as_node_id().is_global_package(),
                        ScryptoCustomTypeKind::ComponentAddress => reference.as_node_id().is_global_component(),
                        ScryptoCustomTypeKind::ResourceAddress => reference.as_node_id().is_global_resource(),
                        ScryptoCustomTypeKind::ObjectRef(package, blueprint) => match &type_info {
                            TypeInfo::Object { package_address, blueprint_name,.. }
                                if package_address == package
                                &&  blueprint_name == blueprint => true,
                            _ => false,
                        }
                        ScryptoCustomTypeKind::Own |
                        ScryptoCustomTypeKind::Bucket |
                        ScryptoCustomTypeKind::Proof |
                        ScryptoCustomTypeKind::Vault |
                        ScryptoCustomTypeKind::KeyValueStore |
                        ScryptoCustomTypeKind::Object(_, _) |
                        ScryptoCustomTypeKind::Decimal |
                        ScryptoCustomTypeKind::PreciseDecimal |
                        ScryptoCustomTypeKind::NonFungibleLocalId  => panic!("Non-reference type matched with reference value; please check `custom_type_kind_matches_value_kind` ")
                    } {
                        Ok(())
                    } else {
                        Err(ValidationError::CustomError(format!(
                            "Invalid reference: expected = {:?}, actual = {:?} of type {:?}", custom_type_kind, reference.as_node_id(),  type_info
                        )))
                    }
                } else {
                    Err(ValidationError::CustomError(format!(
                        "Missing type info for {:?}",
                        reference
                    )))
                }
            }
            ScryptoCustomValue::Own(own) => {
                if let Some(type_info) = context.get_node_type_info(own.as_node_id()) {
                    if match  custom_type_kind {
                        ScryptoCustomTypeKind::Own => true,
                        ScryptoCustomTypeKind::Bucket => match &type_info {
                            TypeInfo::Object { package_address, blueprint_name,.. }
                                if package_address == &RESOURCE_MANAGER_PACKAGE
                                && (blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT || blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT) => true,
                            _ => false,
                        }
                        ScryptoCustomTypeKind::Proof => match &type_info {
                            TypeInfo::Object { package_address, blueprint_name,.. }
                                if package_address == &RESOURCE_MANAGER_PACKAGE
                                    &&  blueprint_name == PROOF_BLUEPRINT => true,
                            _ => false,
                        }
                        ScryptoCustomTypeKind::Vault => match &type_info {
                            TypeInfo::Object { package_address, blueprint_name,.. }
                                if package_address == &RESOURCE_MANAGER_PACKAGE
                                && (blueprint_name == FUNGIBLE_VAULT_BLUEPRINT || blueprint_name == NON_FUNGIBLE_VAULT_BLUEPRINT) => true,
                            _ => false,
                        }
                        ScryptoCustomTypeKind::KeyValueStore => match &type_info {
                            TypeInfo::KeyValueStore => true,
                            _ => false,
                        }
                        ScryptoCustomTypeKind::Object(package, blueprint) => match &type_info {
                            TypeInfo::Object { package_address, blueprint_name,.. }
                                if package_address == package
                                &&  blueprint_name == blueprint => true,
                            _ => false,
                        }
                        ScryptoCustomTypeKind::Reference |
                        ScryptoCustomTypeKind::GlobalAddress |
                        ScryptoCustomTypeKind::LocalAddress |
                        ScryptoCustomTypeKind::PackageAddress |
                        ScryptoCustomTypeKind::ComponentAddress |
                        ScryptoCustomTypeKind::ResourceAddress |
                        ScryptoCustomTypeKind::ObjectRef(_, _) |
                        ScryptoCustomTypeKind::Decimal |
                        ScryptoCustomTypeKind::PreciseDecimal |
                        ScryptoCustomTypeKind::NonFungibleLocalId  => panic!("Non-own type matched with own value; please check `custom_type_kind_matches_value_kind` ")
                    } {
                        Ok(())
                    } else {
                        Err(ValidationError::CustomError(format!(
                            "Invalid own: expected = {:?}, actual = {:?} of type {:?}", custom_type_kind, own.as_node_id(),  type_info
                        )))
                    }
                } else {
                    Err(ValidationError::CustomError(format!(
                        "Missing type info for {:?}",
                        own
                    )))
                }
            }
            ScryptoCustomValue::Decimal(_)
            | ScryptoCustomValue::PreciseDecimal(_)
            | ScryptoCustomValue::NonFungibleLocalId(_) => Ok(()),
        }
    }
}
