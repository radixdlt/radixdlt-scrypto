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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicGlobalAddress {
    Static(GlobalAddress),
    Named(u32),
}

scrypto_describe_for_manifest_type!(
    DynamicGlobalAddress,
    GLOBAL_ADDRESS_TYPE,
    global_address_type_data,
);

impl Categorize<ManifestCustomValueKind> for DynamicGlobalAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for DynamicGlobalAddress
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
                encoder.write_slice(&address_id.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for DynamicGlobalAddress
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
                Ok(Self::Named(id))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl DynamicGlobalAddress {
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

impl From<GlobalAddress> for DynamicGlobalAddress {
    fn from(value: GlobalAddress) -> Self {
        Self::Static(value)
    }
}

impl From<PackageAddress> for DynamicGlobalAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<DynamicPackageAddress> for DynamicGlobalAddress {
    fn from(value: DynamicPackageAddress) -> Self {
        match value {
            DynamicPackageAddress::Static(value) => Self::Static(value.into()),
            DynamicPackageAddress::Named(value) => Self::Named(value),
        }
    }
}

impl From<ResourceAddress> for DynamicGlobalAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<DynamicResourceAddress> for DynamicGlobalAddress {
    fn from(value: DynamicResourceAddress) -> Self {
        match value {
            DynamicResourceAddress::Static(value) => Self::Static(value.into()),
            DynamicResourceAddress::Named(value) => Self::Named(value),
        }
    }
}

impl From<ComponentAddress> for DynamicGlobalAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<DynamicComponentAddress> for DynamicGlobalAddress {
    fn from(value: DynamicComponentAddress) -> Self {
        match value {
            DynamicComponentAddress::Static(value) => Self::Static(value.into()),
            DynamicComponentAddress::Named(value) => Self::Named(value),
        }
    }
}

impl From<u32> for DynamicGlobalAddress {
    fn from(value: u32) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<ManifestAddress> for DynamicGlobalAddress {
    type Error = ParseGlobalAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicPackageAddress {
    Static(PackageAddress),
    Named(u32),
}

scrypto_describe_for_manifest_type!(
    DynamicPackageAddress,
    PACKAGE_ADDRESS_TYPE,
    package_address_type_data,
);

impl Categorize<ManifestCustomValueKind> for DynamicPackageAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for DynamicPackageAddress
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
                encoder.write_slice(&address_id.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for DynamicPackageAddress
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
                Ok(Self::Named(id))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl DynamicPackageAddress {
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

impl From<PackageAddress> for DynamicPackageAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<u32> for DynamicPackageAddress {
    fn from(value: u32) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<GlobalAddress> for DynamicPackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(value: GlobalAddress) -> Result<Self, Self::Error> {
        Ok(Self::Static(PackageAddress::try_from(
            value.into_node_id(),
        )?))
    }
}

impl TryFrom<ManifestAddress> for DynamicPackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicComponentAddress {
    Static(ComponentAddress),
    Named(u32),
}

scrypto_describe_for_manifest_type!(
    DynamicComponentAddress,
    COMPONENT_ADDRESS_TYPE,
    component_address_type_data,
);

impl Categorize<ManifestCustomValueKind> for DynamicComponentAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for DynamicComponentAddress
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
                encoder.write_slice(&address_id.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for DynamicComponentAddress
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
                Ok(Self::Named(id))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl From<ComponentAddress> for DynamicComponentAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::Static(value)
    }
}

impl From<u32> for DynamicComponentAddress {
    fn from(value: u32) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<GlobalAddress> for DynamicComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(value: GlobalAddress) -> Result<Self, Self::Error> {
        Ok(Self::Static(ComponentAddress::try_from(
            value.into_node_id(),
        )?))
    }
}

impl TryFrom<ManifestAddress> for DynamicComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicResourceAddress {
    Static(ResourceAddress),
    Named(u32),
}

scrypto_describe_for_manifest_type!(
    DynamicResourceAddress,
    RESOURCE_ADDRESS_TYPE,
    resource_address_type_data,
);

impl Categorize<ManifestCustomValueKind> for DynamicResourceAddress {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for DynamicResourceAddress
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
                encoder.write_slice(&address_id.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for DynamicResourceAddress
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
                Ok(Self::Named(id))
            }
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl From<ResourceAddress> for DynamicResourceAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::Static(value)
    }
}

impl From<u32> for DynamicResourceAddress {
    fn from(value: u32) -> Self {
        Self::Named(value)
    }
}

impl TryFrom<GlobalAddress> for DynamicResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(value: GlobalAddress) -> Result<Self, Self::Error> {
        Ok(Self::Static(ResourceAddress::try_from(
            value.into_node_id(),
        )?))
    }
}

impl TryFrom<ManifestAddress> for DynamicResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(value: ManifestAddress) -> Result<Self, Self::Error> {
        Ok(match value {
            ManifestAddress::Static(value) => Self::Static(value.try_into()?),
            ManifestAddress::Named(value) => Self::Named(value),
        })
    }
}
