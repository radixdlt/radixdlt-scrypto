use crate::address::*;
use crate::constants::*;
use crate::crypto::*;
use crate::data::scrypto::model::*;
use crate::types::BlueprintId;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::data::scrypto::scrypto_encode;
use radix_engine_common::types::*;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use utils::ContextualDisplay;

/// Represents the global id of a non-fungible.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleGlobalId(ResourceAddress, NonFungibleLocalId);

impl NonFungibleGlobalId {
    pub const fn new(resource_address: ResourceAddress, local_id: NonFungibleLocalId) -> Self {
        Self(resource_address, local_id)
    }

    pub fn package_of_direct_caller_badge(address: PackageAddress) -> Self {
        // TODO: Is there a better way of ensuring that number of bytes is less than 64 over hashing?
        let hashed = hash(scrypto_encode(&address).unwrap()).to_vec();
        let local_id = NonFungibleLocalId::bytes(hashed).unwrap();
        NonFungibleGlobalId::new(PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE, local_id)
    }

    pub fn global_caller_badge<T: Into<GlobalCaller>>(global_caller: T) -> Self {
        // TODO: Is there a better way of ensuring that number of bytes is less than 64 over hashing?
        let hashed = hash(scrypto_encode(&global_caller.into()).unwrap()).to_vec();
        let local_id = NonFungibleLocalId::bytes(hashed).unwrap();
        NonFungibleGlobalId::new(GLOBAL_CALLER_VIRTUAL_BADGE, local_id)
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.0
    }

    /// Returns the non-fungible id.
    pub fn local_id(&self) -> &NonFungibleLocalId {
        &self.1
    }

    /// Returns canonical representation of this NonFungibleGlobalId.
    pub fn to_canonical_string(&self, address_bech32_encoder: &AddressBech32Encoder) -> String {
        format!("{}", self.display(address_bech32_encoder))
    }

    /// Converts canonical representation to NonFungibleGlobalId.
    ///
    /// This is composed of `resource_address:id_simple_representation`
    pub fn try_from_canonical_string(
        address_bech32_decoder: &AddressBech32Decoder,
        s: &str,
    ) -> Result<Self, ParseNonFungibleGlobalIdError> {
        let parts = s.split(':').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(ParseNonFungibleGlobalIdError::RequiresTwoParts);
        }
        let resource_address = ResourceAddress::try_from_bech32(address_bech32_decoder, parts[0])
            .ok_or(ParseNonFungibleGlobalIdError::InvalidResourceAddress)?;
        let local_id = NonFungibleLocalId::from_str(parts[1])?;
        Ok(NonFungibleGlobalId::new(resource_address, local_id))
    }
}

#[derive(Clone, Debug, ScryptoSbor)]
pub enum GlobalCaller {
    /// If the previous global frame started with an object's main module
    GlobalObject(GlobalAddress),
    /// If the previous global frame started with a function call
    PackageBlueprint(BlueprintId),
}

impl<T> From<T> for GlobalCaller
where
    T: Into<GlobalAddress>,
{
    fn from(value: T) -> Self {
        GlobalCaller::GlobalObject(value.into())
    }
}

impl From<BlueprintId> for GlobalCaller {
    fn from(blueprint: BlueprintId) -> Self {
        GlobalCaller::PackageBlueprint(blueprint)
    }
}

//======
// error
//======

/// Represents an error when parsing non-fungible address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleGlobalIdError {
    InvalidResourceAddress,
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
    RequiresTwoParts,
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
            self.local_id()
        )
    }
}

impl fmt::Debug for NonFungibleGlobalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

pub trait FromPublicKey: Sized {
    fn from_public_key<P: HasPublicKeyHash>(public_key: &P) -> Self;
    fn from_public_key_hash<P: IsPublicKeyHash>(public_key_hash: P) -> Self;
}

impl FromPublicKey for NonFungibleGlobalId {
    fn from_public_key<P: HasPublicKeyHash>(public_key: &P) -> Self {
        Self::from_public_key_hash(public_key.get_hash())
    }

    fn from_public_key_hash<P: IsPublicKeyHash>(public_key_hash: P) -> Self {
        match public_key_hash.into_enum() {
            PublicKeyHash::Secp256k1(public_key_hash) => NonFungibleGlobalId::new(
                SECP256K1_SIGNATURE_VIRTUAL_BADGE,
                NonFungibleLocalId::bytes(public_key_hash.get_hash_bytes().to_vec()).unwrap(),
            ),
            PublicKeyHash::Ed25519(public_key_hash) => NonFungibleGlobalId::new(
                ED25519_SIGNATURE_VIRTUAL_BADGE,
                NonFungibleLocalId::bytes(public_key_hash.get_hash_bytes().to_vec()).unwrap(),
            ),
        }
    }
}

//======
// test
//======

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::test_addresses::*;
    use crate::address::AddressBech32Decoder;

    #[test]
    fn non_fungible_global_id_canonical_conversion() {
        let dec = AddressBech32Decoder::for_simulator();
        let enc = AddressBech32Encoder::for_simulator();

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                &format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:<id>"),
            )
            .unwrap()
            .to_canonical_string(&enc),
            format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:<id>")
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                &format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:#123#"),
            )
            .unwrap()
            .to_canonical_string(&enc),
            format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:#123#")
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                &format!(
                    "{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:{{1111111111111111-2222222222222222-3333333333333333-4444444444444444}}"
                ),
            )
            .unwrap()
            .to_canonical_string(&enc),
            format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:{{1111111111111111-2222222222222222-3333333333333333-4444444444444444}}")
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                &format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:<test>"),
            )
            .unwrap()
            .to_canonical_string(&enc),
            format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:<test>"),
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                &format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:[010a]"),
            )
            .unwrap()
            .to_canonical_string(&enc),
            format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:[010a]"),
        );
    }

    #[test]
    fn non_fungible_global_id_canonical_conversion_error() {
        let address_bech32_decoder = AddressBech32Decoder::for_simulator();
        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &address_bech32_decoder,
                &NON_FUNGIBLE_RESOURCE_SIM_ADDRESS,
            ),
            Err(ParseNonFungibleGlobalIdError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &address_bech32_decoder,
                &format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:1:2"),
            ),
            Err(ParseNonFungibleGlobalIdError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &address_bech32_decoder,
                &format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:"),
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidNonFungibleLocalId(
                ParseNonFungibleLocalIdError::UnknownType
            ))
        );

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(&address_bech32_decoder, ":",),
            Err(ParseNonFungibleGlobalIdError::InvalidResourceAddress)
        ));

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(
                &address_bech32_decoder,
                "3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:#1#",
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidResourceAddress)
        ));

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(
                &address_bech32_decoder,
                &format!("{NON_FUNGIBLE_RESOURCE_SIM_ADDRESS}:#notnumber#"),
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidNonFungibleLocalId(
                ParseNonFungibleLocalIdError::InvalidInteger
            ))
        ));
    }
}
