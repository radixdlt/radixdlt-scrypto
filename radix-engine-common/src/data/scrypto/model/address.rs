use crate::address::{AddressDisplayContext, AddressError, NO_NETWORK};
use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::model::*;
use crate::data::scrypto::*;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::ContextualDisplay;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Address {
    Package(PackageAddress),
    Component(ComponentAddress),
    Resource(ResourceAddress),
}

impl Address {
    pub fn encode_body_common<X: CustomValueKind, E: Encoder<X>>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        encoder.write_slice(&self.to_vec())
    }

    pub fn decode_body_common<X: CustomValueKind, D: Decoder<X>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let slice = decoder.read_slice(27)?;
        PackageAddress::try_from(slice)
            .map(|x| Address::Package(x))
            .or(ComponentAddress::try_from(slice).map(|x| Address::Component(x)))
            .or(ResourceAddress::try_from(slice).map(|x| Address::Resource(x)))
            .map_err(|_| DecodeError::InvalidCustomValue)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Address::Package(inner) => inner.to_vec(),
            Address::Component(inner) => inner.to_vec(),
            Address::Resource(inner) => inner.to_vec(),
        }
    }
}

impl From<ComponentAddress> for Address {
    fn from(value: ComponentAddress) -> Self {
        Address::Component(value)
    }
}

impl From<ResourceAddress> for Address {
    fn from(value: ResourceAddress) -> Self {
        Address::Resource(value)
    }
}

impl From<PackageAddress> for Address {
    fn from(value: PackageAddress) -> Self {
        Address::Package(value)
    }
}

// TODO: replace with TryInto

impl Into<ComponentAddress> for Address {
    fn into(self) -> ComponentAddress {
        match self {
            Address::Component(component_address) => component_address,
            _ => panic!("Not a component address"),
        }
    }
}

impl Into<PackageAddress> for Address {
    fn into(self) -> PackageAddress {
        match self {
            Address::Package(package_address) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for Address {
    fn into(self) -> ResourceAddress {
        match self {
            Address::Resource(resource_address) => resource_address,
            _ => panic!("Not a resource address"),
        }
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Address {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Address)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Address {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Address {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

impl Describe<ScryptoCustomTypeKind<GlobalTypeId>> for Address {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(crate::data::scrypto::well_known_scrypto_custom_types::ADDRESS_ID);
}

//==================
// binary (manifest)
//==================

impl Categorize<ManifestCustomValueKind> for Address {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for Address {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for Address {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

//======
// text
//======

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Address::Package(a) => write!(f, "{}", a.display(NO_NETWORK)),
            Address::Component(a) => write!(f, "{}", a.display(NO_NETWORK)),
            Address::Resource(a) => write!(f, "{}", a.display(NO_NETWORK)),
        }
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for Address {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        match self {
            Address::Package(a) => a.contextual_format(f, context),
            Address::Component(a) => a.contextual_format(f, context),
            Address::Resource(a) => a.contextual_format(f, context),
        }
    }
}
