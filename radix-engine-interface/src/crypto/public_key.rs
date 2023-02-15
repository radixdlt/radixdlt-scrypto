use crate::data::*;
use crate::*;
use sbor::*;
use scrypto_abi::Type;
use utils::copy_u8_array;

/// Represents an ECDSA public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EcdsaSecp256k1PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl EcdsaSecp256k1PublicKey {
    pub const LENGTH: usize = 33;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Categorize<ScryptoCustomValueKind> for EcdsaSecp256k1PublicKey {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for EcdsaSecp256k1PublicKey
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        PublicKey::EcdsaSecp256k1(self.clone()).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for EcdsaSecp256k1PublicKey
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = PublicKey::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            PublicKey::EcdsaSecp256k1(pk) => Ok(pk),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for EcdsaSecp256k1PublicKey {
    fn describe() -> scrypto_abi::Type {
        Type::EcdsaSecp256k1PublicKey
    }
}

/// Represents an ED25519 public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EddsaEd25519PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl EddsaEd25519PublicKey {
    pub const LENGTH: usize = 32;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Categorize<ScryptoCustomValueKind> for EddsaEd25519PublicKey {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for EddsaEd25519PublicKey
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        PublicKey::EddsaEd25519(self.clone()).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for EddsaEd25519PublicKey
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = PublicKey::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            PublicKey::EddsaEd25519(pk) => Ok(pk),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for EddsaEd25519PublicKey {
    fn describe() -> scrypto_abi::Type {
        Type::EddsaEd25519PublicKey
    }
}

/// Represents any natively supported public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "public_key")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
