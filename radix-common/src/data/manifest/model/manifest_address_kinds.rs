use crate::internal_prelude::*;
use radix_common::scrypto_describe_for_manifest_type;
use sbor::{Decoder, Encoder};

/*
=================================================================================
NOTE: For now, we only support "dynamic" addresses for CALL instructions.
=================================================================================
This is to reduce the scope of change and make manifest easier to reason about.

In theory, we can apply it to all types of global addresses (`GlobalAddress`,
`PackageAddress`, `ResourceAddress` and `ComponentAddress`).

Then, we can do more advanced stuff in manifest, such as
```
ALLOCATE_GLOBAL_ADDRESS
    Address("{resource_package}")
    "FungibleResourceManager"
    AddressReservation("address_reservation")
    NamedAddress("new_resource")
;
CALL_FUNCTION
    Address("{resource_package}")
    "FungibleResourceManager"
    "create_with_initial_supply_and_address"
    Decimal("10")
    AddressReservation("address_reservation")
;
TAKE_FROM_WORKTOP
    NamedAddress("new_resource")
    Decimal("5.0")
    Bucket("bucket1")
;
TAKE_FROM_WORKTOP
    NamedAddress("new_resource")
    Decimal("5.0")
    Bucket("bucket2")
;
```
*/

macro_rules! labelled_resolvable_address {
    ($ty:ty$(,)?) => {
        resolvable_with_try_into_impls!($ty);
        labelled_resolvable_using_resolvable_impl!($ty, resolver_output: ManifestNamedAddress);

        impl<'a> LabelledResolveFrom<&'a str> for $ty {
            fn labelled_resolve_from(value: &'a str, resolver: &impl LabelResolver<ManifestNamedAddress>) -> Self {
                resolver.resolve_label_into(value).into()
            }
        }

        impl<'a> LabelledResolveFrom<&'a String> for $ty {
            fn labelled_resolve_from(value: &'a String, resolver: &impl LabelResolver<ManifestNamedAddress>) -> Self {
                resolver.resolve_label_into(value.as_str()).into()
            }
        }

        impl<'a> LabelledResolveFrom<String> for $ty {
            fn labelled_resolve_from(value: String, resolver: &impl LabelResolver<ManifestNamedAddress>) -> Self {
                resolver.resolve_label_into(value.as_str()).into()
            }
        }
    };
}

// Alias for backwards-compatibility
pub type DynamicGlobalAddress = ManifestGlobalAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManifestGlobalAddress {
    Static(GlobalAddress),
    Named(ManifestNamedAddress),
}

scrypto_describe_for_manifest_type!(
    ManifestGlobalAddress,
    GLOBAL_ADDRESS_TYPE,
    global_address_type_data,
);

labelled_resolvable_address!(ManifestGlobalAddress);

impl Categorize<ManifestCustomValueKind> for ManifestGlobalAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for ManifestGlobalAddress
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::Static(address) => {
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_STATIC)?;
                encoder.write_slice(address.as_node_id().as_bytes())?;
            }
            Self::Named(address_id) => {
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_NAMED)?;
                encoder.write_slice(&address_id.0.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for ManifestGlobalAddress
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            MANIFEST_ADDRESS_DISCRIMINATOR_STATIC => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                Ok(Self::Static(
                    GlobalAddress::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            MANIFEST_ADDRESS_DISCRIMINATOR_NAMED => {
                let slice = decoder.read_slice(4)?;
                let id = u32::from_le_bytes(slice.try_into().unwrap());
                Ok(Self::Named(ManifestNamedAddress(id)))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl ManifestGlobalAddress {
    /// This is to support either `Address("static_address")` or `NamedAddress("abc")` in manifest instruction,
    /// instead of `Enum<0u8>(Address("static_address"))`.
    pub fn to_instruction_argument(&self) -> ManifestValue {
        match self {
            Self::Static(address) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Static(
                    address.into_node_id(),
                )),
            },
            Self::Named(id) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Named(*id)),
            },
        }
    }

    pub fn is_static_global_package(&self) -> bool {
        match self {
            Self::Static(address) => address.as_node_id().is_global_package(),
            Self::Named(_) => false,
        }
    }

    pub fn is_static_global_fungible_resource_manager(&self) -> bool {
        match self {
            Self::Static(address) => address.as_node_id().is_global_fungible_resource_manager(),
            Self::Named(_) => false,
        }
    }
    pub fn is_static_global_non_fungible_resource_manager(&self) -> bool {
        match self {
            Self::Static(address) => address
                .as_node_id()
                .is_global_non_fungible_resource_manager(),
            Self::Named(_) => false,
        }
    }
}

impl From<GlobalAddress> for ManifestGlobalAddress {
    fn from(value: GlobalAddress) -> Self {
        Self::Static(value)
    }
}

impl From<PackageAddress> for ManifestGlobalAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<ManifestPackageAddress> for ManifestGlobalAddress {
    fn from(value: ManifestPackageAddress) -> Self {
        match value {
            ManifestPackageAddress::Static(value) => Self::Static(value.into()),
            ManifestPackageAddress::Named(value) => Self::Named(value),
        }
    }
}

impl From<ResourceAddress> for ManifestGlobalAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<ManifestResourceAddress> for ManifestGlobalAddress {
    fn from(value: ManifestResourceAddress) -> Self {
        match value {
            ManifestResourceAddress::Static(value) => Self::Static(value.into()),
            ManifestResourceAddress::Named(value) => Self::Named(value),
        }
    }
}

impl From<ComponentAddress> for ManifestGlobalAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<ManifestComponentAddress> for ManifestGlobalAddress {
    fn from(value: ManifestComponentAddress) -> Self {
        match value {
            ManifestComponentAddress::Static(value) => Self::Static(value.into()),
            ManifestComponentAddress::Named(value) => Self::Named(value),
        }
    }
}

impl From<ManifestNamedAddress> for ManifestGlobalAddress {
    fn from(value: ManifestNamedAddress) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<ManifestAddress> for ManifestGlobalAddress {
    type Error = ParseGlobalAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}

// Alias for backwards-compatibility
pub type DynamicPackageAddress = ManifestPackageAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManifestPackageAddress {
    Static(PackageAddress),
    Named(ManifestNamedAddress),
}

scrypto_describe_for_manifest_type!(
    ManifestPackageAddress,
    PACKAGE_ADDRESS_TYPE,
    package_address_type_data,
);

labelled_resolvable_address!(ManifestPackageAddress);

impl Categorize<ManifestCustomValueKind> for ManifestPackageAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for ManifestPackageAddress
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::Static(address) => {
                encoder.write_discriminator(0)?;
                encoder.write_slice(address.as_node_id().as_bytes())?;
            }
            Self::Named(address_id) => {
                encoder.write_discriminator(1)?;
                encoder.write_slice(&address_id.0.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for ManifestPackageAddress
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            MANIFEST_ADDRESS_DISCRIMINATOR_STATIC => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                Ok(Self::Static(
                    PackageAddress::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            MANIFEST_ADDRESS_DISCRIMINATOR_NAMED => {
                let slice = decoder.read_slice(4)?;
                let id = u32::from_le_bytes(slice.try_into().unwrap());
                Ok(Self::Named(ManifestNamedAddress(id)))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl ManifestPackageAddress {
    /// This is to support either `Address("static_address")` or `NamedAddress("abc")` in manifest instruction,
    /// instead of `Enum<0u8>(Address("static_address"))`.
    pub fn to_instruction_argument(&self) -> ManifestValue {
        match self {
            Self::Static(address) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Static(
                    address.into_node_id(),
                )),
            },
            Self::Named(id) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Named(*id)),
            },
        }
    }

    pub fn is_static_global_package_of(&self, package_address: &PackageAddress) -> bool {
        match self {
            Self::Static(address) => address.as_node_id().eq(package_address.as_node_id()),
            Self::Named(_) => false,
        }
    }
}

impl From<PackageAddress> for ManifestPackageAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<ManifestNamedAddress> for ManifestPackageAddress {
    fn from(value: ManifestNamedAddress) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<GlobalAddress> for ManifestPackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(value: GlobalAddress) -> Result<Self, Self::Error> {
        Ok(Self::Static(PackageAddress::try_from(
            value.into_node_id(),
        )?))
    }
}

impl TryFrom<ManifestAddress> for ManifestPackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}

/// This trait resolves a static resource address for manifest instructions which
/// (as of Cuttlefish) only support a fixed address.
///
/// We hope to remove this restriction and enable these instructions to take a
/// dynamic package address at a protocol update soon.
pub trait ResolvableStaticManifestPackageAddress: Sized {
    fn resolve_static(self) -> PackageAddress;
}

impl<A, E> ResolvableStaticManifestPackageAddress for A
where
    A: TryInto<ManifestPackageAddress, Error = E>,
    E: Debug,
{
    fn resolve_static(self) -> PackageAddress {
        let address = self
            .try_into()
            .expect("Address was not a valid ManifestPackageAddress");
        match address {
            ManifestPackageAddress::Static(address) => address,
            ManifestPackageAddress::Named(_) => {
                panic!("This address needs to be a static/fixed address")
            }
        }
    }
}

// Alias for backwards-compatibility
pub type DynamicComponentAddress = ManifestComponentAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManifestComponentAddress {
    Static(ComponentAddress),
    Named(ManifestNamedAddress),
}

scrypto_describe_for_manifest_type!(
    ManifestComponentAddress,
    COMPONENT_ADDRESS_TYPE,
    component_address_type_data,
);

labelled_resolvable_address!(ManifestComponentAddress);

impl Categorize<ManifestCustomValueKind> for ManifestComponentAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for ManifestComponentAddress
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::Static(address) => {
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_STATIC)?;
                encoder.write_slice(address.as_node_id().as_bytes())?;
            }
            Self::Named(address_id) => {
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_NAMED)?;
                encoder.write_slice(&address_id.0.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for ManifestComponentAddress
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            MANIFEST_ADDRESS_DISCRIMINATOR_STATIC => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                Ok(Self::Static(
                    ComponentAddress::try_from(slice)
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            MANIFEST_ADDRESS_DISCRIMINATOR_NAMED => {
                let slice = decoder.read_slice(4)?;
                let id = u32::from_le_bytes(slice.try_into().unwrap());
                Ok(Self::Named(ManifestNamedAddress(id)))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl From<ComponentAddress> for ManifestComponentAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::Static(value)
    }
}

impl From<ManifestNamedAddress> for ManifestComponentAddress {
    fn from(value: ManifestNamedAddress) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<GlobalAddress> for ManifestComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(value: GlobalAddress) -> Result<Self, Self::Error> {
        Ok(Self::Static(ComponentAddress::try_from(
            value.into_node_id(),
        )?))
    }
}

impl TryFrom<ManifestAddress> for ManifestComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}

// Alias for backwards compatibility
pub type DynamicResourceAddress = ManifestResourceAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManifestResourceAddress {
    Static(ResourceAddress),
    Named(ManifestNamedAddress),
}

scrypto_describe_for_manifest_type!(
    ManifestResourceAddress,
    RESOURCE_ADDRESS_TYPE,
    resource_address_type_data,
);

labelled_resolvable_address!(ManifestResourceAddress);

impl Categorize<ManifestCustomValueKind> for ManifestResourceAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for ManifestResourceAddress
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::Static(address) => {
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_STATIC)?;
                encoder.write_slice(address.as_node_id().as_bytes())?;
            }
            Self::Named(address_id) => {
                encoder.write_discriminator(MANIFEST_ADDRESS_DISCRIMINATOR_NAMED)?;
                encoder.write_slice(&address_id.0.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for ManifestResourceAddress
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_discriminator()? {
            MANIFEST_ADDRESS_DISCRIMINATOR_STATIC => {
                let slice = decoder.read_slice(NodeId::LENGTH)?;
                Ok(Self::Static(
                    ResourceAddress::try_from(slice)
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            MANIFEST_ADDRESS_DISCRIMINATOR_NAMED => {
                let slice = decoder.read_slice(4)?;
                let id = u32::from_le_bytes(slice.try_into().unwrap());
                Ok(Self::Named(ManifestNamedAddress(id)))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl From<ResourceAddress> for ManifestResourceAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::Static(value)
    }
}

impl From<ManifestNamedAddress> for ManifestResourceAddress {
    fn from(value: ManifestNamedAddress) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<GlobalAddress> for ManifestResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(value: GlobalAddress) -> Result<Self, Self::Error> {
        Ok(Self::Static(ResourceAddress::try_from(
            value.into_node_id(),
        )?))
    }
}

impl TryFrom<ManifestAddress> for ManifestResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}

/// This trait resolves a static resource address for manifest instructions which
/// (as of Cuttlefish) only support a fixed address.
///
/// We hope to remove this restriction and enable these instructions to take a
/// dynamic package address at a protocol update soon.
pub trait ResolvableStaticManifestResourceAddress: Sized {
    fn resolve_static(self) -> ResourceAddress;
}

impl<A, E> ResolvableStaticManifestResourceAddress for A
where
    A: TryInto<ManifestResourceAddress, Error = E>,
    E: Debug,
{
    fn resolve_static(self) -> ResourceAddress {
        let address = self
            .try_into()
            .expect("Address was not a valid ManifestResourceAddress");
        match address {
            ManifestResourceAddress::Static(address) => address,
            ManifestResourceAddress::Named(_) => {
                panic!("This address needs to be a static/fixed address")
            }
        }
    }
}
