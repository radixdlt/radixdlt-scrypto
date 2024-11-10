use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use radix_common::constants::*;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT,
    NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use sbor::rust::prelude::*;
use sbor::traversal::TerminalValueRef;

use super::system::SystemService;
use super::system_callback::*;
use super::type_info::TypeInfoSubstate;

//=======================================================================================================
// NOTE:
// The validation implemented here makes use of a type info lookup to provide tighter validation
// There is also static validation defined in the `radix-common` repository which is less
// powerful, does not require this lookup.
//=======================================================================================================

//==================
// TRAITS
//==================

/// We use a trait here so it can be implemented either by the System API (mid-execution) or by off-ledger systems
pub trait ValidationContext {
    type Error;

    fn get_node_type_info(&self, node_id: &NodeId) -> Result<TypeInfoForValidation, Self::Error>;

    fn schema_origin(&self) -> &SchemaOrigin;

    fn allow_ownership(&self) -> bool;

    fn allow_non_global_ref(&self) -> bool;
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

pub struct SystemServiceTypeInfoLookup<'s, 'a, Y: SystemBasedKernelApi> {
    system_service: RefCell<&'s mut SystemService<'a, Y>>,
    schema_origin: SchemaOrigin,
    allow_ownership: bool,
    allow_non_global_ref: bool,
}

impl<'s, 'a, Y: SystemBasedKernelApi> SystemServiceTypeInfoLookup<'s, 'a, Y> {
    pub fn new(
        system_service: &'s mut SystemService<'a, Y>,
        schema_origin: SchemaOrigin,
        allow_ownership: bool,
        allow_non_global_ref: bool,
    ) -> Self {
        Self {
            system_service: system_service.into(),
            schema_origin,
            allow_ownership,
            allow_non_global_ref,
        }
    }
}

impl<'s, 'a, Y: SystemBasedKernelApi> ValidationContext for SystemServiceTypeInfoLookup<'s, 'a, Y> {
    type Error = RuntimeError;

    fn get_node_type_info(&self, node_id: &NodeId) -> Result<TypeInfoForValidation, RuntimeError> {
        let type_info = self
            .system_service
            .borrow_mut()
            .get_node_type_info(&node_id)?;
        let mapped = match type_info {
            TypeInfoSubstate::Object(ObjectInfo {
                blueprint_info: BlueprintInfo { blueprint_id, .. },
                ..
            }) => TypeInfoForValidation::Object {
                package: blueprint_id.package_address,
                blueprint: blueprint_id.blueprint_name,
            },
            TypeInfoSubstate::KeyValueStore(_) => TypeInfoForValidation::KeyValueStore,
            TypeInfoSubstate::GlobalAddressReservation(_) => {
                TypeInfoForValidation::GlobalAddressReservation
            }
            TypeInfoSubstate::GlobalAddressPhantom(info) => TypeInfoForValidation::Object {
                package: info.blueprint_id.package_address,
                blueprint: info.blueprint_id.blueprint_name,
            },
        };
        Ok(mapped)
    }

    fn schema_origin(&self) -> &SchemaOrigin {
        &self.schema_origin
    }

    fn allow_ownership(&self) -> bool {
        self.allow_ownership
    }

    fn allow_non_global_ref(&self) -> bool {
        self.allow_non_global_ref
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

type Lookup<'a, E> = Box<dyn ValidationContext<Error = E> + 'a>;

impl<'a, E: Debug> ValidatableCustomExtension<Lookup<'a, E>> for ScryptoCustomExtension {
    fn apply_validation_for_custom_value<'de>(
        schema: &Schema<Self::CustomSchema>,
        custom_value: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        type_id: LocalTypeId,
        context: &Lookup<'a, E>,
    ) -> Result<(), PayloadValidationError<Self>> {
        match &custom_value.0 {
            ScryptoCustomValue::Own(..) => {
                if !context.allow_ownership() {
                    return Err(PayloadValidationError::ValidationError(
                        ValidationError::CustomError(format!("Ownership is not allowed")),
                    ));
                }
            }
            ScryptoCustomValue::Reference(reference) => {
                if !reference.0.is_global() {
                    if !context.allow_non_global_ref() {
                        return Err(PayloadValidationError::ValidationError(
                            ValidationError::CustomError(format!(
                                "Non Global Reference is not allowed"
                            )),
                        ));
                    }
                }
            }
            _ => {}
        }

        match schema
            .resolve_type_validation(type_id)
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
        _: &Lookup<'a, E>,
    ) -> Result<(), PayloadValidationError<Self>> {
        // Non-custom values must have non-custom type kinds...
        // But custom type validations aren't allowed to combine with non-custom type kinds
        Err(PayloadValidationError::SchemaInconsistency)
    }
}

fn apply_custom_validation_to_custom_value<E: Debug>(
    custom_validation: &ScryptoCustomTypeValidation,
    custom_value: &ScryptoCustomValue,
    lookup: &Lookup<E>,
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
                    node_id.is_internal()
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

fn resolve_type_info<E: Debug>(
    node_id: &NodeId,
    lookup: &Lookup<E>,
) -> Result<TypeInfoForValidation, PayloadValidationError<ScryptoCustomExtension>> {
    lookup.get_node_type_info(node_id).map_err(|e| {
        PayloadValidationError::ValidationError(ValidationError::CustomError(format!("{:?}", e)))
    })
}
