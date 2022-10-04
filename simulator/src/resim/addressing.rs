use core::fmt;
use std::str::FromStr;

use radix_engine::types::{
    AddressError, Bech32Decoder, Bech32Encoder, ComponentAddress, PackageAddress, ResourceAddress,
};

#[derive(Clone)]
pub struct SimulatorPackageAddress(pub PackageAddress);

impl From<SimulatorPackageAddress> for PackageAddress {
    fn from(simulator_address: SimulatorPackageAddress) -> Self {
        simulator_address.0
    }
}

impl FromStr for SimulatorPackageAddress {
    type Err = AddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        let address =
            Bech32Decoder::for_simulator().validate_and_decode_package_address(address)?;
        Ok(Self(address))
    }
}

impl fmt::Display for SimulatorPackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let address = Bech32Encoder::for_simulator().encode_package_address(&self.0);
        write!(f, "{}", address)
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

impl FromStr for SimulatorResourceAddress {
    type Err = AddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        let address =
            Bech32Decoder::for_simulator().validate_and_decode_resource_address(address)?;
        Ok(Self(address))
    }
}

impl fmt::Display for SimulatorResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let address = Bech32Encoder::for_simulator().encode_resource_address(&self.0);
        write!(f, "{}", address)
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

impl FromStr for SimulatorComponentAddress {
    type Err = AddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        let address =
            Bech32Decoder::for_simulator().validate_and_decode_component_address(address)?;
        Ok(Self(address))
    }
}

impl fmt::Display for SimulatorComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let address = Bech32Encoder::for_simulator().encode_component_address(&self.0);
        write!(f, "{}", address)
    }
}

impl fmt::Debug for SimulatorComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
