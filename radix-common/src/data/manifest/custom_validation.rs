use super::model::*;
use crate::data::scrypto::{
    ReferenceValidation, ScryptoCustomTypeKind, ScryptoCustomTypeValidation,
};
use crate::internal_prelude::*;

impl<'a> ValidatableCustomExtension<()> for ManifestCustomExtension {
    fn apply_validation_for_custom_value<'de>(
        schema: &Schema<Self::CustomSchema>,
        custom_value: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        type_id: LocalTypeId,
        _: &(),
    ) -> Result<(), PayloadValidationError<Self>> {
        let ManifestCustomTerminalValueRef(custom_value) = custom_value;
        // Because of the mis-match between Manifest values and Scrypto TypeKinds/TypeValidations, it's easier to match over values first,
        // and then check over which validations apply.
        match custom_value {
            ManifestCustomValue::Expression(ManifestExpression::EntireWorktop) => {
                let element_type = match schema
                    .resolve_type_kind(type_id)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?
                {
                    TypeKind::Any => return Ok(()), // Can't do any validation on an any
                    TypeKind::Array { element_type } => element_type,
                    _ => return Err(PayloadValidationError::SchemaInconsistency),
                };
                let element_type_kind = schema
                    .resolve_type_kind(*element_type)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?;
                match element_type_kind {
                    TypeKind::Any |
                    TypeKind::Custom(ScryptoCustomTypeKind::Own) => {
                        let element_type_validation = schema.resolve_type_validation(*element_type).ok_or(PayloadValidationError::SchemaInconsistency)?;
                        match element_type_validation {
                            TypeValidation::None => {},
                            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(own_validation)) => {
                                if !own_validation.could_match_manifest_bucket() {
                                    return Err(PayloadValidationError::ValidationError(ValidationError::CustomError(format!("ENTIRE_WORKTOP gives an array of buckets, but an array of Own<{:?}> was expected", own_validation))))
                                }
                            },
                            _ => return Err(PayloadValidationError::SchemaInconsistency),
                        }
                    },
                    _ => {
                        return Err(PayloadValidationError::ValidationError(ValidationError::CustomError(format!("ENTIRE_WORKTOP gives an array of buckets, but an array of {:?} was expected", element_type_kind))))
                    }
                };
            }
            ManifestCustomValue::Expression(ManifestExpression::EntireAuthZone) => {
                let element_type = match schema
                    .resolve_type_kind(type_id)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?
                {
                    TypeKind::Any => return Ok(()), // Can't do any validation on an any
                    TypeKind::Array { element_type } => element_type,
                    _ => return Err(PayloadValidationError::SchemaInconsistency),
                };
                let element_type_kind = schema
                    .resolve_type_kind(*element_type)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?;
                match element_type_kind {
                    TypeKind::Any |
                    TypeKind::Custom(ScryptoCustomTypeKind::Own) => {
                        let element_type_validation = schema.resolve_type_validation(*element_type).ok_or(PayloadValidationError::SchemaInconsistency)?;
                        match element_type_validation {
                            TypeValidation::None => {},
                            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(own_validation)) => {
                                if !own_validation.could_match_manifest_proof() {
                                    return Err(PayloadValidationError::ValidationError(ValidationError::CustomError(format!("ENTIRE_AUTH_ZONE gives an array of proofs, but an array of Own<{:?}> was expected", own_validation))))
                                }
                            },
                            _ => return Err(PayloadValidationError::SchemaInconsistency),
                        }
                    },
                    _ => {
                        return Err(PayloadValidationError::ValidationError(ValidationError::CustomError(format!("ENTIRE_AUTH_ZONE gives an array of proofs, but an array of {:?} was expected", element_type_kind))))
                    }
                };
            }
            ManifestCustomValue::Blob(_) => {
                let element_type = match schema
                    .resolve_type_kind(type_id)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?
                {
                    TypeKind::Any => return Ok(()), // Can't do any validation on an any
                    TypeKind::Array { element_type } => element_type,
                    _ => return Err(PayloadValidationError::SchemaInconsistency),
                };
                let element_type_kind = schema
                    .resolve_type_kind(*element_type)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?;
                let is_valid = match element_type_kind {
                    TypeKind::Any => true,
                    TypeKind::U8 => true,
                    _ => false,
                };
                if !is_valid {
                    return Err(PayloadValidationError::ValidationError(
                        ValidationError::CustomError(format!(
                            "Blob provides a U8 array, but an array of {:?} was expected",
                            element_type_kind
                        )),
                    ));
                }
            }
            ManifestCustomValue::Address(address) => {
                // We know from `custom_value_kind_matches_type_kind` that this has a ScryptoCustomTypeKind::Reference
                let validation = schema
                    .resolve_type_validation(type_id)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?;
                match validation {
                    TypeValidation::None => {}
                    TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                        reference_validation,
                    )) => {
                        let is_valid = match address {
                            ManifestAddress::Static(node_id) => match reference_validation {
                                ReferenceValidation::IsGlobal => node_id.is_global(),
                                ReferenceValidation::IsGlobalPackage => node_id.is_global_package(),
                                ReferenceValidation::IsGlobalComponent => {
                                    node_id.is_global_component()
                                }
                                ReferenceValidation::IsGlobalResourceManager => {
                                    node_id.is_global_resource_manager()
                                }
                                ReferenceValidation::IsGlobalTyped(_, _) => node_id.is_global(), // Assume yes
                                ReferenceValidation::IsInternal => node_id.is_internal(),
                                ReferenceValidation::IsInternalTyped(_, _) => node_id.is_internal(), // Assume yes
                            },
                            ManifestAddress::Named(_) => {
                                reference_validation.could_match_manifest_address()
                            }
                        };
                        if !is_valid {
                            return Err(PayloadValidationError::ValidationError(
                                ValidationError::CustomError(format!(
                                    "Expected Reference<{:?}>",
                                    reference_validation
                                )),
                            ));
                        }
                    }
                    _ => return Err(PayloadValidationError::SchemaInconsistency),
                };
            }
            ManifestCustomValue::Bucket(_) => {
                // We know from `custom_value_kind_matches_type_kind` that this has a ScryptoCustomTypeKind::Own
                let validation = schema
                    .resolve_type_validation(type_id)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?;
                match validation {
                    TypeValidation::None => {}
                    TypeValidation::Custom(ScryptoCustomTypeValidation::Own(own_validation)) => {
                        if !own_validation.could_match_manifest_bucket() {
                            return Err(PayloadValidationError::ValidationError(
                                ValidationError::CustomError(format!(
                                    "Expected Own<{:?}>, but found manifest bucket",
                                    own_validation
                                )),
                            ));
                        }
                    }
                    _ => return Err(PayloadValidationError::SchemaInconsistency),
                };
            }
            ManifestCustomValue::Proof(_) => {
                // We know from `custom_value_kind_matches_type_kind` that this has a ScryptoCustomTypeKind::Own
                let validation = schema
                    .resolve_type_validation(type_id)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?;
                match validation {
                    TypeValidation::None => {}
                    TypeValidation::Custom(ScryptoCustomTypeValidation::Own(own_validation)) => {
                        if !own_validation.could_match_manifest_proof() {
                            return Err(PayloadValidationError::ValidationError(
                                ValidationError::CustomError(format!(
                                    "Expected Own<{:?}>, but found manifest proof",
                                    own_validation
                                )),
                            ));
                        }
                    }
                    _ => return Err(PayloadValidationError::SchemaInconsistency),
                };
            }
            ManifestCustomValue::AddressReservation(_) => {
                // We know from `custom_value_kind_matches_type_kind` that this has a ScryptoCustomTypeKind::Own
                let validation = schema
                    .resolve_type_validation(type_id)
                    .ok_or(PayloadValidationError::SchemaInconsistency)?;
                match validation {
                    TypeValidation::None => {}
                    TypeValidation::Custom(ScryptoCustomTypeValidation::Own(own_validation)) => {
                        if !own_validation.could_match_manifest_address_reservation() {
                            return Err(PayloadValidationError::ValidationError(
                                ValidationError::CustomError(format!(
                                    "Expected Own<{:?}>, but found manifest address reservation",
                                    own_validation
                                )),
                            ));
                        }
                    }
                    _ => return Err(PayloadValidationError::SchemaInconsistency),
                };
            }
            // No custom validations apply (yet) to Decimal/PreciseDecimal/NonFungibleLocalId
            ManifestCustomValue::Decimal(_) => {}
            ManifestCustomValue::PreciseDecimal(_) => {}
            ManifestCustomValue::NonFungibleLocalId(_) => {}
        };
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::*;
    use crate::data::scrypto::model::NonFungibleLocalId;
    use crate::data::scrypto::{well_known_scrypto_custom_types, ScryptoValue};
    use crate::data::scrypto::{ScryptoCustomSchema, ScryptoDescribe};
    use crate::math::{Decimal, PreciseDecimal};
    use crate::types::{PackageAddress, ResourceAddress};

    pub struct Bucket;

    impl Describe<ScryptoCustomTypeKind> for Bucket {
        const TYPE_ID: RustTypeId =
            RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_BUCKET_TYPE);

        fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
            well_known_scrypto_custom_types::own_bucket_type_data()
        }
    }

    pub struct Proof;

    impl Describe<ScryptoCustomTypeKind> for Proof {
        const TYPE_ID: RustTypeId =
            RustTypeId::WellKnown(well_known_scrypto_custom_types::OWN_PROOF_TYPE);

        fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
            well_known_scrypto_custom_types::own_proof_type_data()
        }
    }

    type MyScryptoTuple = (
        ResourceAddress,
        Vec<u8>,
        Bucket,
        Proof,
        Decimal,
        PreciseDecimal,
        NonFungibleLocalId,
        Vec<Proof>,
        Vec<Bucket>,
    );

    type Any = ScryptoValue;

    #[test]
    fn valid_manifest_composite_value_passes_validation_against_radix_blueprint_schema_init() {
        let payload = manifest_encode(&(
            ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Static(
                    XRD.as_node_id().clone(),
                )),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Blob(ManifestBlobRef([0; 32])),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Bucket(ManifestBucket(0)),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Proof(ManifestProof(0)),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Decimal(ManifestDecimal([0; DECIMAL_SIZE])),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::PreciseDecimal(ManifestPreciseDecimal(
                    [0; PRECISE_DECIMAL_SIZE],
                )),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::NonFungibleLocalId(ManifestNonFungibleLocalId::String(
                    "hello".to_string(),
                )),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Expression(ManifestExpression::EntireAuthZone),
            },
            ManifestValue::Custom {
                value: ManifestCustomValue::Expression(ManifestExpression::EntireWorktop),
            },
        ))
        .unwrap();

        let (type_id, schema) =
            generate_full_schema_from_single_type::<MyScryptoTuple, ScryptoCustomSchema>();

        let result = validate_payload_against_schema::<ManifestCustomExtension, _>(
            &payload,
            schema.v1(),
            type_id,
            &(),
            MANIFEST_SBOR_V1_MAX_DEPTH,
        );

        result.expect("Validation check failed");
    }

    #[test]
    fn manifest_address_fails_validation_against_mismatching_radix_blueprint_schema_init() {
        let payload = manifest_encode(&ManifestValue::Custom {
            value: ManifestCustomValue::Address(ManifestAddress::Static(XRD.as_node_id().clone())),
        })
        .unwrap();

        expect_matches::<ResourceAddress>(&payload);
        expect_matches::<Any>(&payload);
        expect_does_not_match::<PackageAddress>(&payload);
        expect_does_not_match::<Bucket>(&payload);
        expect_does_not_match::<u8>(&payload);
    }

    #[test]
    fn manifest_blob_fails_validation_against_mismatching_radix_blueprint_schema_init() {
        let payload = manifest_encode(&ManifestValue::Custom {
            value: ManifestCustomValue::Blob(ManifestBlobRef([0; 32])),
        })
        .unwrap();

        expect_matches::<Vec<u8>>(&payload);
        expect_matches::<Any>(&payload);
        expect_does_not_match::<Vec<Bucket>>(&payload);
        expect_does_not_match::<Vec<Proof>>(&payload);
        expect_does_not_match::<Proof>(&payload);
        expect_does_not_match::<u8>(&payload);
    }

    #[test]
    fn manifest_entire_worktop_expression_fails_validation_against_mismatching_radix_blueprint_schema_init(
    ) {
        let payload = manifest_encode(&ManifestValue::Custom {
            value: ManifestCustomValue::Expression(ManifestExpression::EntireWorktop),
        })
        .unwrap();

        expect_matches::<Vec<Bucket>>(&payload);
        expect_matches::<Any>(&payload);
        expect_does_not_match::<Vec<Proof>>(&payload);
        expect_does_not_match::<Proof>(&payload);
        expect_does_not_match::<Bucket>(&payload);
        expect_does_not_match::<u8>(&payload);
    }

    #[test]
    fn manifest_entire_auth_zone_expression_fails_validation_against_mismatching_radix_blueprint_schema_init(
    ) {
        let payload = manifest_encode(&ManifestValue::Custom {
            value: ManifestCustomValue::Expression(ManifestExpression::EntireAuthZone),
        })
        .unwrap();

        expect_matches::<Vec<Proof>>(&payload);
        expect_matches::<Any>(&payload);
        expect_does_not_match::<Vec<Bucket>>(&payload);
        expect_does_not_match::<Proof>(&payload);
        expect_does_not_match::<Bucket>(&payload);
        expect_does_not_match::<u8>(&payload);
    }

    fn expect_matches<T: ScryptoDescribe>(payload: &[u8]) {
        let (type_id, schema) = generate_full_schema_from_single_type::<T, ScryptoCustomSchema>();

        let result = validate_payload_against_schema::<ManifestCustomExtension, _>(
            &payload,
            schema.v1(),
            type_id,
            &(),
            MANIFEST_SBOR_V1_MAX_DEPTH,
        );

        result.expect("Expected validation to succeed");
    }

    fn expect_does_not_match<T: ScryptoDescribe>(payload: &[u8]) {
        let (type_id, schema) = generate_full_schema_from_single_type::<T, ScryptoCustomSchema>();

        let result = validate_payload_against_schema::<ManifestCustomExtension, _>(
            &payload,
            schema.v1(),
            type_id,
            &(),
            MANIFEST_SBOR_V1_MAX_DEPTH,
        );

        matches!(
            result,
            Err(LocatedValidationError {
                error: PayloadValidationError::ValidationError(_),
                ..
            })
        );
    }
}
