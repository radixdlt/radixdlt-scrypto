use core::fmt;
use std::str::FromStr;

use radix_engine::types::{
    AddressError, Bech32Decoder, Bech32Encoder, ComponentAddress, PackageAddress, ResourceAddress,
};
use scrypto::address::ContextualDisplay;

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
