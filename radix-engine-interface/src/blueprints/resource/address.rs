use crate::address::*;
use crate::api::types::*;
use crate::data::model::Address;
use crate::data::ScryptoCustomValueKind;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_abi::*;
use transaction_data::*;
use utils::{copy_u8_array, ContextualDisplay};

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceAddress {
    Normal([u8; 26]),
}

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::Resource => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceAddress {
    pub fn raw(&self) -> [u8; 26] {
        match self {
            Self::Normal(v) => v.clone(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::resource(self).id());
        match self {
            Self::Normal(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for ResourceAddress {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Address)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for ResourceAddress {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Address::Resource(self.clone()).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for ResourceAddress {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let a = Address::decode_body_with_value_kind(decoder, value_kind)?;
        match a {
            Address::Resource(x) => Ok(x),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for ResourceAddress {
    fn describe() -> scrypto_abi::Type {
        Type::ResourceAddress
    }
}

//===================
// binary (manifest)
//===================

impl Categorize<ManifestCustomValueKind> for ResourceAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for ResourceAddress {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Address::Resource(self.clone()).encode_body(encoder)
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for ResourceAddress {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let a = Address::decode_body_with_value_kind(decoder, value_kind)?;
        match a {
            Address::Resource(x) => Ok(x),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

//======
// text
//======

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ResourceAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_resource_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ResourceAddress::Normal(_) => {
                write!(f, "NormalResource[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
