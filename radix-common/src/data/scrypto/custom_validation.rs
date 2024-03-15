use crate::internal_prelude::*;

//=======================================================================================================
// NOTE:
// The validation implemented here is static and can be used where we can't create a TypeInfo lookup.
//
// There is more powerful validation defined in `payload_validation.rs` in the `radix-engine` crate
// which does requires creating the lookup, and checks package/blueprint names of objects.
//=======================================================================================================

impl<'a> ValidatableCustomExtension<()> for ScryptoCustomExtension {
    fn apply_validation_for_custom_value<'de>(
        schema: &Schema<Self::CustomSchema>,
        custom_value: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        type_id: LocalTypeId,
        _: &(),
    ) -> Result<(), PayloadValidationError<Self>> {
        match schema
            .resolve_type_validation(type_id)
            .ok_or(PayloadValidationError::SchemaInconsistency)?
        {
            TypeValidation::None => Ok(()),
            TypeValidation::Custom(custom_validation) => {
                apply_static_custom_validation_to_custom_value(custom_validation, &custom_value.0)
            }
            _ => Err(PayloadValidationError::SchemaInconsistency),
        }
    }

    fn apply_custom_type_validation_for_non_custom_value<'de>(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomSchema as CustomSchema>::CustomTypeValidation,
        _: &TerminalValueRef<'de, Self::CustomTraversal>,
        _: &(),
    ) -> Result<(), PayloadValidationError<Self>> {
        // Non-custom values must have non-custom type kinds...
        // But custom type validations aren't allowed to combine with non-custom type kinds
        Err(PayloadValidationError::SchemaInconsistency)
    }
}

fn apply_static_custom_validation_to_custom_value(
    custom_validation: &ScryptoCustomTypeValidation,
    custom_value: &ScryptoCustomValue,
) -> Result<(), PayloadValidationError<ScryptoCustomExtension>> {
    match custom_validation {
        ScryptoCustomTypeValidation::Reference(reference_validation) => {
            let ScryptoCustomValue::Reference(reference) = custom_value else {
                return Err(PayloadValidationError::SchemaInconsistency);
            };
            let node_id = reference.0;
            let is_valid = match &reference_validation {
                ReferenceValidation::IsGlobal => node_id.is_global(),
                ReferenceValidation::IsGlobalPackage => node_id.is_global_package(),
                ReferenceValidation::IsGlobalComponent => node_id.is_global_component(),
                ReferenceValidation::IsGlobalResourceManager => {
                    node_id.is_global_resource_manager()
                }
                // We can't check this statically without a type_info lookup, so assume valid
                ReferenceValidation::IsGlobalTyped(_, _) => node_id.is_global(),
                ReferenceValidation::IsInternal => node_id.is_internal(),
                // We can't check this statically without a type_info lookup, so assume valid
                ReferenceValidation::IsInternalTyped(_, _) => node_id.is_internal(),
            };
            if !is_valid {
                return Err(PayloadValidationError::ValidationError(
                    ValidationError::CustomError(format!(
                        "Expected = Reference<{:?}>, found node id with entity type: {:?}",
                        reference_validation,
                        node_id.entity_type()
                    )),
                ));
            }
        }
        ScryptoCustomTypeValidation::Own(own_validation) => {
            let ScryptoCustomValue::Own(own) = custom_value else {
                return Err(PayloadValidationError::SchemaInconsistency);
            };
            let node_id = own.0;
            // We can't check the type_info details statically, so we do the best we can with the entity byte
            let is_valid = match own_validation {
                OwnValidation::IsBucket => node_id.is_internal(),
                OwnValidation::IsProof => node_id.is_internal(),
                OwnValidation::IsVault => node_id.is_internal_vault(),
                OwnValidation::IsKeyValueStore => node_id.is_internal_kv_store(),
                OwnValidation::IsGlobalAddressReservation => true,
                OwnValidation::IsTypedObject(_, _) => true,
            };
            if !is_valid {
                return Err(PayloadValidationError::ValidationError(
                    ValidationError::CustomError(format!(
                        "Expected = Own<{:?}>, found node id with entity type: {:?}",
                        own_validation,
                        node_id.entity_type()
                    )),
                ));
            }
        }
    };
    Ok(())
}
