use radix_engine_interface::rule;
use sbor::rust::fmt;
use std::str::FromStr;

use radix_engine::types::{
    require, AccessRule, AddressError, Bech32Decoder, Bech32Encoder, ComponentAddress,
    NonFungibleGlobalId, NonFungibleLocalId, PackageAddress, ParseNonFungibleLocalIdError, ResourceAddress,
};
use utils::ContextualDisplay;

use crate::ledger::{lookup_non_fungible_local_id_type, LedgerLookupError};

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
        if let Ok(address) = PackageAddress::try_from_hex(address) {
            return Ok(address.into());
        }
        let address =
            Bech32Decoder::for_simulator().validate_and_decode_package_address(address)?;
        Ok(Self(address))
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
        if let Ok(address) = ResourceAddress::try_from_hex(address) {
            return Ok(address.into());
        }
        let address =
            Bech32Decoder::for_simulator().validate_and_decode_resource_address(address)?;
        Ok(Self(address))
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
        if let Ok(address) = ComponentAddress::try_from_hex(address) {
            return Ok(address.into());
        }
        let address =
            Bech32Decoder::for_simulator().validate_and_decode_component_address(address)?;
        Ok(Self(address))
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
    fn from(simulator_address: SimulatorNonFungibleGlobalId) -> Self {
        simulator_address.0
    }
}

impl From<NonFungibleGlobalId> for SimulatorNonFungibleGlobalId {
    fn from(address: NonFungibleGlobalId) -> Self {
        Self(address)
    }
}

impl FromStr for SimulatorNonFungibleGlobalId {
    type Err = ParseNonFungibleGlobalIdError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        // Non-fungible addresses provided as arguments to the simulator can use the
        // same string format as that used in the explorer and wallet since we can
        // do ledger lookups to determine the non-fungible id type that the resource
        // uses.

        // Splitting up the input into two parts: the resource address and the non-fungible ids
        let tokens = address
            .trim()
            .split(':')
            .map(|s| s.trim())
            .collect::<Vec<_>>();

        // There MUST only be two tokens in the tokens vector, one for the resource address, and
        // another for the non-fungible ids. If there is more or less, then this function returns
        // an error.
        if tokens.len() != 2 {
            return Err(ParseNonFungibleGlobalIdError::InvalidLengthError(
                tokens.len(),
            ));
        }

        // Paring the resource address fully first to use it for the non-fungible id type ledger
        // lookup
        let resource_address_string = tokens[0];
        let resource_address = Bech32Decoder::for_simulator()
            .validate_and_decode_resource_address(resource_address_string)
            .map_err(ParseNonFungibleGlobalIdError::InvalidResourceAddress)?;
        let non_fungible_local_id_type = lookup_non_fungible_local_id_type(&resource_address)
            .map_err(ParseNonFungibleGlobalIdError::LedgerLookupError)?;

        // Parsing the non-fungible id given the non-fungible id type above
        let non_fungible_local_id =
            NonFungibleLocalId::try_from_simple_string(non_fungible_local_id_type, tokens[1])
                .map_err(ParseNonFungibleGlobalIdError::InvalidNonFungibleLocalId)?;
        Ok(Self(NonFungibleGlobalId::new(
            resource_address,
            non_fungible_local_id,
        )))
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
        if address.contains(":") {
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

#[derive(Debug)]
pub enum ParseNonFungibleGlobalIdError {
    InvalidLengthError(usize),
    InvalidResourceAddress(AddressError),
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
    LedgerLookupError(LedgerLookupError),
}

impl fmt::Display for ParseNonFungibleGlobalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseNonFungibleGlobalIdError {}
