use crate::address::*;
use crate::constants::*;
use crate::crypto::*;
use crate::model::*;
use radix_engine_derive::*;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_abi::LegacyDescribe;
use scrypto_abi::Type;
use utils::ContextualDisplay;

#[derive(
    Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub struct NonFungibleGlobalId(ResourceAddress, NonFungibleLocalId);

impl LegacyDescribe for NonFungibleGlobalId {
    fn describe() -> scrypto_abi::Type {
        Type::NonFungibleGlobalId
    }
}

impl NonFungibleGlobalId {
    pub const fn new(
        resource_address: ResourceAddress,
        non_fungible_local_id: NonFungibleLocalId,
    ) -> Self {
        Self(resource_address, non_fungible_local_id)
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.0
    }

    /// Returns the non-fungible id.
    pub fn non_fungible_local_id(&self) -> &NonFungibleLocalId {
        &self.1
    }

    /// Returns canonical representation of this NonFungibleGlobalId.
    pub fn to_canonical_string(&self, bech32_encoder: &Bech32Encoder) -> String {
        format!(
            "{}:{}",
            bech32_encoder.encode_resource_address_to_string(&self.resource_address()),
            self.non_fungible_local_id().to_simple_string()
        )
    }

    /// Converts canonical representation to NonFungibleGlobalId.
    ///
    /// This is composed of `resource_address:id_simple_representation`
    pub fn try_from_canonical_string(
        bech32_decoder: &Bech32Decoder,
        id_type: NonFungibleIdType,
        s: &str,
    ) -> Result<Self, ParseNonFungibleGlobalIdError> {
        let v = s
            .splitn(2, ':')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>();
        if v.len() != 2 {
            return Err(ParseNonFungibleGlobalIdError::RequiresTwoParts);
        }
        let resource_address = bech32_decoder.validate_and_decode_resource_address(v[0])?;
        let non_fungible_local_id = NonFungibleLocalId::try_from_simple_string(id_type, v[1])?;
        Ok(NonFungibleGlobalId::new(
            resource_address,
            non_fungible_local_id,
        ))
    }

    /// Returns canonical representation of this NonFungibleGlobalId.
    pub fn to_canonical_combined_string(&self, bech32_encoder: &Bech32Encoder) -> String {
        format!(
            "{}:{}",
            bech32_encoder.encode_resource_address_to_string(&self.resource_address()),
            self.non_fungible_local_id().to_combined_simple_string()
        )
    }

    /// Converts combined canonical representation to NonFungibleGlobalId.
    ///
    /// This is composed of `resource_address:IdType#id_simple_representation`
    ///
    /// Prefer the canonical string where the id type can be looked up.
    pub fn try_from_canonical_combined_string(
        bech32_decoder: &Bech32Decoder,
        s: &str,
    ) -> Result<Self, ParseNonFungibleGlobalIdError> {
        let v = s
            .splitn(2, ':')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>();
        if v.len() != 2 {
            return Err(ParseNonFungibleGlobalIdError::RequiresTwoParts);
        }
        let resource_address = bech32_decoder.validate_and_decode_resource_address(v[0])?;
        let non_fungible_local_id = NonFungibleLocalId::try_from_combined_simple_string(v[1])?;
        Ok(NonFungibleGlobalId::new(
            resource_address,
            non_fungible_local_id,
        ))
    }
}

//======
// error
//======

/// Represents an error when parsing non-fungible address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleGlobalIdError {
    InvalidResourceAddress(AddressError),
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
    RequiresTwoParts,
}

impl From<AddressError> for ParseNonFungibleGlobalIdError {
    fn from(err: AddressError) -> Self {
        Self::InvalidResourceAddress(err)
    }
}

impl From<ParseNonFungibleLocalIdError> for ParseNonFungibleGlobalIdError {
    fn from(err: ParseNonFungibleLocalIdError) -> Self {
        Self::InvalidNonFungibleLocalId(err)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleGlobalIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleGlobalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl fmt::Debug for NonFungibleGlobalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

pub trait FromPublicKey: Sized {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self;
}

impl FromPublicKey for NonFungibleGlobalId {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self {
        let public_key: PublicKey = public_key.clone().into();
        match public_key {
            PublicKey::EcdsaSecp256k1(public_key) => NonFungibleGlobalId::new(
                ECDSA_SECP256K1_TOKEN,
                NonFungibleLocalId::Bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
            PublicKey::EddsaEd25519(public_key) => NonFungibleGlobalId::new(
                EDDSA_ED25519_TOKEN,
                NonFungibleLocalId::Bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
        }
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for NonFungibleGlobalId {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}:{}",
            self.resource_address().display(*context),
            self.non_fungible_local_id()
        )
    }
}

//======
// test
//======

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Bech32Decoder;

    #[test]
    fn non_fungible_global_id_canonical_conversion() {
        let dec = Bech32Decoder::for_simulator();
        let enc = Bech32Encoder::for_simulator();

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                NonFungibleIdType::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                NonFungibleIdType::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:10",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:10"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                NonFungibleIdType::UUID,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1234567890",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1234567890"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                NonFungibleIdType::String,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:test",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:test"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                NonFungibleIdType::Bytes,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:010a",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:010a"
        );
    }

    #[test]
    fn non_fungible_global_id_canonical_conversion_error() {
        let bech32_decoder = Bech32Decoder::for_simulator();
        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdType::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p",
            ),
            Err(ParseNonFungibleGlobalIdError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdType::String,
                // : is not currently allowed in non-fungible ids
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1:2",
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidNonFungibleLocalId(
                ParseNonFungibleLocalIdError::InvalidCharacter(':')
            ))
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdType::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:",
            ),
            Err(ParseNonFungibleGlobalIdError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdType::Number,
                ":",
            ),
            Err(ParseNonFungibleGlobalIdError::RequiresTwoParts)
        );

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdType::Number,
                "3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1",
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidResourceAddress(_))
        ));

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdType::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:notnumber",
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidNonFungibleLocalId(
                ParseNonFungibleLocalIdError::InvalidInt(_)
            ))
        ));
    }
}
