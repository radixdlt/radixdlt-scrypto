use sbor::rust::vec::Vec;

use crate::core::Network;
use crate::engine::types::{ComponentAddress, PackageAddress, ResourceAddress};

use super::entity::{
    EntityType, ALLOWED_COMPONENT_ENTITY_TYPES, ALLOWED_PACKAGE_ENTITY_TYPES,
    ALLOWED_RESOURCE_ENTITY_TYPES,
};
use super::errors::AddressError;
use super::hrpset::{get_network_hrp_set, HrpSet};

use bech32::{self, FromBase32, Variant};
use once_cell::unsync::Lazy;

/// Represents a decoder which understands how to decode Scrypto addresses in Bech32.
pub struct Bech32Decoder {
    pub hrp_set: HrpSet,
}

impl Bech32Decoder {
    /// Instantiates a new Bech32Decoder with the HRP corresponding to the passed network.
    pub fn new_from_network(network: &Network) -> Self {
        Self {
            hrp_set: get_network_hrp_set(network),
        }
    }

    /// Decodes a package address string from a Bech32 string into a `PackageAddress` and returns an `AddressError` on
    /// validation or decoding failure.
    pub fn validate_and_decode_package_address(
        &self,
        package_address: &str,
    ) -> Result<PackageAddress, AddressError> {
        Ok(PackageAddress::try_from(
            self.validate_and_decode(package_address, &ALLOWED_PACKAGE_ENTITY_TYPES)?
                .as_slice(),
        )?)
    }

    /// Decodes a component address string from a Bech32 string into a `ComponentAddress` and returns an `AddressError` on
    /// validation or decoding failure.
    pub fn validate_and_decode_component_address(
        &self,
        component_address: &str,
    ) -> Result<ComponentAddress, AddressError> {
        Ok(ComponentAddress::try_from(
            self.validate_and_decode(component_address, &ALLOWED_COMPONENT_ENTITY_TYPES)?
                .as_slice(),
        )?)
    }

    /// Decodes a resource address string from a Bech32 string into a `ResourceAddress` and returns an `AddressError` on
    /// validation or decoding failure.
    pub fn validate_and_decode_resource_address(
        &self,
        resource_address: &str,
    ) -> Result<ResourceAddress, AddressError> {
        Ok(ResourceAddress::try_from(
            self.validate_and_decode(resource_address, &ALLOWED_RESOURCE_ENTITY_TYPES)?
                .as_slice(),
        )?)
    }

    /// Low level method which performs the Bech32 validation and decoding of the data.
    fn validate_and_decode(
        &self,
        address: &str,
        allowed_entity_types: &[EntityType],
    ) -> Result<Vec<u8>, AddressError> {
        // Decode the address string
        let (actual_hrp, data, variant) =
            bech32::decode(address).map_err(|err| AddressError::DecodingError(err))?;

        // Validate the Bech32 variant to ensure that is is Bech32m
        match variant {
            Variant::Bech32m => {}
            _ => return Err(AddressError::InvalidVariant(variant)),
        };

        // Convert the data to u8 from u5.
        let data = Vec::<u8>::from_base32(&data).map_err(|err| AddressError::DecodingError(err))?;

        // Obtain the HRP based on the entity byte in the data
        let expected_hrp = if let Some(entity_type_id) = data.get(0) {
            let entity_type = EntityType::try_from(*entity_type_id)
                .map_err(|_| AddressError::InvalidEntityTypeId(*entity_type_id))?;

            // Enure that the entity type of the address matches the allowed list of entity types
            if !allowed_entity_types.contains(&entity_type) {
                return Err(AddressError::InvalidEntityTypeId(*entity_type_id));
            }

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

#[cfg(target_arch = "wasm32")]
pub const BECH32_DECODER: Lazy<Bech32Decoder> = Lazy::new(|| {
    use crate::core::Runtime;
    Bech32Decoder::new_from_network(&Runtime::transaction_network())
});

#[cfg(not(target_arch = "wasm32"))]
pub const BECH32_DECODER: Lazy<Bech32Decoder> =
    Lazy::new(|| Bech32Decoder::new_from_network(&Network::LocalSimulator));
