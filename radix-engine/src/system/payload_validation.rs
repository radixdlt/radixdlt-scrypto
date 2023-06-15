use crate::kernel::kernel_api::KernelApi;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT,
    NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use radix_engine_interface::constants::*;
use sbor::rust::prelude::*;
use sbor::traversal::TerminalValueRef;

use super::node_modules::type_info::TypeInfoSubstate;
use super::system::SystemService;
use super::system_callback::SystemConfig;
use super::system_callback_api::SystemCallbackObject;

//=======================================================================================================
// NOTE:
// The validation implemented here makes use of a type info lookup to provide tighter validation
// There is also static validation defined in the `radix-engine-common` repository which is less
// powerful, does not require this lookup.
//=======================================================================================================

//==================
// TRAITS
//==================

/// We use a trait here so it can be implemented either by the System API (mid-execution) or by off-ledger systems
pub trait TypeInfoLookup {
    fn get_node_type_info(&self, node_id: &NodeId) -> Option<TypeInfoForValidation>;

    fn schema_origin(&self) -> &SchemaOrigin;
}

#[derive(Debug, Clone)]
pub enum SchemaOrigin {
    Blueprint(BlueprintId),
    Instance,
    KeyValueStore,
}

//==================
// SYSTEM ADAPTERS
//==================

pub struct SystemServiceTypeInfoLookup<
    's,
    'a,
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
> {
    system_service: RefCell<&'s mut SystemService<'a, Y, V>>,
    schema_origin: SchemaOrigin,
}

impl<'s, 'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>
    SystemServiceTypeInfoLookup<'s, 'a, Y, V>
{
    pub fn new(
        system_service: &'s mut SystemService<'a, Y, V>,
        schema_origin: SchemaOrigin,
    ) -> Self {
        Self {
            system_service: system_service.into(),
            schema_origin,
        }
    }
}

impl<'s, 'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject> TypeInfoLookup
    for SystemServiceTypeInfoLookup<'s, 'a, Y, V>
{
    fn get_node_type_info(&self, node_id: &NodeId) -> Option<TypeInfoForValidation> {
        let type_info = self
            .system_service
            .borrow_mut()
            .get_node_type_info(&node_id)?;
        let mapped = match type_info {
            TypeInfoSubstate::Object(ObjectInfo { blueprint_id, .. }) => {
                TypeInfoForValidation::Object {
                    package: blueprint_id.package_address,
                    blueprint: blueprint_id.blueprint_name,
                }
            }
            TypeInfoSubstate::KeyValueStore(_) => TypeInfoForValidation::KeyValueStore,
            TypeInfoSubstate::GlobalAddressReservation(_) => {
                TypeInfoForValidation::GlobalAddressReservation
            }
            TypeInfoSubstate::GlobalAddressPhantom(info) => TypeInfoForValidation::Object {
                package: info.blueprint_id.package_address,
                blueprint: info.blueprint_id.blueprint_name,
            },
        };
        Some(mapped)
    }

    fn schema_origin(&self) -> &SchemaOrigin {
        &self.schema_origin
    }
}

#[derive(Debug, Clone)]
pub enum TypeInfoForValidation {
    Object {
        package: PackageAddress,
        blueprint: String,
    },
    KeyValueStore,
    GlobalAddressReservation,
}

impl TypeInfoForValidation {
    fn matches(&self, expected_package: &PackageAddress, expected_blueprint: &str) -> bool {
        matches!(
            self,
            TypeInfoForValidation::Object { package, blueprint }
                if package == expected_package && blueprint == expected_blueprint
        )
    }

    fn matches_with_origin(
        &self,
        expected_package: &Option<PackageAddress>,
        expected_blueprint: &str,
        schema_origin: &SchemaOrigin,
    ) -> bool {
        match expected_package {
            Some(package_address) => self.matches(package_address, expected_blueprint),
            None => match schema_origin {
                SchemaOrigin::Blueprint(blueprint_id) => {
                    self.matches(&blueprint_id.package_address, expected_blueprint)
                }
                SchemaOrigin::Instance | SchemaOrigin::KeyValueStore => false,
            },
        }
    }
}

//==================
// VALIDATION
//==================

type Lookup<'a> = Box<dyn TypeInfoLookup + 'a>;

impl<'a> ValidatableCustomExtension<Lookup<'a>> for ScryptoCustomExtension {
    fn apply_validation_for_custom_value<'de>(
        schema: &Schema<Self::CustomSchema>,
        custom_value: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        type_index: LocalTypeIndex,
        context: &Lookup<'a>,
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
        _: &Lookup<'a>,
    ) -> Result<(), PayloadValidationError<Self>> {
        // Non-custom values must have non-custom type kinds...
        // But custom type validations aren't allowed to combine with non-custom type kinds
        Err(PayloadValidationError::SchemaInconsistency)
    }
}

fn apply_custom_validation_to_custom_value(
    custom_validation: &ScryptoCustomTypeValidation,
    custom_value: &ScryptoCustomValue,
    lookup: &Lookup,
) -> Result<(), PayloadValidationError<ScryptoCustomExtension>> {
    match custom_validation {
        ScryptoCustomTypeValidation::Reference(reference_validation) => {
            let ScryptoCustomValue::Reference(reference) = custom_value else {
                return Err(PayloadValidationError::SchemaInconsistency);
            };
            let node_id = reference.0;
            let type_info = resolve_type_info(&node_id, lookup)?;
            let is_valid = match &reference_validation {
                ReferenceValidation::IsGlobal => node_id.is_global(),
                ReferenceValidation::IsGlobalPackage => node_id.is_global_package(),
                ReferenceValidation::IsGlobalComponent => node_id.is_global_component(),
                ReferenceValidation::IsGlobalResourceManager => {
                    node_id.is_global_resource_manager()
                }
                ReferenceValidation::IsGlobalTyped(expected_package, expected_blueprint) => {
                    node_id.is_global()
                        && type_info.matches_with_origin(
                            expected_package,
                            expected_blueprint,
                            lookup.schema_origin(),
                        )
                }
                ReferenceValidation::IsInternal => node_id.is_internal(),
                ReferenceValidation::IsInternalTyped(expected_package, expected_blueprint) => {
                    node_id.is_global()
                        && type_info.matches_with_origin(
                            expected_package,
                            expected_blueprint,
                            lookup.schema_origin(),
                        )
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
            let type_info = resolve_type_info(&node_id, lookup)?;
            let is_valid = match own_validation {
                OwnValidation::IsBucket => {
                    type_info.matches(&RESOURCE_PACKAGE, FUNGIBLE_BUCKET_BLUEPRINT)
                        || type_info.matches(&RESOURCE_PACKAGE, NON_FUNGIBLE_BUCKET_BLUEPRINT)
                }
                OwnValidation::IsProof => {
                    type_info.matches(&RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT)
                        || type_info.matches(&RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT)
                }
                OwnValidation::IsVault => node_id.is_internal_vault(),
                OwnValidation::IsKeyValueStore => node_id.is_internal_kv_store(),
                OwnValidation::IsGlobalAddressReservation => {
                    matches!(type_info, TypeInfoForValidation::GlobalAddressReservation)
                }
                OwnValidation::IsTypedObject(expected_package, expected_blueprint) => type_info
                    .matches_with_origin(
                        expected_package,
                        expected_blueprint,
                        lookup.schema_origin(),
                    ),
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
    lookup: &Lookup,
) -> Result<TypeInfoForValidation, PayloadValidationError<ScryptoCustomExtension>> {
    lookup.get_node_type_info(node_id).ok_or_else(|| {
        PayloadValidationError::ValidationError(ValidationError::CustomError(format!(
            "Node doesn't exist - could not lookup type info: {:?}",
            node_id
        )))
    })
}
