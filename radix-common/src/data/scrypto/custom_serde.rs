use crate::internal_prelude::*;

impl SerializableCustomExtension for ScryptoCustomExtension {
    fn map_value_for_serialization<'s, 'de, 'a, 't, 's1, 's2>(
        context: &SerializationContext<'s, 'a, Self>,
        _: LocalTypeId,
        custom_value: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self> {
        let (serialization, include_type_tag_in_simple_mode) = match custom_value.0 {
            ScryptoCustomValue::Reference(value) => (
                SerializableType::String(
                    value
                        .0
                        .to_string(context.custom_context.address_bech32_encoder),
                ),
                true,
            ),
            ScryptoCustomValue::Own(value) => (
                SerializableType::String(
                    value
                        .0
                        .to_string(context.custom_context.address_bech32_encoder),
                ),
                true,
            ),
            ScryptoCustomValue::Decimal(value) => {
                (SerializableType::String(value.to_string()), false)
            }
            ScryptoCustomValue::PreciseDecimal(value) => {
                (SerializableType::String(value.to_string()), false)
            }
            ScryptoCustomValue::NonFungibleLocalId(value) => {
                (SerializableType::String(value.to_string()), true)
            }
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
    use crate::data::scrypto::model::*;
    use crate::data::scrypto::{scrypto_encode, ScryptoValue};
    use crate::math::*;
    use crate::types::*;
    use radix_rust::ContextualSerialize;
    use sbor::rust::vec;
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
        let value = Reference(FUNGIBLE_RESOURCE.as_node_id().to_owned());

        let expected_natural = json!({
            "kind": "Reference",
            "value": FUNGIBLE_RESOURCE_NO_NETWORK_STRING
        });
        let expected_invertible = json!({
            "kind": "Reference",
            "value": FUNGIBLE_RESOURCE_NO_NETWORK_STRING
        });

        assert_natural_json_matches(
            &value,
            ScryptoValueDisplayContext::no_context(),
            expected_natural,
        );
        assert_programmatic_json_matches(
            &value,
            ScryptoValueDisplayContext::no_context(),
            expected_invertible,
        );
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_address_encoding_with_network() {
        let value = Reference(FUNGIBLE_RESOURCE_NODE_ID);
        let encoder = AddressBech32Encoder::for_simulator();

        let expected_simple = json!({
            "kind": "Reference",
            "value": FUNGIBLE_RESOURCE_SIM_ADDRESS
        });
        let expected_invertible = json!({
            "kind": "Reference",
            "value": FUNGIBLE_RESOURCE_SIM_ADDRESS
        });

        assert_natural_json_matches(&value, &encoder, expected_simple);
        assert_programmatic_json_matches(&value, &encoder, expected_invertible);
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_complex_encoding_with_network() {
        let encoder = AddressBech32Encoder::for_simulator();
        let value = ScryptoValue::Tuple {
            fields: vec![
                Value::Custom {
                    value: ScryptoCustomValue::Reference(Reference(FUNGIBLE_RESOURCE_NODE_ID)),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own(FUNGIBLE_RESOURCE_NODE_ID)),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Decimal(Decimal::ONE),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Decimal(Decimal::ONE.checked_div(100).unwrap()),
                },
                Value::Custom {
                    value: ScryptoCustomValue::PreciseDecimal(PreciseDecimal::ZERO),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(
                        NonFungibleLocalId::string("hello").unwrap(),
                    ),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(NonFungibleLocalId::integer(123)),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(
                        NonFungibleLocalId::bytes(vec![0x23, 0x45]).unwrap(),
                    ),
                },
                Value::Custom {
                    value: ScryptoCustomValue::NonFungibleLocalId(NonFungibleLocalId::ruid(
                        [0x11; 32],
                    )),
                },
            ],
        };

        let expected_programmatic = json!({
            "fields": [
                {
                    "kind": "Reference",
                    "value": FUNGIBLE_RESOURCE_SIM_ADDRESS
                },
                {
                    "kind": "Own",
                    "value": FUNGIBLE_RESOURCE_SIM_ADDRESS
                },
                {
                    "kind": "Decimal",
                    "value": "1"
                },
                {
                    "kind": "Decimal",
                    "value": "0.01"
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
                    "kind": "NonFungibleLocalId",
                    "value": "#123#"
                },
                {
                    "kind": "NonFungibleLocalId",
                    "value": "[2345]"
                },
                {
                    "kind": "NonFungibleLocalId",
                    "value": "{1111111111111111-1111111111111111-1111111111111111-1111111111111111}"
                },
            ],
            "kind": "Tuple"
        });

        let expected_natural = json!([
            {
                "kind": "Reference",
                "value": FUNGIBLE_RESOURCE_SIM_ADDRESS
            },
            {
                "kind": "Own",
                "value": FUNGIBLE_RESOURCE_SIM_ADDRESS
            },
            "1",
            "0.01",
            "0",
            {
                "kind": "NonFungibleLocalId",
                "value": "<hello>"
            },
            {
                "kind": "NonFungibleLocalId",
                "value": "#123#"
            },
            {
                "kind": "NonFungibleLocalId",
                "value": "[2345]"
            },
            {
                "kind": "NonFungibleLocalId",
                "value": "{1111111111111111-1111111111111111-1111111111111111-1111111111111111}"
            },
        ]);

        let context = ScryptoValueDisplayContext::with_optional_bech32(Some(&encoder));

        assert_natural_json_matches(&value, context, expected_natural);
        assert_programmatic_json_matches(&value, context, expected_programmatic);
    }

    fn assert_natural_json_matches<
        'a,
        T: ScryptoEncode,
        C: Into<ScryptoValueDisplayContext<'a>>,
    >(
        value: &T,
        context: C,
        expected: JsonValue,
    ) {
        let payload = scrypto_encode(&value).unwrap();

        assert_json_eq(
            ScryptoRawPayload::new_from_valid_slice(&payload).serializable(
                SerializationParameters::Schemaless {
                    mode: SerializationMode::Natural,
                    custom_context: context.into(),
                    depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
                },
            ),
            expected,
        );
    }

    fn assert_programmatic_json_matches<
        'a,
        T: ScryptoEncode,
        C: Into<ScryptoValueDisplayContext<'a>>,
    >(
        value: &T,
        context: C,
        expected: JsonValue,
    ) {
        let payload = scrypto_encode(&value).unwrap();

        assert_json_eq(
            ScryptoRawPayload::new_from_valid_slice(&payload).serializable(
                SerializationParameters::Schemaless {
                    mode: SerializationMode::Programmatic,
                    custom_context: context.into(),
                    depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
                },
            ),
            expected,
        );
    }
}
