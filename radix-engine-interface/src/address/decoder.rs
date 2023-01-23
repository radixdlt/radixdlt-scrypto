use sbor::rust::vec::Vec;

use crate::model::*;
use crate::node::NetworkDefinition;

use super::entity::EntityType;
use super::errors::AddressError;
use super::hrpset::HrpSet;

use bech32::{self, FromBase32, Variant};

/// Represents a decoder which understands how to decode Scrypto addresses in Bech32.
pub struct Bech32Decoder {
    pub hrp_set: HrpSet,
}

impl Bech32Decoder {
    pub fn for_simulator() -> Self {
        Self::new(&NetworkDefinition::simulator())
    }

    /// Instantiates a new Bech32Decoder with the HRP corresponding to the passed network.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            hrp_set: network.into(),
        }
    }

    /// Decodes a package address string from a Bech32 string into a `PackageAddress` and returns an `AddressError` on
    /// validation or decoding failure.
    pub fn validate_and_decode_package_address(
        &self,
        package_address: &str,
    ) -> Result<PackageAddress, AddressError> {
        Ok(PackageAddress::try_from(
            self.validate_and_decode(package_address)?.as_slice(),
        )?)
    }

    /// Decodes a component address string from a Bech32 string into a `ComponentAddress` and returns an `AddressError` on
    /// validation or decoding failure.
    pub fn validate_and_decode_component_address(
        &self,
        component_address: &str,
    ) -> Result<ComponentAddress, AddressError> {
        Ok(ComponentAddress::try_from(
            self.validate_and_decode(component_address)?.as_slice(),
        )?)
    }

    /// Decodes a resource address string from a Bech32 string into a `ResourceAddress` and returns an `AddressError` on
    /// validation or decoding failure.
    pub fn validate_and_decode_resource_address(
        &self,
        resource_address: &str,
    ) -> Result<ResourceAddress, AddressError> {
        Ok(ResourceAddress::try_from(
            self.validate_and_decode(resource_address)?.as_slice(),
        )?)
    }

    /// Low level method which performs the Bech32 validation and decoding of the data.
    fn validate_and_decode(&self, address: &str) -> Result<Vec<u8>, AddressError> {
        // Decode the address string
        let (actual_hrp, data, variant) =
            bech32::decode(address).map_err(|err| AddressError::Bech32mDecodingError(err))?;

        // Validate the Bech32 variant to ensure that is is Bech32m
        match variant {
            Variant::Bech32m => {}
            _ => return Err(AddressError::InvalidVariant(variant)),
        };

        // Convert the data to u8 from u5.
        let data =
            Vec::<u8>::from_base32(&data).map_err(|err| AddressError::Bech32mDecodingError(err))?;

        // Obtain the HRP based on the entity byte in the data
        let expected_hrp = if let Some(entity_type_id) = data.get(0) {
            let entity_type = EntityType::try_from(*entity_type_id)
                .map_err(|_| AddressError::InvalidEntityTypeId(*entity_type_id))?;

            // Obtain the HRP corresponding to this entity type
            self.hrp_set.get_entity_hrp(&entity_type)
        } else {
            return Err(AddressError::DataSectionTooShort);
        };

        // Validate that the decoded HRP matches that corresponding to the entity byte
        if actual_hrp != expected_hrp {
            return Err(AddressError::InvalidHrp);
        }

        // Validation complete, return data bytes
        Ok(data)
    }
}
