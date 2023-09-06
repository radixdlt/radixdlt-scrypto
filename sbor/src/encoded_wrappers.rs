use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

/// A wrapper for a full encoded SBOR payload, including the prefix byte.
///
/// Encode is implemented, but Decode is not - because the payload needs to be checked to be valid.
/// Instead, you can use the typed or untyped traverser to validate the payload.
///
/// The payload is assumed to be valid, and for performance, the payload is
/// encoded directly without checking if it is valid.
///
/// If you need to check the validity of a payload first:
/// * If you have a schema - use the typed traverser
/// * If it is schemaless - use the untyped traverser
pub struct RawPayload<'a, E: CustomExtension> {
    full_payload: Cow<'a, [u8]>,
    root_value_kind: ValueKind<E::CustomValueKind>,
    custom_extension: PhantomData<E>,
}

impl<'a, E: CustomExtension> RawPayload<'a, E> {
    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller.
    ///
    /// This constructor does not check the prefix byte, and panics if the root value kind is invalid.
    pub fn new_from_valid_slice(payload_bytes: &'a [u8]) -> Self {
        Self {
            full_payload: Cow::Borrowed(payload_bytes),
            root_value_kind: ValueKind::<E::CustomValueKind>::from_u8(payload_bytes[1]).unwrap(),
            custom_extension: PhantomData,
        }
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller.
    ///
    /// Unlike `new_from_valid_payload_bytes`, the constructor includes a prefix byte check to hopefully
    /// catch some errors - but it is not guaranteed to catch all errors.
    pub fn new_from_valid_slice_with_checks(payload_bytes: &'a [u8]) -> Option<Self> {
        if payload_bytes.len() < 2 || payload_bytes[0] != E::PAYLOAD_PREFIX {
            return None;
        }
        let Some(value_kind) = ValueKind::<E::CustomValueKind>::from_u8(payload_bytes[1]) else {
            return None;
        };
        Some(Self {
            full_payload: Cow::Borrowed(payload_bytes),
            root_value_kind: value_kind,
            custom_extension: PhantomData,
        })
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller.
    ///
    /// This constructor does not check the prefix byte, and panics if the root value kind is invalid.
    pub fn new_from_valid_owned(payload_bytes: Vec<u8>) -> Self {
        let root_value_kind = ValueKind::<E::CustomValueKind>::from_u8(payload_bytes[1]).unwrap();
        Self {
            full_payload: Cow::Owned(payload_bytes),
            root_value_kind,
            custom_extension: PhantomData,
        }
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller.
    ///
    /// Unlike `new_from_valid_payload_bytes`, the constructor includes a prefix byte check to hopefully
    /// catch some errors - but it is not guaranteed to catch all errors.
    pub fn new_from_valid_owned_with_checks(payload_bytes: Vec<u8>) -> Option<Self> {
        if payload_bytes.len() < 2 || payload_bytes[0] != E::PAYLOAD_PREFIX {
            return None;
        }
        let Some(value_kind) = ValueKind::<E::CustomValueKind>::from_u8(payload_bytes[1]) else {
            return None;
        };
        Some(Self {
            full_payload: Cow::Owned(payload_bytes),
            root_value_kind: value_kind,
            custom_extension: PhantomData,
        })
    }

    pub fn as_encoded_value<'b>(&'b self) -> RawValue<'b, E> {
        RawValue::new_from_valid_value_body_slice(
            self.root_value_kind,
            self.encoded_root_body_bytes(),
        )
    }

    pub fn decode_into<
        T: for<'b> Decode<E::CustomValueKind, VecDecoder<'b, E::CustomValueKind>>,
    >(
        &self,
        depth_limit: usize,
    ) -> Result<T, DecodeError> {
        let mut decoder = VecDecoder::new(self.encoded_root_body_bytes(), depth_limit);
        T::decode_body_with_value_kind(&mut decoder, self.root_value_kind)
    }

    pub fn root_value_kind(&self) -> ValueKind<E::CustomValueKind> {
        self.root_value_kind
    }

    pub fn payload_bytes(&self) -> &[u8] {
        &self.full_payload
    }

    pub fn encoded_root_value_bytes(&self) -> &[u8] {
        &self.full_payload[1..]
    }

    pub fn encoded_root_body_bytes(&self) -> &[u8] {
        &self.full_payload[2..]
    }
}

impl<'a, E: CustomExtension> TryFrom<&'a [u8]> for RawPayload<'a, E> {
    type Error = ();

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        RawPayload::new_from_valid_slice_with_checks(value).ok_or(())
    }
}

impl<Ext: CustomExtension, E: Encoder<Ext::CustomValueKind>> Encode<Ext::CustomValueKind, E>
    for RawPayload<'_, Ext>
{
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(self.root_value_kind)
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_slice(self.encoded_root_body_bytes())
    }
}

/// A wrapper for a reference to a valid partial SBOR payload representing a single value.
///
/// Encode, Decode and Describe are implemented:
/// * For performance, the payload is encoded directly without checking if it is valid.
/// * Decoding goes through a traverser, which calculates the length of the bytes and ensures that the bytes are actually valid.
/// * Describe is implemented as Any.
///
/// Categorize can't be implemented, because we can't guarantee the value kind is constant.
/// This means RawValue can't be put as an immediate child to a Vec or Map.
#[derive(Debug, Clone)]
pub struct RawValue<'a, E: CustomExtension> {
    value_kind: ValueKind<E::CustomValueKind>,
    value_body: Cow<'a, [u8]>,
    custom_extension: PhantomData<E>,
}

impl<'a, E: CustomExtension> RawValue<'a, E> {
    /// The bytes should include the value kind byte, but not the payload prefix byte.
    ///
    /// The bytes must be at least 1 byte long, else this will panic.
    pub fn new_from_valid_full_value_slice(encoded_full_value: &'a [u8]) -> Self {
        let value_kind = ValueKind::from_u8(encoded_full_value[0]).unwrap();
        Self {
            value_kind,
            value_body: Cow::Borrowed(&encoded_full_value[1..]),
            custom_extension: PhantomData,
        }
    }

    /// The bytes should include the value, not the value kind or the prefix byte
    pub fn new_from_valid_value_body_slice(
        value_kind: ValueKind<E::CustomValueKind>,
        encoded_value_body: &'a [u8],
    ) -> Self {
        Self {
            value_kind,
            value_body: Cow::Borrowed(encoded_value_body),
            custom_extension: PhantomData,
        }
    }

    /// The bytes should include the value, not the value kind or the prefix byte
    pub fn new_from_valid_owned_value_body(
        value_kind: ValueKind<E::CustomValueKind>,
        encoded_value_body: Vec<u8>,
    ) -> Self {
        Self {
            value_kind,
            value_body: Cow::Owned(encoded_value_body),
            custom_extension: PhantomData,
        }
    }

    pub fn value_kind(&self) -> ValueKind<E::CustomValueKind> {
        self.value_kind
    }

    pub fn value_body_bytes(&self) -> &[u8] {
        &self.value_body
    }
}

impl<Ext: CustomExtension, E: Encoder<Ext::CustomValueKind>> Encode<Ext::CustomValueKind, E>
    for RawValue<'_, Ext>
{
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(self.value_kind)
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_slice(self.value_body.as_ref())
    }
}

impl<Ext: CustomExtension, D: Decoder<Ext::CustomValueKind>> Decode<Ext::CustomValueKind, D>
    for RawValue<'_, Ext>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<Ext::CustomValueKind>,
    ) -> Result<Self, DecodeError> {
        // Because SBOR isn't a length-first decoding, you don't know how long a value "tree" is until you've decoded it.
        // So we use a traverser to calculate the length of the subpayload, and then read that many bytes.
        let length = calculate_value_tree_body_byte_length::<Ext>(
            decoder.peek_remaining(),
            value_kind,
            decoder.get_stack_depth(),
            decoder.get_depth_limit(),
        )?;
        // Because Decode doesn't (currently) allow borrowing from the decoder, we have to to_vec here
        Ok(Self::new_from_valid_owned_value_body(
            value_kind,
            decoder.read_slice(length)?.to_vec(),
        ))
    }
}

impl<Ext: CustomExtension, C: CustomTypeKind<RustTypeId>> Describe<C> for RawValue<'_, Ext> {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(basic_well_known_types::ANY_TYPE);

    fn type_data() -> TypeData<C, RustTypeId> {
        basic_well_known_types::any_type_data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(BasicSbor)]
    struct RawValueStruct {
        field1: BasicOwnedRawValue,
        field2: (BasicOwnedRawValue, BasicOwnedRawValue),
    }

    #[test]
    pub fn can_encode_and_decode_raw_value() {
        let encoded = basic_encode(&BasicValue::Tuple {
            fields: vec![
                // Field1
                BasicValue::Enum {
                    discriminator: 1,
                    fields: vec![],
                },
                // Field2
                BasicValue::Tuple {
                    fields: vec![BasicValue::U8 { value: 1 }, BasicValue::U16 { value: 5125 }],
                },
            ],
        })
        .unwrap();
        let decoded: RawValueStruct = basic_decode(&encoded).unwrap();
        // Check that the content of the raw value makes sense
        assert_eq!(decoded.field2.1.value_kind, ValueKind::U16);
        assert_eq!(
            decoded.field2.1.value_body_bytes(),
            // Extract the value body (ie remove the payload prefix byte and the value kind byte)
            &basic_encode(&5125u16).unwrap()[2..],
        );
        // Check that it can be encoded back to the original value
        let encoded2 = basic_encode(&decoded).unwrap();
        assert_eq!(encoded, encoded2);
    }
}
