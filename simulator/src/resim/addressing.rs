use radix_engine::types::{
    Bech32Decoder, Bech32Encoder, ComponentAddress, NonFungibleGlobalId, PackageAddress,
    ResourceAddress,
};
use radix_engine_interface::{
    blueprints::resource::{require, AccessRule, ParseNonFungibleGlobalIdError},
    rule,
};
use sbor::rust::fmt;
use std::str::FromStr;
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    InvalidAddress(String),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for AddressError {}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct SimulatorPackageAddress(pub PackageAddress);

impl From<SimulatorPackageAddress> for PackageAddress {
    fn from(simulator_address: SimulatorPackageAddress) -> Self {
        simulator_address.0
    }
}

impl From<PackageAddress> for SimulatorPackageAddress {
    fn from(address: PackageAddress) -> Self {
        Self(address)
    }
}

impl FromStr for SimulatorPackageAddress {
    type Err = AddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        PackageAddress::try_from_hex(address)
            .or(PackageAddress::try_from_bech32(
                &Bech32Decoder::for_simulator(),
                address,
            ))
            .ok_or(AddressError::InvalidAddress(address.to_string()))
            .map(|x| Self(x))
    }
}

impl fmt::Display for SimulatorPackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.display(&Bech32Encoder::for_simulator()))
    }
}

impl fmt::Debug for SimulatorPackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct SimulatorResourceAddress(pub ResourceAddress);

impl From<SimulatorResourceAddress> for ResourceAddress {
    fn from(simulator_address: SimulatorResourceAddress) -> Self {
        simulator_address.0
    }
}

impl From<ResourceAddress> for SimulatorResourceAddress {
    fn from(address: ResourceAddress) -> Self {
        Self(address)
    }
}

impl FromStr for SimulatorResourceAddress {
    type Err = AddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        ResourceAddress::try_from_hex(address)
            .or(ResourceAddress::try_from_bech32(
                &Bech32Decoder::for_simulator(),
                address,
            ))
            .ok_or(AddressError::InvalidAddress(address.to_string()))
            .map(|x| Self(x))
    }
}

impl fmt::Display for SimulatorResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.display(&Bech32Encoder::for_simulator()))
    }
}

impl fmt::Debug for SimulatorResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct SimulatorComponentAddress(pub ComponentAddress);

impl From<SimulatorComponentAddress> for ComponentAddress {
    fn from(simulator_address: SimulatorComponentAddress) -> Self {
        simulator_address.0
    }
}

impl From<ComponentAddress> for SimulatorComponentAddress {
    fn from(address: ComponentAddress) -> Self {
        Self(address)
    }
}

impl FromStr for SimulatorComponentAddress {
    type Err = AddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        ComponentAddress::try_from_hex(address)
            .or(ComponentAddress::try_from_bech32(
                &Bech32Decoder::for_simulator(),
                address,
            ))
            .ok_or(AddressError::InvalidAddress(address.to_string()))
            .map(|x| Self(x))
    }
}

impl fmt::Display for SimulatorComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.display(&Bech32Encoder::for_simulator()))
    }
}

impl fmt::Debug for SimulatorComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct SimulatorNonFungibleGlobalId(pub NonFungibleGlobalId);

impl From<SimulatorNonFungibleGlobalId> for NonFungibleGlobalId {
    fn from(global_id: SimulatorNonFungibleGlobalId) -> Self {
        global_id.0
    }
}

impl From<NonFungibleGlobalId> for SimulatorNonFungibleGlobalId {
    fn from(address: NonFungibleGlobalId) -> Self {
        Self(address)
    }
}

impl FromStr for SimulatorNonFungibleGlobalId {
    type Err = ParseNonFungibleGlobalIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let global_id =
            NonFungibleGlobalId::try_from_canonical_string(&Bech32Decoder::for_simulator(), s)?;
        Ok(Self(global_id))
    }
}

impl fmt::Display for SimulatorNonFungibleGlobalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_canonical_string(&Bech32Encoder::for_simulator())
        )
    }
}

impl fmt::Debug for SimulatorNonFungibleGlobalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub enum SimulatorResourceOrNonFungibleGlobalId {
    ResourceAddress(SimulatorResourceAddress),
    NonFungibleGlobalId(SimulatorNonFungibleGlobalId),
}

impl From<SimulatorResourceAddress> for SimulatorResourceOrNonFungibleGlobalId {
    fn from(address: SimulatorResourceAddress) -> Self {
        Self::ResourceAddress(address)
    }
}

impl From<SimulatorNonFungibleGlobalId> for SimulatorResourceOrNonFungibleGlobalId {
    fn from(address: SimulatorNonFungibleGlobalId) -> Self {
        Self::NonFungibleGlobalId(address)
    }
}

impl From<SimulatorResourceOrNonFungibleGlobalId> for AccessRule {
    fn from(address: SimulatorResourceOrNonFungibleGlobalId) -> Self {
        match address {
            SimulatorResourceOrNonFungibleGlobalId::ResourceAddress(resource_address) => {
                rule!(require(resource_address.0))
            }
            SimulatorResourceOrNonFungibleGlobalId::NonFungibleGlobalId(non_fungible_global_id) => {
                rule!(require(non_fungible_global_id.0))
            }
        }
    }
}

impl FromStr for SimulatorResourceOrNonFungibleGlobalId {
    type Err = ParseSimulatorResourceOrNonFungibleGlobalIdError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        if address.contains(':') {
            SimulatorNonFungibleGlobalId::from_str(address)
                .map_err(ParseSimulatorResourceOrNonFungibleGlobalIdError::from)
                .map(Self::NonFungibleGlobalId)
        } else {
            SimulatorResourceAddress::from_str(address)
                .map_err(ParseSimulatorResourceOrNonFungibleGlobalIdError::from)
                .map(Self::ResourceAddress)
        }
    }
}

impl fmt::Display for SimulatorResourceOrNonFungibleGlobalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFungibleGlobalId(non_fungible_global_id) => non_fungible_global_id.fmt(f),
            Self::ResourceAddress(resource_address) => resource_address.fmt(f),
        }
    }
}

impl fmt::Debug for SimulatorResourceOrNonFungibleGlobalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFungibleGlobalId(non_fungible_global_id) => non_fungible_global_id.fmt(f),
            Self::ResourceAddress(resource_address) => resource_address.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ParseSimulatorResourceOrNonFungibleGlobalIdError {
    ParseNonFungibleGlobalIdError(ParseNonFungibleGlobalIdError),
    ParseResourceAddressError(AddressError),
}

impl From<ParseNonFungibleGlobalIdError> for ParseSimulatorResourceOrNonFungibleGlobalIdError {
    fn from(error: ParseNonFungibleGlobalIdError) -> Self {
        Self::ParseNonFungibleGlobalIdError(error)
    }
}

impl From<AddressError> for ParseSimulatorResourceOrNonFungibleGlobalIdError {
    fn from(error: AddressError) -> Self {
        Self::ParseResourceAddressError(error)
    }
}

impl fmt::Display for ParseSimulatorResourceOrNonFungibleGlobalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseSimulatorResourceOrNonFungibleGlobalIdError {}
