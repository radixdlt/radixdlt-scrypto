use crate::abi::*;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::ScryptoCustomValueKind;
use crate::*;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    Package(PackageAddress),
    Component(ComponentAddress),
    ResourceManager(ResourceAddress),
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Reference {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Reference)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Reference {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Reference::Package(v) => {
                encoder.write_byte(0)?;
                encoder.write_slice(&v.to_vec())?;
            }
            Reference::Component(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_vec())?;
            }
            Reference::ResourceManager(v) => {
                encoder.write_byte(2)?;
                encoder.write_slice(&v.to_vec())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Reference {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_byte()? {
            0 => Ok(Self::Package(
                PackageAddress::try_from(decoder.read_slice(27)?)
                    .map_err(|_| DecodeError::InvalidCustomValue)?,
            )),
            1 => Ok(Self::Component(
                ComponentAddress::try_from(decoder.read_slice(27)?)
                    .map_err(|_| DecodeError::InvalidCustomValue)?,
            )),
            2 => Ok(Self::ResourceManager(
                ResourceAddress::try_from(decoder.read_slice(27)?)
                    .map_err(|_| DecodeError::InvalidCustomValue)?,
            )),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for Reference {
    fn describe() -> scrypto_abi::Type {
        Type::Reference
    }
}
