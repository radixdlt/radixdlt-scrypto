use super::*;
use crate::types::{NodeId, PackageAddress};
use crate::*;
use sbor::rust::prelude::*;
use sbor::traversal::TerminalValueRef;
use sbor::*;

pub trait TypeInfoLookup {
    fn get_node_type_info(&self, node_id: &NodeId) -> Option<TypeInfoForValidation>;
    // radix-engine-common is not privy to specifics of engine blueprints
    // - so instead, we delegate this to the TypeInfoLookup implementation
    fn is_bucket(&self, type_info: &TypeInfoForValidation) -> bool;
    fn is_proof(&self, type_info: &TypeInfoForValidation) -> bool;
}

#[derive(Debug, Clone)]
pub enum TypeInfoForValidation {
    Object {
        package: PackageAddress,
        blueprint: String,
    },
    KeyValueStore,
    Index,
    SortedIndex,
}

pub struct ScryptoCustomValidationContext<'a> {
    type_info_lookup: Option<Box<dyn TypeInfoLookup + 'a>>,
}

impl<'a> ScryptoCustomValidationContext<'a> {
    pub fn new_without_type_info() -> Self {
        Self {
            type_info_lookup: None,
        }
    }

    pub fn new_with_type_info(type_info_lookup: Box<dyn TypeInfoLookup + 'a>) -> Self {
        Self {
            type_info_lookup: Some(type_info_lookup),
        }
    }
}

impl<'a> ValidatableCustomExtension<ScryptoCustomValidationContext<'a>> for ScryptoCustomExtension {
    fn apply_validation_for_custom_value<'de>(
        schema: &Schema<Self::CustomSchema>,
        custom_value: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        type_index: LocalTypeIndex,
        context: &ScryptoCustomValidationContext,
    ) -> Result<(), PayloadValidationError<Self>> {
        match schema
            .resolve_type_validation(type_index)
            .ok_or(PayloadValidationError::SchemaInconsistency)?
        {
            TypeValidation::None => Ok(()),
            TypeValidation::Custom(custom_validation) => {
                apply_custom_validation_to_custom_value(custom_validation, &custom_value.0, context)
            }
            _ => Err(PayloadValidationError::SchemaInconsistency),
        }
    }

    fn apply_custom_type_validation_for_non_custom_value<'de>(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomSchema as CustomSchema>::CustomTypeValidation,
        _: &TerminalValueRef<'de, Self::CustomTraversal>,
        _: &ScryptoCustomValidationContext,
    ) -> Result<(), PayloadValidationError<Self>> {
        // Non-custom values must have non-custom type kinds...
        // But custom type validations aren't allowed to combine with non-custom type kinds
        Err(PayloadValidationError::SchemaInconsistency)
    }
}

fn apply_custom_validation_to_custom_value(
    custom_validation: &ScryptoCustomTypeValidation,
    custom_value: &ScryptoCustomValue,
    context: &ScryptoCustomValidationContext,
) -> Result<(), PayloadValidationError<ScryptoCustomExtension>> {
    match custom_validation {
        ScryptoCustomTypeValidation::Reference(reference_validation) => {
            let ScryptoCustomValue::Reference(reference) = custom_value else {
                return Err(PayloadValidationError::SchemaInconsistency);
            };
            let node_id = reference.0;
            let type_info = resolve_type_info(&node_id, context)?;
            let is_valid = match &reference_validation {
                ReferenceValidation::IsGlobal => node_id.is_global(),
                ReferenceValidation::IsGlobalPackage => node_id.is_global_package(),
                ReferenceValidation::IsGlobalComponent => node_id.is_global_component(),
                ReferenceValidation::IsGlobalResource => node_id.is_global_resource(),
                ReferenceValidation::IsInternal => node_id.is_internal(),
                ReferenceValidation::IsTypedObject(expect_package, expect_blueprint) => {
                    match &type_info {
                        Some(TypeInfoForValidation::Object { package, blueprint }) => {
                            expect_package == package && expect_blueprint == blueprint
                        }
                        Some(_) => false,
                        // Didn't have a type info querier, so have to assume valid
                        None => true,
                    }
                }
            };
            if !is_valid {
                return Err(PayloadValidationError::ValidationError(
                    ValidationError::CustomError(format!(
                        "Expected = Reference<{:?}>, actual node: {:?}, resolved type info: {:?}",
                        reference_validation, node_id, type_info
                    )),
                ));
            }
        }
        ScryptoCustomTypeValidation::Own(own_validation) => {
            let ScryptoCustomValue::Own(own) = custom_value else {
                return Err(PayloadValidationError::SchemaInconsistency);
            };
            let node_id = own.0;
            let type_info = resolve_type_info(&node_id, context)?;
            let is_valid = match &type_info {
                Some(type_info) => {
                    // We have type info, so we must have the type_info_lookup
                    match own_validation {
                        OwnValidation::IsBucket => context
                            .type_info_lookup
                            .as_ref()
                            .unwrap()
                            .is_bucket(&type_info),
                        OwnValidation::IsProof => context
                            .type_info_lookup
                            .as_ref()
                            .unwrap()
                            .is_proof(&type_info),
                        OwnValidation::IsVault => node_id.is_internal_vault(),
                        OwnValidation::IsKeyValueStore => node_id.is_internal_kv_store(),
                        OwnValidation::IsTypedObject(expect_package, expect_blueprint) => {
                            match type_info {
                                TypeInfoForValidation::Object { package, blueprint } => {
                                    expect_package == package && expect_blueprint == blueprint
                                }
                                _ => false,
                            }
                        }
                    }
                }
                None => {
                    // Without a type info resolver, we do the best we can do statically
                    match own_validation {
                        OwnValidation::IsBucket => node_id.is_internal(),
                        OwnValidation::IsProof => node_id.is_internal(),
                        OwnValidation::IsVault => node_id.is_internal_vault(),
                        OwnValidation::IsKeyValueStore => node_id.is_internal_kv_store(),
                        OwnValidation::IsTypedObject(_, _) => true,
                    }
                }
            };
            if !is_valid {
                return Err(PayloadValidationError::ValidationError(
                    ValidationError::CustomError(format!(
                        "Expected = Own<{:?}>, actual node: {:?}, resolved type info: {:?}",
                        own_validation, node_id, type_info
                    )),
                ));
            }
        }
    };
    Ok(())
}

fn resolve_type_info(
    node_id: &NodeId,
    context: &ScryptoCustomValidationContext,
) -> Result<Option<TypeInfoForValidation>, PayloadValidationError<ScryptoCustomExtension>> {
    match &context.type_info_lookup {
        Some(querier) => match querier.get_node_type_info(node_id) {
            Some(type_info) => Ok(Some(type_info)),
            None => Err(PayloadValidationError::ValidationError(
                ValidationError::CustomError(format!(
                    "Node doesn't exist - could not lookup type info: {:?}",
                    node_id
                )),
            )),
        },
        None => Ok(None),
    }
}
