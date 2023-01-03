use crate::api::types::*;
use crate::data::*;
use sbor::rust::format;
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
        SborValue::Unit => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::Unit,
            &(),
        ),
        SborValue::Bool { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::Bool,
            value,
        ),
        SborValue::I8 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::I8,
            value,
        ),
        SborValue::I16 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::I16,
            value,
        ),
        SborValue::I32 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::I32,
            value,
        ),
        SborValue::I64 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode I64s as strings
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::I64,
                &value.to_string(),
            )
        }
        SborValue::I128 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode I128 as strings
            // Moreover, I128 isn't supported by the JSON serializer
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::I128,
                &value.to_string(),
            )
        }
        SborValue::U8 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::U8,
            value,
        ),
        SborValue::U16 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::U16,
            value,
        ),
        SborValue::U32 { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::U32,
            value,
        ),
        SborValue::U64 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode U64s as strings
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::U64,
                &value.to_string(),
            )
        }
        SborValue::U128 { value } => {
            // Javascript only safely decodes JSON integers up to 2^53
            // So to be safe, we encode U128 as strings
            // Moreover, U128 isn't supported by the JSON serializer
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::U128,
                &value.to_string(),
            )
        }
        SborValue::String { value } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::String,
            value,
        ),
        SborValue::Tuple { fields } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::Tuple,
            &fields.serializable(*context),
        ),
        SborValue::Enum {
            discriminator,
            fields,
        } => serialize_value(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::Enum,
            &EnumVariant {
                discriminator,
                fields,
            }
            .serializable(*context),
        ),
        SborValue::Array {
            element_type_id,
            elements,
        } => serialize_value_with_element_type(
            ValueEncoding::NoType,
            serializer,
            context,
            SborTypeId::Array,
            *element_type_id,
            &ArrayValue {
                element_type_id,
                elements,
            }
            .serializable(*context),
        ),
        SborValue::Custom { value } => serialize_custom_value(serializer, value, context),
    }
}

pub struct ArrayValue<'a> {
    element_type_id: &'a ScryptoSborTypeId,
    elements: &'a [ScryptoValue],
}

impl<'a, 'b> ContextualSerialize<ScryptoValueFormattingContext<'a>> for ArrayValue<'b> {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        if *self.element_type_id == SborTypeId::U8 {
            let length = self.elements.len();
            let mut bytes_vec = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
            for element in self.elements {
                let SborValue::U8 { value: byte } = element else {
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

fn type_id_to_string(type_id: &ScryptoSborTypeId) -> String {
    display_type_id(type_id).to_string()
}

pub struct EnumVariant<'a> {
    discriminator: &'a str,
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
        // Global address types
        ScryptoCustomValue::PackageAddress(value) => {
            let string_address =
                format!("{}", value.display(context.display_context.bech32_encoder));
            serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                ScryptoCustomTypeId::PackageAddress,
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
                ScryptoCustomTypeId::ComponentAddress,
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
                ScryptoCustomTypeId::ResourceAddress,
                &string_address,
            )
        }
        ScryptoCustomValue::SystemAddress(value) => {
            let string_address =
                format!("{}", value.display(context.display_context.bech32_encoder));
            serialize_value(
                // The fact it's an address is obvious, so favour simplicity over verbosity
                ValueEncoding::NoType,
                serializer,
                context,
                ScryptoCustomTypeId::SystemAddress,
                &string_address,
            )
        }
        // RE node types
        ScryptoCustomValue::Own(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::Own,
            &hex::encode(value.to_vec()),
        ),
        ScryptoCustomValue::Component(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::Component,
            &hex::encode(value),
        ),
        ScryptoCustomValue::KeyValueStore(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::KeyValueStore,
            &hex::encode(value),
        ),
        ScryptoCustomValue::Bucket(value) => {
            if let Some(name) = context.display_context.get_bucket_name(&value) {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomTypeId::Bucket,
                    name,
                )
            } else {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomTypeId::Bucket,
                    value,
                )
            }
        }
        ScryptoCustomValue::Proof(value) => {
            if let Some(name) = context.display_context.get_proof_name(&value) {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomTypeId::Proof,
                    name,
                )
            } else {
                serialize_value(
                    ValueEncoding::WithType,
                    serializer,
                    context,
                    ScryptoCustomTypeId::Proof,
                    value,
                )
            }
        }
        // Other interpreted types
        ScryptoCustomValue::Expression(value) => serialize_value(
            // The fact it's an expression isn't so relevant, so favour simplicity over verbosity
            ValueEncoding::NoType,
            serializer,
            context,
            ScryptoCustomTypeId::Expression,
            &format!("{}", value),
        ),
        ScryptoCustomValue::Blob(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::Blob,
            &format!("{}", value),
        ),
        ScryptoCustomValue::NonFungibleAddress(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::NonFungibleAddress,
            &value.serializable(*context),
        ),
        // Uninterpreted
        ScryptoCustomValue::Hash(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::Hash,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EcdsaSecp256k1PublicKey(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::EcdsaSecp256k1PublicKey,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EcdsaSecp256k1Signature(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::EcdsaSecp256k1Signature,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EddsaEd25519PublicKey(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::EddsaEd25519PublicKey,
            &format!("{}", value),
        ),
        ScryptoCustomValue::EddsaEd25519Signature(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::EddsaEd25519Signature,
            &format!("{}", value),
        ),
        ScryptoCustomValue::Decimal(value) => serialize_value(
            // The fact it's a decimal number will be obvious from context, so favour simplicity over verbosity
            ValueEncoding::NoType,
            serializer,
            context,
            ScryptoCustomTypeId::Decimal,
            &format!("{}", value),
        ),
        ScryptoCustomValue::PreciseDecimal(value) => serialize_value(
            // The fact it's a decimal number will be obvious from context, so favour simplicity over verbosity
            ValueEncoding::NoType,
            serializer,
            context,
            ScryptoCustomTypeId::PreciseDecimal,
            &format!("{}", value),
        ),
        ScryptoCustomValue::NonFungibleId(value) => serialize_value(
            ValueEncoding::WithType,
            serializer,
            context,
            ScryptoCustomTypeId::NonFungibleId,
            &value.serializable(*context),
        ),
    }
}

impl<'a> ContextualSerialize<ScryptoValueFormattingContext<'a>> for NonFungibleAddress {
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
        tuple.serialize_element(&self.non_fungible_id().serializable(*context))?;
        tuple.end()
    }
}

impl<'a> ContextualSerialize<ScryptoValueFormattingContext<'a>> for NonFungibleId {
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &ScryptoValueFormattingContext<'a>,
    ) -> Result<S::Ok, S::Error> {
        match self {
            NonFungibleId::String(value) => serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::String,
                value,
            ),
            NonFungibleId::U32(value) => serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::U32,
                value,
            ),
            NonFungibleId::U64(value) => serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::U64,
                &value.to_string(),
            ),
            NonFungibleId::Bytes(value) => serialize_value_with_element_type(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::Array,
                SborTypeId::U8,
                &BytesValue { bytes: value }.serializable(*context),
            ),
            NonFungibleId::UUID(value) => serialize_value(
                ValueEncoding::NoType,
                serializer,
                context,
                SborTypeId::U128,
                &value.to_string(),
            ),
        }
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

fn serialize_value<S: Serializer, T: Serialize + ?Sized, TypeId: Into<ScryptoSborTypeId>>(
    value_encoding_type: ValueEncoding,
    serializer: S,
    context: &ScryptoValueFormattingContext,
    type_id: TypeId,
    value: &T,
) -> Result<S::Ok, S::Error> {
    if context.serialization_type == ScryptoValueSerializationType::Simple
        && value_encoding_type == ValueEncoding::NoType
    {
        value.serialize(serializer)
    } else {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &type_id_to_string(&type_id.into()))?;
        map.serialize_entry("value", value)?;
        map.end()
    }
}

fn serialize_value_with_element_type<
    S: Serializer,
    T: Serialize + ?Sized,
    TypeId: Into<ScryptoSborTypeId>,
>(
    value_encoding_type: ValueEncoding,
    serializer: S,
    context: &ScryptoValueFormattingContext,
    type_id: TypeId,
    element_type_id: TypeId,
    value: &T,
) -> Result<S::Ok, S::Error> {
    if context.serialization_type == ScryptoValueSerializationType::Simple
        && value_encoding_type == ValueEncoding::NoType
    {
        value.serialize(serializer)
    } else {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &type_id_to_string(&type_id.into()))?;
        map.serialize_entry("element_type", &type_id_to_string(&element_type_id.into()))?;
        map.serialize_entry("value", value)?;
        map.end()
    }
}

#[cfg(test)]
#[cfg(feature = "serde")] // Ensures that VS Code runs this module with the features serde tag!
mod tests {
    use super::*;
    use crate::address::Bech32Encoder;
    use radix_engine_derive::scrypto;
    use sbor::rust::collections::HashMap;
    use sbor::rust::vec;
    use serde::Serialize;
    use serde_json::{json, to_string, to_value, Value};

    use crate::{
        address::NO_NETWORK,
        api::types::ResourceAddress,
        constants::RADIX_TOKEN,
        data::{scrypto_decode, scrypto_encode, ScryptoValue},
    };

    #[scrypto(TypeId, Encode, Decode)]
    pub struct Sample {
        pub a: ResourceAddress,
    }

    pub fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
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
            data::types::{Blob, Expression, Own},
            math::{Decimal, PreciseDecimal},
        };

        let encoder = Bech32Encoder::for_simulator();
        let account_package_address = ACCOUNT_PACKAGE.display(&encoder).to_string();
        let faucet_address = FAUCET_COMPONENT.display(&encoder).to_string();
        let radix_token_address = RADIX_TOKEN.display(&encoder).to_string();
        let epoch_manager_address = EPOCH_MANAGER.display(&encoder).to_string();

        let value = ScryptoValue::Tuple {
            fields: vec![
                SborValue::Unit,
                SborValue::Bool { value: true },
                SborValue::U8 { value: 5 },
                SborValue::U16 { value: 5 },
                SborValue::U32 { value: 5 },
                SborValue::U64 { value: u64::MAX },
                SborValue::U128 {
                    value: 9912313323213,
                },
                SborValue::I8 { value: -5 },
                SborValue::I16 { value: -5 },
                SborValue::I32 { value: -5 },
                SborValue::I64 { value: -5 },
                SborValue::I128 { value: -5 },
                SborValue::Array {
                    element_type_id: SborTypeId::U8,
                    elements: vec![SborValue::U8 { value: 0x3a }, SborValue::U8 { value: 0x92 }],
                },
                SborValue::Array {
                    element_type_id: SborTypeId::U32,
                    elements: vec![SborValue::U32 { value: 153 }, SborValue::U32 { value: 62 }],
                },
                SborValue::Enum {
                    discriminator: "VariantUnit".to_string(),
                    fields: vec![],
                },
                SborValue::Enum {
                    discriminator: "VariantSingleValue".to_string(),
                    fields: vec![SborValue::U32 { value: 153 }],
                },
                SborValue::Enum {
                    discriminator: "VariantMultiValues".to_string(),
                    fields: vec![
                        SborValue::U32 { value: 153 },
                        SborValue::Bool { value: true },
                    ],
                },
                SborValue::Tuple {
                    fields: vec![
                        SborValue::Custom {
                            value: ScryptoCustomValue::PackageAddress(ACCOUNT_PACKAGE),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::ComponentAddress(FAUCET_COMPONENT),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::ResourceAddress(RADIX_TOKEN),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::SystemAddress(EPOCH_MANAGER),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Own(Own::Vault([0; 36])),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Component([0; 36]),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::KeyValueStore([0; 36]),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Bucket(1), // Will be mapped by context to "Hello"
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Bucket(10),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Proof(2),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Expression(Expression::entire_worktop()),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Blob(Blob(Hash([0; 32]))),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::NonFungibleAddress(NonFungibleAddress::new(
                                RADIX_TOKEN,
                                NonFungibleId::Bytes(vec![0u8, 2u8]),
                            )),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Hash(Hash([0; 32])),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::EcdsaSecp256k1PublicKey(
                                EcdsaSecp256k1PublicKey([0; 33]),
                            ),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::EcdsaSecp256k1Signature(
                                EcdsaSecp256k1Signature([0; 65]),
                            ),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::EddsaEd25519PublicKey(
                                EddsaEd25519PublicKey([0; 32]),
                            ),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::EddsaEd25519Signature(
                                EddsaEd25519Signature([0; 64]),
                            ),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Decimal(Decimal::ONE),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::Decimal(Decimal::ONE / 100),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::PreciseDecimal(PreciseDecimal::ZERO),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::NonFungibleId(NonFungibleId::String(
                                "hello".to_string(),
                            )),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::NonFungibleId(NonFungibleId::U32(123)),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::NonFungibleId(NonFungibleId::U64(123)),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::NonFungibleId(NonFungibleId::Bytes(vec![
                                0x23, 0x45,
                            ])),
                        },
                        SborValue::Custom {
                            value: ScryptoCustomValue::NonFungibleId(NonFungibleId::UUID(371)),
                        },
                    ],
                },
            ],
        };

        let expected_simple = json!([
            null,
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
            { "variant": "VariantUnit", "fields": [] },
            { "variant": "VariantSingleValue", "fields": [153] },
            { "variant": "VariantMultiValues", "fields": [153, true] },
            [
                account_package_address,
                faucet_address,
                radix_token_address,
                epoch_manager_address,
                { "type": "Own", "value": "000000000000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "Component", "value": "000000000000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "KeyValueStore", "value": "000000000000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "Bucket", "value": "Hello" },
                { "type": "Bucket", "value": 10 },
                { "type": "Proof", "value": 2 },
                "ENTIRE_WORKTOP",
                { "type": "Blob", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "NonFungibleAddress", "value": [radix_token_address, { "hex": "0002" }] },
                { "type": "Hash", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EcdsaSecp256k1PublicKey", "value": "000000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EcdsaSecp256k1Signature", "value": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EddsaEd25519PublicKey", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                { "type": "EddsaEd25519Signature", "value": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                "1",
                "0.01",
                "0",
                { "type": "NonFungibleId", "value": "hello" },
                { "type": "NonFungibleId", "value": 123 },
                { "type": "NonFungibleId", "value": "123" },
                { "type": "NonFungibleId", "value": { "hex": "2345" } },
                { "type": "NonFungibleId", "value": "371" },
            ]
        ]);

        let expected_invertible = json!({
            "type": "Tuple",
            "value": [
                { "type": "Unit", "value": null },
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
                { "type": "Enum", "value": { "variant": "VariantUnit", "fields": [] } },
                { "type": "Enum", "value": { "variant": "VariantSingleValue", "fields": [{ "type": "U32", "value": 153 }] } },
                { "type": "Enum", "value": { "variant": "VariantMultiValues", "fields": [{ "type": "U32", "value": 153 }, { "type": "Bool", "value": true }] } },
                {
                    "type": "Tuple",
                    "value": [
                        { "type": "PackageAddress", "value": account_package_address },
                        { "type": "ComponentAddress", "value": faucet_address },
                        { "type": "ResourceAddress", "value": radix_token_address },
                        { "type": "SystemAddress", "value": epoch_manager_address },
                        { "type": "Own", "value": "000000000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "Component", "value": "000000000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "KeyValueStore", "value": "000000000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "Bucket", "value": "Hello" },
                        { "type": "Bucket", "value": 10 },
                        { "type": "Proof", "value": 2 },
                        { "type": "Expression", "value": "ENTIRE_WORKTOP" },
                        { "type": "Blob", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "NonFungibleAddress", "value": [radix_token_address, { "type": "Array", "element_type": "U8", "value": { "hex": "0002" } }] },
                        { "type": "Hash", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EcdsaSecp256k1PublicKey", "value": "000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EcdsaSecp256k1Signature", "value": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EddsaEd25519PublicKey", "value": "0000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "EddsaEd25519Signature", "value": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
                        { "type": "Decimal", "value": "1" },
                        { "type": "Decimal", "value": "0.01" },
                        { "type": "PreciseDecimal", "value": "0" },
                        { "type": "NonFungibleId", "value": { "type": "String", "value": "hello" } },
                        { "type": "NonFungibleId", "value": { "type": "U32", "value": 123 } },
                        { "type": "NonFungibleId", "value": { "type": "U64", "value": "123" } },
                        { "type": "NonFungibleId", "value": { "type": "Array", "element_type": "U8", "value": { "hex": "2345" } } },
                        { "type": "NonFungibleId", "value": { "type": "U128", "value": "371" } },
                    ]
                }
            ]
        });

        let mut bucket_names = HashMap::new();
        bucket_names.insert(1, "Hello".to_owned());
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
        expected: Value,
    ) {
        let bytes = scrypto_encode(&value).unwrap();
        let scrypto_value = scrypto_decode::<ScryptoValue>(&bytes).unwrap();

        let serializable = scrypto_value.simple_serializable(context);

        assert_json_eq(serializable, expected);
    }

    fn assert_invertible_json_matches<'a, T: ScryptoEncode, C: Into<ValueFormattingContext<'a>>>(
        value: &T,
        context: C,
        expected: Value,
    ) {
        let bytes = scrypto_encode(&value).unwrap();
        let scrypto_value = scrypto_decode::<ScryptoValue>(&bytes).unwrap();

        let serializable = scrypto_value.invertible_serializable(context);

        assert_json_eq(serializable, expected);
    }
}
