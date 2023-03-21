use super::*;
use crate::data::scrypto::model::InternalRef;
use crate::*;
use sbor::serde_serialization::*;
use sbor::traversal::*;
use sbor::*;
use utils::ContextualDisplay;

impl<'a> CustomSerializationContext<'a> for ScryptoValueDisplayContext<'a> {
    type CustomTypeExtension = ScryptoCustomTypeExtension;
}

impl SerializableCustomTypeExtension for ScryptoCustomTypeExtension {
    type CustomSerializationContext<'a> = ScryptoValueDisplayContext<'a>;

    fn serialize_value<'s, 'de, 'a, 't, 's1, 's2>(
        context: &SerializationContext<'s, 'a, Self>,
        _: LocalTypeIndex,
        custom_value: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self> {
        let (serialization, include_type_tag_in_simple_mode) = match custom_value.0 {
            ScryptoCustomValue::Address(value) => (
                SerializableType::String(value.to_string(context.custom_context.bech32_encoder)),
                false,
            ),
            ScryptoCustomValue::Own(value) => {
                (SerializableType::String(hex::encode(value.id())), true)
            }
            ScryptoCustomValue::Decimal(value) => {
                (SerializableType::String(value.to_string()), false)
            }
            ScryptoCustomValue::PreciseDecimal(value) => {
                (SerializableType::String(value.to_string()), false)
            }
            ScryptoCustomValue::NonFungibleLocalId(value) => {
                (SerializableType::String(value.to_string()), true)
            }
            ScryptoCustomValue::InternalRef(InternalRef(object_id)) => {
                (SerializableType::String(hex::encode(object_id)), true)
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
    use sbor::rust::vec;
    use serde::Serialize;
    use serde_json::{json, to_string, to_value, Value as JsonValue};
    use utils::ContextualSerialize;

    use crate::data::scrypto::{scrypto_encode, ScryptoValue};

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
        let value = ResourceAddress::Fungible([0; ADDRESS_HASH_LENGTH]);

        let expected =
            json!("FungibleResource[010000000000000000000000000000000000000000000000000000]");
        let expected_invertible = json!({
            "kind": "Address",
            "value": "FungibleResource[010000000000000000000000000000000000000000000000000000]"
        });

        assert_simple_json_matches(&value, ScryptoValueDisplayContext::no_context(), expected);
        assert_invertible_json_matches(
            &value,
            ScryptoValueDisplayContext::no_context(),
            expected_invertible,
        );
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_address_encoding_with_network() {
        let value = ResourceAddress::Fungible([0; ADDRESS_HASH_LENGTH]);
        let encoder = Bech32Encoder::for_simulator();

        let expected_simple =
            json!("resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k");
        let expected_invertible = json!({
            "kind": "Address",
            "value": "resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k"
        });

        assert_simple_json_matches(&value, &encoder, expected_simple);
        assert_invertible_json_matches(&value, &encoder, expected_invertible);
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_complex_encoding_with_network() {
        use crate::math::{Decimal, PreciseDecimal};

        let encoder = Bech32Encoder::for_simulator();
        let value = ScryptoValue::Tuple {
            fields: vec![
                Value::Custom {
                    value: ScryptoCustomValue::Address(Address::Resource(
                        ResourceAddress::Fungible([0; ADDRESS_HASH_LENGTH]),
                    )),
                },
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own::Vault([0; OBJECT_ID_LENGTH])),
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
                Value::Custom {
                    value: ScryptoCustomValue::InternalRef(InternalRef([0; OBJECT_ID_LENGTH])),
                },
            ],
        };

        let expected_simple = json!([
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

        let expected_invertible = json!({
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

        let context = ScryptoValueDisplayContext::with_optional_bench32(Some(&encoder));

        assert_simple_json_matches(&value, context, expected_simple);
        assert_invertible_json_matches(&value, context, expected_invertible);
    }

    fn assert_simple_json_matches<'a, T: ScryptoEncode, C: Into<ScryptoValueDisplayContext<'a>>>(
        value: &T,
        context: C,
        expected: JsonValue,
    ) {
        let payload = scrypto_encode(&value).unwrap();

        assert_json_eq(
            SborPayloadWithoutSchema::<ScryptoCustomTypeExtension>::new(&payload).serializable(
                SchemalessSerializationContext {
                    mode: SerializationMode::Simple,
                    custom_context: context.into(),
                },
            ),
            expected,
        );
    }

    fn assert_invertible_json_matches<
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
            SborPayloadWithoutSchema::<ScryptoCustomTypeExtension>::new(&payload).serializable(
                SchemalessSerializationContext {
                    mode: SerializationMode::Invertible,
                    custom_context: context.into(),
                },
            ),
            expected,
        );
    }
}
