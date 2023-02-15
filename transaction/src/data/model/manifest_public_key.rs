use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::crypto::EddsaEd25519PublicKey;
use radix_engine_interface::crypto::PublicKey;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::*;
use utils::copy_u8_array;

use crate::data::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestPublicKey(pub PublicKey);

//========
// error
//========

/// Represents an error when parsing ManifestPublicKey.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestPublicKeyError {
    InvalidLength,
    UnknownTypeOfPublicKey,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestPublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestPublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl Categorize<ManifestCustomValueKind> for ManifestPublicKey {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::PublicKey)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for ManifestPublicKey {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self.0 {
            PublicKey::EcdsaSecp256k1(pk) => {
                encoder.write_discriminator(0)?;
                encoder.write_slice(&pk.0)
            }
            PublicKey::EddsaEd25519(pk) => {
                encoder.write_discriminator(1)?;
                encoder.write_slice(&pk.0)
            }
        }
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for ManifestPublicKey {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            0 => {
                let bytes = decoder.read_slice(EcdsaSecp256k1PublicKey::LENGTH)?;
                Ok(Self(PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PublicKey(
                    copy_u8_array(bytes),
                ))))
            }
            1 => {
                let bytes = decoder.read_slice(EddsaEd25519PublicKey::LENGTH)?;
                Ok(Self(PublicKey::EddsaEd25519(EddsaEd25519PublicKey(
                    copy_u8_array(bytes),
                ))))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}
