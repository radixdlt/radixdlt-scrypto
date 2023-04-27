use super::*;
use crate::constants::*;
use crate::types::{GlobalAddress, NodeId, PackageAddress};
use crate::*;
use sbor::rust::prelude::*;
use sbor::traversal::TerminalValueRef;
use sbor::*;

impl ValidatableCustomTypeExtension<()> for ScryptoCustomTypeExtension {
    fn apply_custom_type_validation<'de>(
        _: &Self::CustomTypeValidation,
        _: &TerminalValueRef<'de, Self::CustomTraversal>,
        _: &mut (),
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
    Index,
    SortedIndex,
}

pub trait NodeTypeInfoContext {
    fn get_node_type_info(&mut self, reference: &NodeId) -> Option<TypeInfo>;
}

impl<T> ValidatableCustomTypeExtension<T> for ScryptoCustomTypeExtension
where
    T: NodeTypeInfoContext,
{
    fn apply_custom_type_validation<'de>(
        custom_type_validation: &Self::CustomTypeValidation,
        value: &TerminalValueRef<'de, Self::CustomTraversal>,
        context: &mut T,
    ) -> Result<(), ValidationError> {
        match custom_type_validation {
            ScryptoCustomTypeValidation::Reference(validation) => {
                if let TerminalValueRef::Custom(ScryptoCustomTerminalValueRef(
                    ScryptoCustomValue::Reference(reference),
                )) = value
                {
                    if let Some(type_info) = context.get_node_type_info(reference.as_node_id()) {
                        // Alternately, we can just use the type info for reference check.
                        // Using node entity type is to avoid massive list of blueprints for each type.
                        if match validation {
                            ReferenceValidation::IsGlobal => reference.as_node_id().is_global(),
                            ReferenceValidation::IsGlobalPackage => {
                                reference.as_node_id().is_global_package()
                            }
                            ReferenceValidation::IsGlobalComponent => {
                                reference.as_node_id().is_global_component()
                            }
                            ReferenceValidation::IsGlobalResource => {
                                reference.as_node_id().is_global_resource()
                            }
                            ReferenceValidation::IsLocal => reference.as_node_id().is_local(),
                            ReferenceValidation::IsTypedObject(package, blueprint) => {
                                match &type_info {
                                    TypeInfo::Object {
                                        package_address,
                                        blueprint_name,
                                        ..
                                    } if package_address == package
                                        && blueprint_name == blueprint =>
                                    {
                                        true
                                    }
                                    _ => false,
                                }
                            }
                        } {
                            Ok(())
                        } else {
                            Err(ValidationError::CustomError(format!(
                                "Expected: Reference<{:?}>, actual node: {:?}, actual type info: {:?}", validation, reference.as_node_id(),  type_info
                            )))
                        }
                    } else {
                        Err(ValidationError::CustomError(format!(
                            "Missing type info for {:?}",
                            reference
                        )))
                    }
                } else {
                    Err(ValidationError::CustomError(format!(
                        "Expected: Reference<{:?}>, actual value: {:?}",
                        validation, value
                    )))
                }
            }
            ScryptoCustomTypeValidation::Own(validation) => {
                if let TerminalValueRef::Custom(ScryptoCustomTerminalValueRef(
                    ScryptoCustomValue::Own(own),
                )) = value
                {
                    if let Some(type_info) = context.get_node_type_info(own.as_node_id()) {
                        if match validation {
                            OwnValidation::IsBucket => match &type_info {
                                TypeInfo::Object {
                                    package_address,
                                    blueprint_name,
                                    ..
                                } if package_address == &RESOURCE_MANAGER_PACKAGE
                                    && (blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                                        || blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT) =>
                                {
                                    true
                                }
                                _ => false,
                            },
                            OwnValidation::IsProof => match &type_info {
                                TypeInfo::Object {
                                    package_address,
                                    blueprint_name,
                                    ..
                                } if package_address == &RESOURCE_MANAGER_PACKAGE
                                    && blueprint_name == PROOF_BLUEPRINT =>
                                {
                                    true
                                }
                                _ => false,
                            },
                            OwnValidation::IsVault => match &type_info {
                                TypeInfo::Object {
                                    package_address,
                                    blueprint_name,
                                    ..
                                } if package_address == &RESOURCE_MANAGER_PACKAGE
                                    && (blueprint_name == FUNGIBLE_VAULT_BLUEPRINT
                                        || blueprint_name == NON_FUNGIBLE_VAULT_BLUEPRINT) =>
                                {
                                    true
                                }
                                _ => false,
                            },
                            OwnValidation::IsKeyValueStore => match &type_info {
                                TypeInfo::KeyValueStore => true,
                                _ => false,
                            },
                            OwnValidation::IsTypedObject(package, blueprint) => match &type_info {
                                TypeInfo::Object {
                                    package_address,
                                    blueprint_name,
                                    ..
                                } if package_address == package && blueprint_name == blueprint => {
                                    true
                                }
                                _ => false,
                            },
                        } {
                            Ok(())
                        } else {
                            Err(ValidationError::CustomError(format!(
                                "Expected = Own<{:?}>, actual node: {:?}, actual type info: {:?}",
                                validation,
                                own.as_node_id(),
                                type_info
                            )))
                        }
                    } else {
                        Err(ValidationError::CustomError(format!(
                            "Missing type info for {:?}",
                            own
                        )))
                    }
                } else {
                    Err(ValidationError::CustomError(format!(
                        "Expected: Own<{:?}>, actual value: {:?}",
                        validation, value
                    )))
                }
            }
        }
    }
}
