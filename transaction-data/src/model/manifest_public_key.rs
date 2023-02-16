use crate::*;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::*;
use utils::copy_u8_array;

pub const ECDSA_SECP256K1_PUBLIC_KEY_LENGTH: usize = 33;
pub const EDDSA_ED25519_PUBLIC_KEY_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestPublicKey {
    EcdsaSecp256k1([u8; ECDSA_SECP256K1_PUBLIC_KEY_LENGTH]),
    EddsaEd25519([u8; EDDSA_ED25519_PUBLIC_KEY_LENGTH]),
}

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
        match self {
            Self::EcdsaSecp256k1(pk) => {
                encoder.write_discriminator(0)?;
                encoder.write_slice(pk)
            }
            Self::EddsaEd25519(pk) => {
                encoder.write_discriminator(1)?;
                encoder.write_slice(pk)
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
                let bytes = decoder.read_slice(ECDSA_SECP256K1_PUBLIC_KEY_LENGTH)?;
                Ok(Self::EcdsaSecp256k1(copy_u8_array(bytes)))
            }
            1 => {
                let bytes = decoder.read_slice(EDDSA_ED25519_PUBLIC_KEY_LENGTH)?;
                Ok(Self::EddsaEd25519(copy_u8_array(bytes)))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}
