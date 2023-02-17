use crate::crypto::*;
use crate::data::*;
use crate::*;
use sbor::*;
use scrypto_abi::Type;
use transaction_data::*;
use utils::copy_u8_array;

/// Represents any natively supported public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "public_key")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PublicKey {
    EcdsaSecp256k1(EcdsaSecp256k1PublicKey),
    EddsaEd25519(EddsaEd25519PublicKey),
}

impl PublicKey {
    pub fn encode_body_common<X: CustomValueKind, E: Encoder<X>>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        match self {
            Self::EcdsaSecp256k1(v) => {
                encoder.write_discriminator(0)?;
                encoder.write_slice(&v.0)?;
            }
            Self::EddsaEd25519(v) => {
                encoder.write_discriminator(1)?;
                encoder.write_slice(&v.0)?;
            }
        }
        Ok(())
    }

    pub fn decode_body_common<X: CustomValueKind, D: Decoder<X>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match decoder.read_discriminator()? {
            0 => Ok(Self::EcdsaSecp256k1(EcdsaSecp256k1PublicKey(
                copy_u8_array(decoder.read_slice(EcdsaSecp256k1PublicKey::LENGTH)?),
            ))),
            1 => Ok(Self::EddsaEd25519(EddsaEd25519PublicKey(copy_u8_array(
                decoder.read_slice(EcdsaSecp256k1PublicKey::LENGTH)?,
            )))),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl From<EcdsaSecp256k1PublicKey> for PublicKey {
    fn from(public_key: EcdsaSecp256k1PublicKey) -> Self {
        Self::EcdsaSecp256k1(public_key)
    }
}

impl From<EddsaEd25519PublicKey> for PublicKey {
    fn from(public_key: EddsaEd25519PublicKey) -> Self {
        Self::EddsaEd25519(public_key)
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for PublicKey {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::PublicKey)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for PublicKey {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for PublicKey {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

impl scrypto_abi::LegacyDescribe for PublicKey {
    fn describe() -> scrypto_abi::Type {
        Type::PublicKey
    }
}

//===================
// binary (manifest)
//===================

impl Categorize<ManifestCustomValueKind> for PublicKey {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::PublicKey)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for PublicKey {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for PublicKey {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

//========
// text
//========
impl ToString for PublicKey {
    fn to_string(&self) -> String {
        match self {
            PublicKey::EcdsaSecp256k1(pk) => hex::encode(&pk.0),
            PublicKey::EddsaEd25519(pk) => hex::encode(&pk.0),
        }
    }
}
