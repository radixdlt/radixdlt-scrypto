use super::types::ManifestExpression;
use crate::api::types::*;
use crate::data::*;
use sbor::rust::format;
use sbor::rust::vec;
use serde::ser::*;
use serde::*;
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
pub struct ScryptoValueFormattingContext<'a> {
    pub serialization_type: ScryptoValueSerializationType,
    pub display_context: ValueFormattingContext<'a>,
}

impl<'a> ScryptoValueFormattingContext<'a> {
    pub fn simple(display_context: ValueFormattingContext<'a>) -> Self {
        Self {
            serialization_type: ScryptoValueSerializationType::Simple,
            display_context,
        }
    }

    pub fn invertible(display_context: ValueFormattingContext<'a>) -> Self {
        Self {
            serialization_type: ScryptoValueSerializationType::Invertible,
            display_context,
        }
    }
}

pub trait SerializableScryptoValue:
    for<'a> ContextualSerialize<ScryptoValueFormattingContext<'a>>
{
    fn simple_serializable<'a, 'b, TContext: Into<ValueFormattingContext<'b>>>(
        &'a self,
        context: TContext,
    ) -> ContextSerializable<'a, Self, ScryptoValueFormattingContext<'b>> {
        self.serializable(ScryptoValueFormattingContext::simple(context.into()))
    }

    fn invertible_serializable<'a, 'b, TContext: Into<ValueFormattingContext<'b>>>(
        &'a self,
        context: TContext,
    ) -> ContextSerializable<'a, Self, ScryptoValueFormattingContext<'b>> {
        self.serializable(ScryptoValueFormattingContext::invertible(context.into()))
    }
}

impl<'a> ContextualSerialize<ScryptoValueFormattingContext<'a>> for ScryptoValue {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        serialize_schemaless_scrypto_value(serializer, self, context)
    }
}

impl SerializableScryptoValue for ScryptoValue {}

pub fn serialize_schemaless_scrypto_value<S: Serializer>(
    serializer: S,
    value: &ScryptoValue,
    context: &ScryptoValueFormattingContext,
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

impl<'a, 'b> ContextualSerialize<ScryptoValueFormattingContext<'a>> for ArrayValue<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        if *self.element_value_kind == ValueKind::U8 {
            let length = self.elements.len();
            let mut bytes_vec = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
            for element in self.elements {
                let Value::U8 { value: byte } = element else {
                    return Err(ser::Error::custom("An SBOR array of U8 contained a non-U8 value"));
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

impl<'a, 'b> ContextualSerialize<ScryptoValueFormattingContext<'a>> for MapValue<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
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

impl<'a, 'b> ContextualSerialize<ScryptoValueFormattingContext<'a>> for BytesValue<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        _context: &ScryptoValueFormattingContext<'a>,
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

impl<'a, 'b> ContextualSerialize<ScryptoValueFormattingContext<'a>> for EnumVariant<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("variant", &self.discriminator)?;
        map.serialize_entry("fields", &self.fields.serializable(*context))?;
        map.end()
    }
}

impl<'a> ContextualSerialize<ScryptoValueFormattingContext<'a>> for [ScryptoValue] {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        serialize_schemaless_scrypto_value_slice(serializer, self, context)
    }
}

impl SerializableScryptoValue for [ScryptoValue] {}

pub fn serialize_schemaless_scrypto_value_slice<S: Serializer>(
    serializer: S,
    elements: &[ScryptoValue],
    context: &ScryptoValueFormattingContext,
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
    context: &ScryptoValueFormattingContext,
) -> Result<S::Ok, S::Error> {
    match value {
        // RE interpreted types
        ScryptoCustomValue::PackageAddress(value) => {
            let string_address =
                format!("{}", value.display(context.display_context.bech32_encoder));
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                ScryptoCustomValueKind::PackageAddress,
                &string_address,
            )
        }
        ScryptoCustomValue::ComponentAddress(value) => {
            let string_address =
                format!("{}", value.display(context.display_context.bech32_encoder));
            serialize_value(
                // The fact it's an address is obvious, so favour simplicity over verbosity
                ValueEncoding::NoType,
                serializer,
                context,
                ScryptoCustomValueKind::ComponentAddress,
                &string_address,
            )
        }
        ScryptoCustomValue::ResourceAddress(value) => {
            let string_address =
                format!("{}", value.display(context.display_context.bech32_encoder));
            serialize_value(
                // The fact it's an address is obvious, so favour simplicity over verbosity
                ValueEncoding::NoType,
                serializer,
                context,
                ScryptoCustomValueKind::ResourceAddress,
                &string_address,
            )
        }
        ScryptoCustomValue::Own(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::Own,
            &format!("{:?}", value), // TODO: fix syntax
        ),
        // TX interpreted types
        ScryptoCustomValue::Bucket(value) => {
            if let Some(name) = context.display_context.get_bucket_name(&value) {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomValueKind::Bucket,
                    name,
                )
            } else {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomValueKind::Bucket,
                    &value.0,
                )
            }
        }
        ScryptoCustomValue::Proof(value) => {
            if let Some(name) = context.display_context.get_proof_name(&value) {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomValueKind::Proof,
                    name,
                )
            } else {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomValueKind::Proof,
                    &value.0,
                )
            }
        }
        ScryptoCustomValue::Expression(value) => serialize_value(
            // The fact it's an expression isn't so relevant, so favour simplicity over verbosity
            ValueEncoding::NoType,
            serializer,
            context,
            ScryptoCustomValueKind::Expression,
            &format!(
                "{}",
                match value {
                    ManifestExpression::EntireWorktop => "ENTIRE_WORKTOP",
                    ManifestExpression::EntireAuthZone => "ENTIRE_AUTH_ZONE",
                }
            ),
        ),
        ScryptoCustomValue::Blob(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::Blob,
            &format!("{}", hex::encode(&value.0 .0)),
        ),
        // Uninterpreted
        ScryptoCustomValue::Hash(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::Hash,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EcdsaSecp256k1PublicKey(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::EcdsaSecp256k1PublicKey,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EcdsaSecp256k1Signature(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::EcdsaSecp256k1Signature,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EddsaEd25519PublicKey(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::EddsaEd25519PublicKey,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EddsaEd25519Signature(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomValueKind::EddsaEd25519Signature,
            &format!("{}", value),
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
    }
}

impl<'a> ContextualSerialize<ScryptoValueFormattingContext<'a>> for NonFungibleGlobalId {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(
            &self
                .resource_address()
                .display(context.display_context.bech32_encoder)
                .to_string(),
        )?;
        tuple.serialize_element(&self.local_id().to_string())?;
        tuple.end()
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
    context: &ScryptoValueFormattingContext,
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
    context: &ScryptoValueFormattingContext,
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
    context: &ScryptoValueFormattingContext,
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
    use radix_engine_derive::*;
    use sbor::rust::collections::HashMap;
    use sbor::rust::vec;
    use serde::Serialize;
    use serde_json::{json, to_string, to_value, Value as JsonValue};

    use crate::{
        address::NO_NETWORK,
        api::types::ResourceAddress,
        constants::RADIX_TOKEN,
        data::{scrypto_decode, scrypto_encode, ScryptoValue},
    };

    #[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
        let value = RADIX_TOKEN;

        let no_network_radix_token_address = RADIX_TOKEN.display(NO_NETWORK).to_string();

        let expected = json!(no_network_radix_token_address);
        let expected_invertible =
            json!({ "type": "ResourceAddress", "value": no_network_radix_token_address });

        assert_simple_json_matches(&value, ValueFormattingContext::no_context(), expected);
        assert_invertible_json_matches(
            &value,
            ValueFormattingContext::no_context(),
            expected_invertible,
        );
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_address_encoding_with_network() {
        let value = RADIX_TOKEN;
        let encoder = Bech32Encoder::for_simulator();

        let radix_token_address = RADIX_TOKEN.display(&encoder).to_string();

        let expected_simple = json!(radix_token_address);
        let expected_invertible =
            json!({ "type": "ResourceAddress", "value": radix_token_address });

        assert_simple_json_matches(&value, &encoder, expected_simple);
        assert_invertible_json_matches(&value, &encoder, expected_invertible);
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_complex_encoding_with_network() {
        use crate::{
            constants::{ACCOUNT_PACKAGE, EPOCH_MANAGER, FAUCET_COMPONENT},
            crypto::{
                EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature, EddsaEd25519PublicKey,
                EddsaEd25519Signature,
            },
            data::types::{
                ManifestBlobRef, ManifestBucket, ManifestExpression, ManifestProof, Own,
            },
            math::{Decimal, PreciseDecimal},
        };

        let encoder = Bech32Encoder::for_simulator();
        let account_package_address = ACCOUNT_PACKAGE.display(&encoder).to_string();
        let faucet_address = FAUCET_COMPONENT.display(&encoder).to_string();
        let radix_token_address = RADIX_TOKEN.display(&encoder).to_string();
        let epoch_manager_address = EPOCH_MANAGER.display(&encoder).to_string();

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
                            value: ScryptoCustomValue::PackageAddress(ACCOUNT_PACKAGE),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::ComponentAddress(FAUCET_COMPONENT),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::ResourceAddress(RADIX_TOKEN),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::ComponentAddress(EPOCH_MANAGER),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Own(Own::Vault([0; 36])),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Bucket(ManifestBucket(1)), // Will be mapped by context to "Hello"
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Bucket(ManifestBucket(10)),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Proof(ManifestProof(2)),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Expression(
                                ManifestExpression::EntireWorktop,
                            ),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Blob(ManifestBlobRef(Hash([0; 32]))),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Hash(Hash([0; 32])),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::EcdsaSecp256k1PublicKey(
                                EcdsaSecp256k1PublicKey([0; 33]),
                            ),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::EcdsaSecp256k1Signature(
                                EcdsaSecp256k1Signature([0; 65]),
                            ),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::EddsaEd25519PublicKey(
                                EddsaEd25519PublicKey([0; 32]),
                            ),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::EddsaEd25519Signature(
                                EddsaEd25519Signature([0; 64]),
                            ),
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
            { "hex": "3a92" },
            [153, 62],
            { "variant": 0, "fields": [] },
            { "variant": 1, "fields": [153] },
            { "variant": 2, "fields": [153, true] },
            [[153, 62]],
            [
                account_package_address,
                faucet_address,
                radix_token_address,
                epoch_manager_address,
                { "type": "Own", "value": "Vault([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])" },
                { "type": "Bucket", "value": "Hello" },
                { "type": "Bucket", "value": 10 },
                { "type": "Proof", "value": 2 },
                "ENTIRE_WORKTOP",
                { "type": "Blob", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "Hash", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EcdsaSecp256k1PublicKey", "value": "000000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EcdsaSecp256k1Signature", "value": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EddsaEd25519PublicKey", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EddsaEd25519Signature", "value": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                "1",
                "0.01",
                "0",
                { "type": "NonFungibleLocalId", "value": "<hello>" },
                { "type": "NonFungibleLocalId", "value": "#123#" },
                { "type": "NonFungibleLocalId", "value": "[2345]" },
                { "type": "NonFungibleLocalId", "value": "{1f52cb1e-86c4-47ae-9847-9cdb14662ebd}" },
            ]
        ]);

        let expected_invertible = json!({
            "type": "Tuple",
            "value": [
                { "type": "Bool", "value": true },
                { "type": "U8", "value": 5 },
                { "type": "U16", "value": 5 },
                { "type": "U32", "value": 5 },
                { "type": "U64", "value": "18446744073709551615" },
                { "type": "U128", "value": "9912313323213" },
                { "type": "I8", "value": -5 },
                { "type": "I16", "value": -5 },
                { "type": "I32", "value": -5 },
                { "type": "I64", "value": "-5" },
                { "type": "I128", "value": "-5" },
                { "type": "Array", "element_type": "U8", "value": { "hex": "3a92" } },
                {
                    "type": "Array",
                    "element_type": "U32",
                    "value": [
                        { "type": "U32", "value": 153 },
                        { "type": "U32", "value": 62 },
                    ]
                },
                { "type": "Enum", "value": { "variant": 0, "fields": [] } },
                { "type": "Enum", "value": { "variant": 1, "fields": [{ "type": "U32", "value": 153 }] } },
                { "type": "Enum", "value": { "variant": 2, "fields": [{ "type": "U32", "value": 153 }, { "type": "Bool", "value": true }] } },
                { "type": "Map", "key_type": "U32", "value_type": "U32", "value": [{"type":"Tuple","value":[{"type":"U32","value":153},{"type":"U32","value":62}]}] },
                {
                    "type": "Tuple",
                    "value": [
                        { "type": "PackageAddress", "value": account_package_address },
                        { "type": "ComponentAddress", "value": faucet_address },
                        { "type": "ResourceAddress", "value": radix_token_address },
                        { "type": "ComponentAddress", "value": epoch_manager_address },
                        { "type": "Own", "value": "Vault([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])" },
                        { "type": "Bucket", "value": "Hello" },
                        { "type": "Bucket", "value": 10 },
                        { "type": "Proof", "value": 2 },
                        { "type": "Expression", "value": "ENTIRE_WORKTOP" },
                        { "type": "Blob", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "Hash", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EcdsaSecp256k1PublicKey", "value": "000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EcdsaSecp256k1Signature", "value": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EddsaEd25519PublicKey", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EddsaEd25519Signature", "value": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "Decimal", "value": "1" },
                        { "type": "Decimal", "value": "0.01" },
                        { "type": "PreciseDecimal", "value": "0" },
                        { "type": "NonFungibleLocalId", "value": "<hello>" },
                        { "type": "NonFungibleLocalId", "value": "#123#" },
                        { "type": "NonFungibleLocalId", "value": "[2345]" },
                        { "type": "NonFungibleLocalId", "value": "{1f52cb1e-86c4-47ae-9847-9cdb14662ebd}" },
                    ]
                }
            ]
        });

        let mut bucket_names = HashMap::new();
        bucket_names.insert(ManifestBucket(1), "Hello".to_owned());
        let proof_names = HashMap::new();

        let context = ValueFormattingContext::with_manifest_context(
            Some(&encoder),
            &bucket_names,
            &proof_names,
        );

        assert_simple_json_matches(&value, context, expected_simple);
        assert_invertible_json_matches(&value, context, expected_invertible);
    }

    fn assert_simple_json_matches<'a, T: ScryptoEncode, C: Into<ValueFormattingContext<'a>>>(
        value: &T,
        context: C,
        expected: JsonValue,
    ) {
        let bytes = scrypto_encode(&value).unwrap();
        let scrypto_value = scrypto_decode::<ScryptoValue>(&bytes).unwrap();

        let serializable = scrypto_value.simple_serializable(context);

        assert_json_eq(serializable, expected);
    }

    fn assert_invertible_json_matches<'a, T: ScryptoEncode, C: Into<ValueFormattingContext<'a>>>(
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
