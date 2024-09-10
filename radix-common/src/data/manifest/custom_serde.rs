use super::converter::*;
use super::model::*;
use super::*;
use crate::internal_prelude::*;

impl SerializableCustomExtension for ManifestCustomExtension {
    fn map_value_for_serialization<'s, 'de, 'a, 't, 's1, 's2>(
        context: &SerializationContext<'s, 'a, Self>,
        _: LocalTypeId,
        custom_value: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self> {
        let (serialization, include_type_tag_in_simple_mode) = match custom_value.0 {
            ManifestCustomValue::Address(value) => match value {
                ManifestAddress::Static(node_id) => {
                    if let Some(encoder) = context.custom_context.address_bech32_encoder {
                        if let Ok(bech32) = encoder.encode(node_id.as_ref()) {
                            (SerializableType::String(bech32), false)
                        } else {
                            (
                                SerializableType::String(hex::encode(node_id.as_bytes())),
                                true,
                            )
                        }
                    } else {
                        (
                            SerializableType::String(hex::encode(node_id.as_bytes())),
                            true,
                        )
                    }
                }
                ManifestAddress::Named(value) => {
                    if let Some(name) = context.custom_context.get_address_name(&value) {
                        (SerializableType::String(name.to_string()), true)
                    } else {
                        (SerializableType::String(value.0.to_string()), true)
                    }
                }
            },
            ManifestCustomValue::Bucket(value) => {
                if let Some(name) = context.custom_context.get_bucket_name(&value) {
                    (SerializableType::String(name.to_string()), true)
                } else {
                    (SerializableType::String(value.0.to_string()), true)
                }
            }
            ManifestCustomValue::Proof(value) => {
                if let Some(name) = context.custom_context.get_proof_name(&value) {
                    (SerializableType::String(name.to_string()), true)
                } else {
                    (SerializableType::String(value.0.to_string()), true)
                }
            }
            ManifestCustomValue::AddressReservation(value) => {
                if let Some(name) = context.custom_context.get_address_reservation_name(&value) {
                    (SerializableType::String(name.to_string()), true)
                } else {
                    (SerializableType::String(value.0.to_string()), true)
                }
            }
            ManifestCustomValue::Expression(value) => {
                let text = match value {
                    ManifestExpression::EntireWorktop => "ENTIRE_WORKTOP",
                    ManifestExpression::EntireAuthZone => "ENTIRE_AUTH_ZONE",
                };
                (SerializableType::String(text.to_string()), true)
            }
            ManifestCustomValue::Blob(value) => {
                (SerializableType::String(hex::encode(&value.0)), true)
            }
            ManifestCustomValue::Decimal(value) => (
                SerializableType::String(format!("{}", to_decimal(&value))),
                true,
            ),
            ManifestCustomValue::PreciseDecimal(value) => (
                SerializableType::String(format!("{}", to_precise_decimal(&value))),
                true,
            ),
            ManifestCustomValue::NonFungibleLocalId(value) => (
                SerializableType::String(format!("{}", to_non_fungible_local_id(value))),
                true,
            ),
        };
        CustomTypeSerialization {
            serialization,
            include_type_tag_in_simple_mode,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "serde")] // Ensures that VS Code runs this module with the features serde tag!
mod tests {
    use super::*;
    use crate::address::test_addresses::*;
    use crate::address::AddressBech32Encoder;
    use crate::types::*;
    use radix_rust::ContextualSerialize;
    use serde::Serialize;
    use serde_json::{json, to_string, to_value, Value as JsonValue};

    #[derive(ScryptoSbor)]
    pub struct Sample {
        pub a: ResourceAddress,
    }

    pub fn assert_json_eq<T: Serialize>(actual: T, expected: JsonValue) {
        let actual = to_value(&actual).unwrap();
        if actual != expected {
            panic!(
                "Mismatching JSON:\nActual:\n{}\nExpected:\n{}\n",
                to_string(&actual).unwrap(),
                to_string(&expected).unwrap()
            );
        }
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_address_encoding_no_network() {
        let value = ManifestCustomValue::Address(ManifestAddress::Static(
            FUNGIBLE_RESOURCE.as_node_id().clone(),
        ));

        let expected_natural = json!({
            "kind": "Address",
            "value": FUNGIBLE_RESOURCE_HEX_STRING
        });
        let expected_programmatic = json!({
            "kind": "Address",
            "value": FUNGIBLE_RESOURCE_HEX_STRING
        });

        assert_natural_json_matches(
            &value,
            ManifestValueDisplayContext::no_context(),
            expected_natural,
        );
        assert_programmatic_json_matches(
            &value,
            ManifestValueDisplayContext::no_context(),
            expected_programmatic,
        );
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_address_encoding_with_network() {
        let value = ManifestCustomValue::Address(ManifestAddress::Static(
            FUNGIBLE_RESOURCE.as_node_id().clone(),
        ));
        let encoder = AddressBech32Encoder::for_simulator();

        let expected_natural = json!(FUNGIBLE_RESOURCE_SIM_ADDRESS);
        let expected_programmatic = json!({
            "kind": "Address",
            "value": FUNGIBLE_RESOURCE_SIM_ADDRESS
        });

        assert_natural_json_matches(&value, &encoder, expected_natural);
        assert_programmatic_json_matches(&value, &encoder, expected_programmatic);
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_complex_encoding_with_network() {
        let encoder = AddressBech32Encoder::for_simulator();
        let value = (
            ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Static(
                    FUNGIBLE_RESOURCE.as_node_id().clone(),
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
        );

        let expected_programmatic = json!({
            "fields": [
                {
                    "kind": "Address",
                    "value": "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3"
                },
                {
                    "kind": "Blob",
                    "value": "0000000000000000000000000000000000000000000000000000000000000000"
                },
                {
                    "kind": "Bucket",
                    "value": "0"
                },
                {
                    "kind": "Proof",
                    "value": "0"
                },
                {
                    "kind": "Decimal",
                    "value": "0"
                },
                {
                    "kind": "PreciseDecimal",
                    "value": "0"
                },
                {
                    "kind": "NonFungibleLocalId",
                    "value": "<hello>"
                },
                {
                    "kind": "Expression",
                    "value": "ENTIRE_AUTH_ZONE"
                },
                {
                    "kind": "Expression",
                    "value": "ENTIRE_WORKTOP"
                }
            ],
            "kind": "Tuple"
        });

        let expected_natural = json!([
            "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3",
            {
                "kind": "Blob",
                "value": "0000000000000000000000000000000000000000000000000000000000000000"
            },
            {
                "kind": "Bucket",
                "value": "0"
            },
            {
                "kind": "Proof",
                "value": "0"
            },
            {
                "kind": "Decimal",
                "value": "0"
            },
            {
                "kind": "PreciseDecimal",
                "value": "0"
            },
            {
                "kind": "NonFungibleLocalId",
                "value": "<hello>"
            },
            {
                "kind": "Expression",
                "value": "ENTIRE_AUTH_ZONE"
            },
            {
                "kind": "Expression",
                "value": "ENTIRE_WORKTOP"
            }
        ]);

        let context = ManifestValueDisplayContext::with_optional_bech32(Some(&encoder));

        assert_natural_json_matches(&value, context, expected_natural);
        assert_programmatic_json_matches(&value, context, expected_programmatic);
    }

    fn assert_natural_json_matches<
        'a,
        T: ManifestEncode,
        C: Into<ManifestValueDisplayContext<'a>>,
    >(
        value: &T,
        context: C,
        expected: JsonValue,
    ) {
        let payload = manifest_encode(&value).unwrap();

        assert_json_eq(
            ManifestRawPayload::new_from_valid_slice(&payload).serializable(
                SerializationParameters::Schemaless {
                    mode: SerializationMode::Natural,
                    custom_context: context.into(),
                    depth_limit: MANIFEST_SBOR_V1_MAX_DEPTH,
                },
            ),
            expected,
        );
    }

    fn assert_programmatic_json_matches<
        'a,
        T: ManifestEncode,
        C: Into<ManifestValueDisplayContext<'a>>,
    >(
        value: &T,
        context: C,
        expected: JsonValue,
    ) {
        let payload = manifest_encode(&value).unwrap();

        assert_json_eq(
            ManifestRawPayload::new_from_valid_slice(&payload).serializable(
                SerializationParameters::Schemaless {
                    mode: SerializationMode::Programmatic,
                    custom_context: context.into(),
                    depth_limit: MANIFEST_SBOR_V1_MAX_DEPTH,
                },
            ),
            expected,
        );
    }
}
