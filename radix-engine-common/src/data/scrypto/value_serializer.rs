use super::model::*;
use super::*;
use sbor::rust::prelude::*;
use serde::ser::*;
use utils::{ContextSerializable, ContextualDisplay, ContextualSerialize};

// TODO - Add a deserializer for invertible JSON, and tests that the process is invertible
// TODO - Rewrite value formatter as a serializer/deserializer variant?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScryptoValueSerializationType {
    /// This "simple" encoding is intended to be "nice to read" for a human.
    /// It is intended to be one option (likely the default option) for representing
    /// schemaless scrypto values in a JSON API.
    ///
    /// In particular:
    /// * It is not intended to be invertible - ie the output cannot be mapped back into a ScryptoValue
    /// * It should favour simplicity for human comprehension, in particular:
    ///   * If the concept which is being represented (eg number/amount or address) is clear
    ///     to a human, the type information can be dropped
    ///   * If the concept which is being represented (eg number/amount or address) is clear
    ///     to a human, the type information can be dropped
    ///
    /// We will eventually support simple_with_schema encoding, which will likely be
    /// similar to this, except replace Struct/Enum encodings.
    Simple,
    /// This "invertible" encoding is intended to fully capture the scrypto value's type along with its value
    Invertible,
}

#[derive(Clone, Copy, Debug)]
pub struct ScryptoValueSerializationContext<'a> {
    pub serialization_type: ScryptoValueSerializationType,
    pub display_context: ScryptoValueDisplayContext<'a>,
}

impl<'a> ScryptoValueSerializationContext<'a> {
    pub fn simple(display_context: ScryptoValueDisplayContext<'a>) -> Self {
        Self {
            serialization_type: ScryptoValueSerializationType::Simple,
            display_context,
        }
    }

    pub fn invertible(display_context: ScryptoValueDisplayContext<'a>) -> Self {
        Self {
            serialization_type: ScryptoValueSerializationType::Invertible,
            display_context,
        }
    }
}

pub trait SerializableScryptoValue:
    for<'a> ContextualSerialize<ScryptoValueSerializationContext<'a>>
{
    fn simple_serializable<'a, 'b, TContext: Into<ScryptoValueDisplayContext<'b>>>(
        &'a self,
        context: TContext,
    ) -> ContextSerializable<'a, Self, ScryptoValueSerializationContext<'b>> {
        self.serializable(ScryptoValueSerializationContext::simple(context.into()))
    }

    fn invertible_serializable<'a, 'b, TContext: Into<ScryptoValueDisplayContext<'b>>>(
        &'a self,
        context: TContext,
    ) -> ContextSerializable<'a, Self, ScryptoValueSerializationContext<'b>> {
        self.serializable(ScryptoValueSerializationContext::invertible(context.into()))
    }
}

impl<'a> ContextualSerialize<ScryptoValueSerializationContext<'a>> for ScryptoValue {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueSerializationContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        serialize_schemaless_scrypto_value(serializer, self, context)
    }
}

impl SerializableScryptoValue for ScryptoValue {}

pub fn serialize_schemaless_scrypto_value<S: Serializer>(
    serializer: S,
    value: &ScryptoValue,
    context: &ScryptoValueSerializationContext,
) -> Result<S::Ok, S::Error> {
    match value {
        // primitive types
        Value::Bool { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::Bool,
            value,
        ),
        Value::I8 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::I8,
            value,
        ),
        Value::I16 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::I16,
            value,
        ),
        Value::I32 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::I32,
            value,
        ),
        Value::I64 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode I64s as strings
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                ValueKind::I64,
                &value.to_string(),
            )
        }
        Value::I128 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode I128 as strings
            // Moreover, I128 isn't supported by the JSON serializer
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                ValueKind::I128,
                &value.to_string(),
            )
        }
        Value::U8 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::U8,
            value,
        ),
        Value::U16 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::U16,
            value,
        ),
        Value::U32 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::U32,
            value,
        ),
        Value::U64 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode U64s as strings
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                ValueKind::U64,
                &value.to_string(),
            )
        }
        Value::U128 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode U128 as strings
            // Moreover, U128 isn't supported by the JSON serializer
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                ValueKind::U128,
                &value.to_string(),
            )
        }
        Value::String { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::String,
            value,
        ),
        Value::Tuple { fields } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::Tuple,
            &fields.serializable(*context),
        ),
        Value::Enum {
            discriminator,
            fields,
        } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::Enum,
            &EnumVariant {
                discriminator: *discriminator,
                fields,
            }
            .serializable(*context),
        ),
        Value::Array {
            element_value_kind,
            elements,
        } => serialize_value_with_element_type(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::Array,
            *element_value_kind,
            &ArrayValue {
                element_value_kind,
                elements,
            }
            .serializable(*context),
        ),
        Value::Map {
            key_value_kind,
            value_value_kind,
            entries,
        } => serialize_value_with_kv_types(
            ValueEncoding::NoType,
            serializer,
            context,
            ValueKind::Map,
            *key_value_kind,
            *value_value_kind,
            &MapValue { entries }.serializable(*context),
        ),
        Value::Custom { value } => serialize_custom_value(serializer, value, context),
    }
}

pub struct ArrayValue<'a> {
    element_value_kind: &'a ScryptoValueKind,
    elements: &'a [ScryptoValue],
}

impl<'a, 'b> ContextualSerialize<ScryptoValueSerializationContext<'a>> for ArrayValue<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueSerializationContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        if *self.element_value_kind == ValueKind::U8 {
            let length = self.elements.len();
            let mut bytes_vec = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
            for element in self.elements {
                let Value::U8 { value: byte } = element else {
                    return Err(Error::custom("An SBOR array of U8 contained a non-U8 value"));
                };
                bytes_vec.push(*byte);
            }
            serialize_hex(serializer, &bytes_vec)
        } else {
            serialize_schemaless_scrypto_value_slice(serializer, self.elements, context)
        }
    }
}

pub struct MapValue<'a> {
    entries: &'a [(ScryptoValue, ScryptoValue)],
}

impl<'a, 'b> ContextualSerialize<ScryptoValueSerializationContext<'a>> for MapValue<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueSerializationContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        // Serialize map into JSON array instead of JSON map because SBOR map is a superset of JSON map.
        let mut tuple = serializer.serialize_tuple(self.entries.len())?;
        for entry in self.entries {
            let t = ScryptoValue::Tuple {
                fields: vec![entry.0.clone(), entry.1.clone()],
            };
            tuple.serialize_element(&t.serializable(*context))?;
        }
        tuple.end()
    }
}

pub struct BytesValue<'a> {
    bytes: &'a [u8],
}

impl<'a, 'b> ContextualSerialize<ScryptoValueSerializationContext<'a>> for BytesValue<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        _context: &ScryptoValueSerializationContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        serialize_hex(serializer, &self.bytes)
    }
}

fn serialize_hex<S: Serializer>(serializer: S, slice: &[u8]) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(1))?;
    map.serialize_entry("hex", &hex::encode(slice))?;
    map.end()
}

fn value_kind_to_string(value_kind: &ScryptoValueKind) -> String {
    display_value_kind(value_kind).to_string()
}

pub struct EnumVariant<'a> {
    discriminator: u8,
    fields: &'a [ScryptoValue],
}

impl<'a, 'b> ContextualSerialize<ScryptoValueSerializationContext<'a>> for EnumVariant<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueSerializationContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("variant", &self.discriminator)?;
        map.serialize_entry("fields", &self.fields.serializable(*context))?;
        map.end()
    }
}

impl<'a> ContextualSerialize<ScryptoValueSerializationContext<'a>> for [ScryptoValue] {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueSerializationContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        serialize_schemaless_scrypto_value_slice(serializer, self, context)
    }
}

impl SerializableScryptoValue for [ScryptoValue] {}

pub fn serialize_schemaless_scrypto_value_slice<S: Serializer>(
    serializer: S,
    elements: &[ScryptoValue],
    context: &ScryptoValueSerializationContext,
) -> Result<S::Ok, S::Error> {
    // Tuple is the serde type corresponding to a known-length list
    // See https://serde.rs/data-model.html
    let mut tuple = serializer.serialize_tuple(elements.len())?;
    for element in elements {
        tuple.serialize_element(&element.serializable(*context))?;
    }
    tuple.end()
}

pub fn serialize_custom_value<S: Serializer>(
    serializer: S,
    value: &ScryptoCustomValue,
    context: &ScryptoValueSerializationContext,
) -> Result<S::Ok, S::Error> {
    match value {
        ScryptoCustomValue::Address(value) => {
            let string_address =
                format!("{}", value.display(context.display_context.bech32_encoder));

            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                ScryptoCustomValueKind::Address,
                &string_address,
            )
        }
        ScryptoCustomValue::Own(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::Own,
            &format!("{}", hex::encode(value.id())),
        ),
        ScryptoCustomValue::Decimal(value) => serialize_value(
            // The fact it's a decimal number will be obvious from context, so favour simplicity over verbosity
            ValueEncoding::NoType,
            serializer,
            context,
            ScryptoCustomValueKind::Decimal,
            &format!("{}", value),
        ),
        ScryptoCustomValue::PreciseDecimal(value) => serialize_value(
            // The fact it's a decimal number will be obvious from context, so favour simplicity over verbosity
            ValueEncoding::NoType,
            serializer,
            context,
            ScryptoCustomValueKind::PreciseDecimal,
            &format!("{}", value),
        ),
        ScryptoCustomValue::NonFungibleLocalId(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::NonFungibleLocalId,
            &format!("{}", value),
        ),
        ScryptoCustomValue::InternalRef(InternalRef(object_id)) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::Reference,
            &format!("{}", hex::encode(object_id)),
        ),
    }
}

/// We encode custom types in one of two ways:
/// - As a tagged object { "type": "TypeName", "value": X }
/// - As a transparent value (ie without a wrapper)
///
/// For invertible JSON, we always use the former.
/// For simple JSON, we often use the latter, where the type is obvious or unnecessary information.
#[derive(Debug, Eq, PartialEq)]
enum ValueEncoding {
    NoType,
    WithType,
}

fn serialize_value<S: Serializer, T: Serialize + ?Sized, K: Into<ScryptoValueKind>>(
    value_encoding_type: ValueEncoding,
    serializer: S,
    context: &ScryptoValueSerializationContext,
    value_kind: K,
    value: &T,
) -> Result<S::Ok, S::Error> {
    if context.serialization_type == ScryptoValueSerializationType::Simple
        && value_encoding_type == ValueEncoding::NoType
    {
        value.serialize(serializer)
    } else {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &value_kind_to_string(&value_kind.into()))?;
        map.serialize_entry("value", value)?;
        map.end()
    }
}

fn serialize_value_with_element_type<
    S: Serializer,
    T: Serialize + ?Sized,
    K: Into<ScryptoValueKind>,
>(
    value_encoding_type: ValueEncoding,
    serializer: S,
    context: &ScryptoValueSerializationContext,
    value_kind: K,
    element_value_kind: K,
    value: &T,
) -> Result<S::Ok, S::Error> {
    if context.serialization_type == ScryptoValueSerializationType::Simple
        && value_encoding_type == ValueEncoding::NoType
    {
        value.serialize(serializer)
    } else {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &value_kind_to_string(&value_kind.into()))?;
        map.serialize_entry(
            "element_type",
            &value_kind_to_string(&element_value_kind.into()),
        )?;
        map.serialize_entry("value", value)?;
        map.end()
    }
}

fn serialize_value_with_kv_types<
    S: Serializer,
    T: Serialize + ?Sized,
    K: Into<ScryptoValueKind>,
>(
    value_encoding_type: ValueEncoding,
    serializer: S,
    context: &ScryptoValueSerializationContext,
    value_kind: K,
    key_value_kind: K,
    value_value_kind: K,
    value: &T,
) -> Result<S::Ok, S::Error> {
    if context.serialization_type == ScryptoValueSerializationType::Simple
        && value_encoding_type == ValueEncoding::NoType
    {
        value.serialize(serializer)
    } else {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &value_kind_to_string(&value_kind.into()))?;
        map.serialize_entry("key_type", &value_kind_to_string(&key_value_kind.into()))?;
        map.serialize_entry(
            "value_type",
            &value_kind_to_string(&value_value_kind.into()),
        )?;
        map.serialize_entry("value", value)?;
        map.end()
    }
}

#[cfg(test)]
#[cfg(feature = "serde")] // Ensures that VS Code runs this module with the features serde tag!
mod tests {
    use super::*;
    use crate::address::Bech32Encoder;
    use crate::*;
    use sbor::rust::vec;
    use serde::Serialize;
    use serde_json::{json, to_string, to_value, Value as JsonValue};

    use crate::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};

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
            "type": "Address",
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
            "type": "Address",
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
                Value::Bool { value: true },
                Value::U8 { value: 5 },
                Value::U16 { value: 5 },
                Value::U32 { value: 5 },
                Value::U64 { value: u64::MAX },
                Value::U128 {
                    value: 9912313323213,
                },
                Value::I8 { value: -5 },
                Value::I16 { value: -5 },
                Value::I32 { value: -5 },
                Value::I64 { value: -5 },
                Value::I128 { value: -5 },
                Value::Array {
                    element_value_kind: ValueKind::U8,
                    elements: vec![Value::U8 { value: 0x3a }, Value::U8 { value: 0x92 }],
                },
                Value::Array {
                    element_value_kind: ValueKind::U32,
                    elements: vec![Value::U32 { value: 153 }, Value::U32 { value: 62 }],
                },
                Value::Enum {
                    discriminator: 0,
                    fields: vec![],
                },
                Value::Enum {
                    discriminator: 1,
                    fields: vec![Value::U32 { value: 153 }],
                },
                Value::Enum {
                    discriminator: 2,
                    fields: vec![Value::U32 { value: 153 }, Value::Bool { value: true }],
                },
                Value::Map {
                    key_value_kind: ValueKind::U32,
                    value_value_kind: ValueKind::U32,
                    entries: vec![(Value::U32 { value: 153 }, Value::U32 { value: 62 })],
                },
                Value::Tuple {
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
                            value: ScryptoCustomValue::NonFungibleLocalId(
                                NonFungibleLocalId::integer(123),
                            ),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::NonFungibleLocalId(
                                NonFungibleLocalId::bytes(vec![0x23, 0x45]).unwrap(),
                            ),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::NonFungibleLocalId(
                                NonFungibleLocalId::uuid(0x1f52cb1e_86c4_47ae_9847_9cdb14662ebd)
                                    .unwrap(),
                            ),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::InternalRef(InternalRef(
                                [0; OBJECT_ID_LENGTH],
                            )),
                        },
                    ],
                },
            ],
        };

        let expected_simple = json!([
            true,
            5,
            5,
            5,
            "18446744073709551615",
            "9912313323213",
            -5,
            -5,
            -5,
            "-5",
            "-5",
            {
                "hex": "3a92"
            },
            [
                153,
                62
            ],
            {
                "fields": [],
                "variant": 0
            },
            {
                "fields": [
                    153
                ],
                "variant": 1
            },
            {
                "fields": [
                    153,
                    true
                ],
                "variant": 2
            },
            [
                [
                    153,
                    62
                ]
            ],
            [
                "resource_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs6d89k",
                {
                    "type": "Own",
                    "value": "00000000000000000000000000000000000000000000000000000000000000"
                },
                "1",
                "0.01",
                "0",
                {
                    "type": "NonFungibleLocalId",
                    "value": "<hello>"
                },
                {
                    "type": "NonFungibleLocalId",
                    "value": "#123#"
                },
                {
                    "type": "NonFungibleLocalId",
                    "value": "[2345]"
                },
                {
                    "type": "NonFungibleLocalId",
                    "value": "{1f52cb1e-86c4-47ae-9847-9cdb14662ebd}"
                },
                {
                    "type": "Reference",
                    "value": "00000000000000000000000000000000000000000000000000000000000000"
                }
            ]
        ]);

        let expected_invertible = json!({
            "type": "Tuple",
            "value": [
                {
                    "type": "Bool",
                    "value": true
                },
                {
                    "type": "U8",
                    "value": 5
                },
                {
                    "type": "U16",
                    "value": 5
                },
                {
                    "type": "U32",
                    "value": 5
                },
                {
                    "type": "U64",
                    "value": "18446744073709551615"
                },
                {
                    "type": "U128",
                    "value": "9912313323213"
                },
                {
                    "type": "I8",
                    "value": -5
                },
                {
                    "type": "I16",
                    "value": -5
                },
                {
                    "type": "I32",
                    "value": -5
                },
                {
                    "type": "I64",
                    "value": "-5"
                },
                {
                    "type": "I128",
                    "value": "-5"
                },
                {
                    "element_type": "U8",
                    "type": "Array",
                    "value": {
                        "hex": "3a92"
                    }
                },
                {
                    "element_type": "U32",
                    "type": "Array",
                    "value": [
                        {
                            "type": "U32",
                            "value": 153
                        },
                        {
                            "type": "U32",
                            "value": 62
                        }
                    ]
                },
                {
                    "type": "Enum",
                    "value": {
                        "fields": [],
                        "variant": 0
                    }
                },
                {
                    "type": "Enum",
                    "value": {
                        "fields": [
                            {
                                "type": "U32",
                                "value": 153
                            }
                        ],
                        "variant": 1
                    }
                },
                {
                    "type": "Enum",
                    "value": {
                        "fields": [
                            {
                                "type": "U32",
                                "value": 153
                            },
                            {
                                "type": "Bool",
                                "value": true
                            }
                        ],
                        "variant": 2
                    }
                },
                {
                    "key_type": "U32",
                    "type": "Map",
                    "value": [
                        {
                            "type": "Tuple",
                            "value": [
                                {
                                    "type": "U32",
                                    "value": 153
                                },
                                {
                                    "type": "U32",
                                    "value": 62
                                }
                            ]
                        }
                    ],
                    "value_type": "U32"
                },
                {
                    "type": "Tuple",
                    "value": [
                        {
                            "type": "Address",
                            "value": "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz8qety"
                        },
                        {
                            "type": "Own",
                            "value": "00000000000000000000000000000000000000000000000000000000000000"
                        },
                        {
                            "type": "Decimal",
                            "value": "1"
                        },
                        {
                            "type": "Decimal",
                            "value": "0.01"
                        },
                        {
                            "type": "PreciseDecimal",
                            "value": "0"
                        },
                        {
                            "type": "NonFungibleLocalId",
                            "value": "<hello>"
                        },
                        {
                            "type": "NonFungibleLocalId",
                            "value": "#123#"
                        },
                        {
                            "type": "NonFungibleLocalId",
                            "value": "[2345]"
                        },
                        {
                            "type": "NonFungibleLocalId",
                            "value": "{1f52cb1e-86c4-47ae-9847-9cdb14662ebd}"
                        },
                        {
                            "type": "Reference",
                            "value": "00000000000000000000000000000000000000000000000000000000000000"
                        },
                    ]
                }
            ]
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
        let bytes = scrypto_encode(&value).unwrap();
        let scrypto_value = scrypto_decode::<ScryptoValue>(&bytes).unwrap();

        let serializable = scrypto_value.simple_serializable(context);

        assert_json_eq(serializable, expected);
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
        let bytes = scrypto_encode(&value).unwrap();
        let scrypto_value = scrypto_decode::<ScryptoValue>(&bytes).unwrap();

        let serializable = scrypto_value.invertible_serializable(context);

        assert_json_eq(serializable, expected);
    }
}
