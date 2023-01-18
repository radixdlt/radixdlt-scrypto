use radix_engine_interface::rule;
use sbor::rust::fmt;
use std::str::FromStr;

use radix_engine::types::{
    require, AccessRule, AddressError, Bech32Decoder, Bech32Encoder, ComponentAddress,
    NonFungibleAddress, NonFungibleLocalId, PackageAddress, ParseNonFungibleLocalIdError, ResourceAddress,
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
pub struct SimulatorNonFungibleAddress(pub NonFungibleAddress);

impl From<SimulatorNonFungibleAddress> for NonFungibleAddress {
    fn from(simulator_address: SimulatorNonFungibleAddress) -> Self {
        simulator_address.0
    }
}

impl From<NonFungibleAddress> for SimulatorNonFungibleAddress {
    fn from(address: NonFungibleAddress) -> Self {
        Self(address)
    }
}

impl FromStr for SimulatorNonFungibleAddress {
    type Err = ParseNonFungibleAddressError;

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
            return Err(ParseNonFungibleAddressError::InvalidLengthError(
                tokens.len(),
            ));
        }

        // Paring the resource address fully first to use it for the non-fungible id type ledger
        // lookup
        let resource_address_string = tokens[0];
        let resource_address = Bech32Decoder::for_simulator()
            .validate_and_decode_resource_address(resource_address_string)
            .map_err(ParseNonFungibleAddressError::InvalidResourceAddress)?;
        let non_fungible_local_id_type = lookup_non_fungible_local_id_type(&resource_address)
            .map_err(ParseNonFungibleAddressError::LedgerLookupError)?;

        // Parsing the non-fungible id given the non-fungible id type above
        let non_fungible_local_id =
            NonFungibleLocalId::try_from_simple_string(non_fungible_local_id_type, tokens[1])
                .map_err(ParseNonFungibleAddressError::InvalidNonFungibleLocalId)?;
        Ok(Self(NonFungibleAddress::new(
            resource_address,
            non_fungible_local_id,
        )))
    }
}

impl fmt::Display for SimulatorNonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_canonical_string(&Bech32Encoder::for_simulator())
        )
    }
}

impl fmt::Debug for SimulatorNonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub enum SimulatorResourceOrNonFungibleAddress {
    ResourceAddress(SimulatorResourceAddress),
    NonFungibleAddress(SimulatorNonFungibleAddress),
}

impl From<SimulatorResourceAddress> for SimulatorResourceOrNonFungibleAddress {
    fn from(address: SimulatorResourceAddress) -> Self {
        Self::ResourceAddress(address)
    }
}

impl From<SimulatorNonFungibleAddress> for SimulatorResourceOrNonFungibleAddress {
    fn from(address: SimulatorNonFungibleAddress) -> Self {
        Self::NonFungibleAddress(address)
    }
}

impl From<SimulatorResourceOrNonFungibleAddress> for AccessRule {
    fn from(address: SimulatorResourceOrNonFungibleAddress) -> Self {
        match address {
            SimulatorResourceOrNonFungibleAddress::ResourceAddress(resource_address) => {
                rule!(require(resource_address.0))
            }
            SimulatorResourceOrNonFungibleAddress::NonFungibleAddress(non_fungible_address) => {
                rule!(require(non_fungible_address.0))
            }
        }
    }
}

impl FromStr for SimulatorResourceOrNonFungibleAddress {
    type Err = ParseSimulatorResourceOrNonFungibleAddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        if address.contains(":") {
            SimulatorNonFungibleAddress::from_str(address)
                .map_err(ParseSimulatorResourceOrNonFungibleAddressError::from)
                .map(Self::NonFungibleAddress)
        } else {
            SimulatorResourceAddress::from_str(address)
                .map_err(ParseSimulatorResourceOrNonFungibleAddressError::from)
                .map(Self::ResourceAddress)
        }
    }
}

impl fmt::Display for SimulatorResourceOrNonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFungibleAddress(non_fungible_address) => non_fungible_address.fmt(f),
            Self::ResourceAddress(resource_address) => resource_address.fmt(f),
        }
    }
}

impl fmt::Debug for SimulatorResourceOrNonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFungibleAddress(non_fungible_address) => non_fungible_address.fmt(f),
            Self::ResourceAddress(resource_address) => resource_address.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ParseSimulatorResourceOrNonFungibleAddressError {
    ParseNonFungibleAddressError(ParseNonFungibleAddressError),
    ParseResourceAddressError(AddressError),
}

impl From<ParseNonFungibleAddressError> for ParseSimulatorResourceOrNonFungibleAddressError {
    fn from(error: ParseNonFungibleAddressError) -> Self {
        Self::ParseNonFungibleAddressError(error)
    }
}

impl From<AddressError> for ParseSimulatorResourceOrNonFungibleAddressError {
    fn from(error: AddressError) -> Self {
        Self::ParseResourceAddressError(error)
    }
}

impl fmt::Display for ParseSimulatorResourceOrNonFungibleAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseSimulatorResourceOrNonFungibleAddressError {}

#[derive(Debug)]
pub enum ParseNonFungibleAddressError {
    InvalidLengthError(usize),
    InvalidResourceAddress(AddressError),
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
    LedgerLookupError(LedgerLookupError),
}

impl fmt::Display for ParseNonFungibleAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseNonFungibleAddressError {}
