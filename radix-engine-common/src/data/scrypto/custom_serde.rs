use super::*;
use crate::*;
use sbor::representations::*;
use sbor::rust::prelude::*;
use sbor::traversal::*;
use sbor::*;
use utils::ContextualDisplay;

impl SerializableCustomTypeExtension for ScryptoCustomTypeExtension {
    fn map_value_for_serialization<'s, 'de, 'a, 't, 's1, 's2>(
        context: &SerializationContext<'s, 'a, Self>,
        _: LocalTypeIndex,
        custom_value: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self> {
        let (serialization, include_type_tag_in_simple_mode) = match custom_value.0 {
            ScryptoCustomValue::Reference(value) => (
                SerializableType::String(value.0.to_string(context.custom_context.bech32_encoder)),
                true,
            ),
            ScryptoCustomValue::Own(value) => (
                SerializableType::String(value.0.to_string(context.custom_context.bech32_encoder)),
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
    use crate::address::Bech32Encoder;
    use crate::data::scrypto::model::*;
    use crate::data::scrypto::{scrypto_encode, ScryptoValue};
    use crate::types::*;
    use sbor::rust::vec;
    use serde::Serialize;
    use serde_json::{json, to_string, to_value, Value as JsonValue};
    use utils::ContextualSerialize;

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
        use crate::types::NodeId;
        let value = Reference(NodeId([0; 27]));

        let expected =
            json!("FungibleResource[010000000000000000000000000000000000000000000000000000]");
        let expected_invertible = json!({
            "kind": "Reference",
            "value": "000000000000000000000000000000000000000000000000000000"
        });

        assert_natural_json_matches(&value, ScryptoValueDisplayContext::no_context(), expected);
        assert_programmatic_json_matches(
            &value,
            ScryptoValueSerializationContext::no_context(),
            expected_invertible,
        );
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_address_encoding_with_network() {
        use crate::types::NodeId;

        let value = Reference(NodeId([0; 27]));
        let encoder = Bech32Encoder::for_simulator();

        let expected_simple =
            json!("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k");
        let expected_invertible = json!({
            "kind": "Reference",
            "value": "resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k"
        });

        assert_natural_json_matches(&value, &encoder, expected_simple);
        assert_programmatic_json_matches(&value, &encoder, expected_invertible);
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_complex_encoding_with_network() {
        use crate::math::{Decimal, PreciseDecimal};
        use crate::types::NodeId;

        let encoder = Bech32Encoder::for_simulator();
        let value = ScryptoValue::Tuple {
            fields: vec![
                Value::Custom {
                    value: ScryptoCustomValue::Reference(Reference(NodeId([0; NodeId::LENGTH]))),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own(NodeId([0; NodeId::LENGTH]))),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Decimal(Decimal::ONE),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Decimal(Decimal::ONE / 100),
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
                    value: ScryptoCustomValue::NonFungibleLocalId(
                        NonFungibleLocalId::uuid(0x1f52cb1e_86c4_47ae_9847_9cdb14662ebd).unwrap(),
                    ),
                },
            ],
        };

        let expected_programmatic = json!({
            "fields": [
                {
                    "kind": "Address",
                    "value": "resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k"
                },
                {
                    "kind": "Own",
                    "value": "00000000000000000000000000000000000000000000000000000000000000"
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
                    "value": "{1f52cb1e-86c4-47ae-9847-9cdb14662ebd}"
                },
                {
                    "kind": "Reference",
                    "value": "00000000000000000000000000000000000000000000000000000000000000"
                }
            ],
            "kind": "Tuple"
        });

        let expected_natural = json!([
            "resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k",
            {
                "kind": "Own",
                "value": "00000000000000000000000000000000000000000000000000000000000000"
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
                "value": "{1f52cb1e-86c4-47ae-9847-9cdb14662ebd}"
            },
            {
                "kind": "Reference",
                "value": "00000000000000000000000000000000000000000000000000000000000000"
            }
        ]);

        let context = ScryptoValueDisplayContext::with_optional_bech32(Some(&encoder));

        assert_natural_json_matches(&value, context, expected_natural);
        assert_programmatic_json_matches(&value, context, expected_programmatic);
    }

    fn assert_natural_json_matches<
        'a,
        T: ScryptoEncode,
        C: Into<ScryptoValueSerializationContext<'a>>,
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
                },
            ),
            expected,
        );
    }
}
