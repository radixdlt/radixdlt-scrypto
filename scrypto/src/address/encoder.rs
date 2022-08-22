use sbor::String;

use crate::component::{ComponentAddress, PackageAddress};
use crate::core::NetworkDefinition;
use crate::resource::ResourceAddress;

use super::entity::{
    EntityType, ALLOWED_COMPONENT_ENTITY_TYPES, ALLOWED_PACKAGE_ENTITY_TYPES,
    ALLOWED_RESOURCE_ENTITY_TYPES,
};
use super::errors::AddressError;
use super::hrpset::HrpSet;

use bech32::{self, ToBase32, Variant};
use once_cell::unsync::Lazy;

/// Represents an encoder which understands how to encode Scrypto addresses in Bech32.
pub struct Bech32Encoder {
    pub hrp_set: HrpSet,
}

impl Bech32Encoder {
    pub fn for_simulator() -> Self {
        Self::new(&NetworkDefinition::local_simulator())
    }

    /// Instantiates a new Bech32Encoder with the HRP corresponding to the passed network.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            hrp_set: network.into(),
        }
    }

    /// Encodes a package address in Bech32 and returns a String on success or an `AddressError` on failure.
    pub fn encode_package_address(
        &self,
        package_address: &PackageAddress,
    ) -> Result<String, AddressError> {
        Ok(self.encode(&package_address.0, &ALLOWED_PACKAGE_ENTITY_TYPES)?)
    }

    /// Encodes a component address in Bech32 and returns a String on success or an `AddressError` on failure.
    pub fn encode_component_address(
        &self,
        component_address: &ComponentAddress,
    ) -> Result<String, AddressError> {
        Ok(self.encode(&component_address.0, &ALLOWED_COMPONENT_ENTITY_TYPES)?)
    }

    /// Encodes a resource address in Bech32 and returns a String on success or an `AddressError` on failure.
    pub fn encode_resource_address(
        &self,
        resource_address: &ResourceAddress,
    ) -> Result<String, AddressError> {
        Ok(self.encode(&resource_address.0, &ALLOWED_RESOURCE_ENTITY_TYPES)?)
    }

    /// Low level method which performs the Bech32 encoding of the data.
    fn encode(
        &self,
        data: &[u8],
        allowed_entity_types: &[EntityType],
    ) -> Result<String, AddressError> {
        // Obtain the HRP used for the encoding of this address
        let hrp = if let Some(entity_type_id) = data.get(0) {
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

        let bech32_string = bech32::encode(hrp, data.to_base32(), Variant::Bech32m)
            .map_err(|err| AddressError::EncodingError(err))?;

        Ok(bech32_string)
    }
}

pub const BECH32_ENCODER: Lazy<Bech32Encoder> =
    Lazy::new(|| Bech32Encoder::new(&NetworkDefinition::local_simulator()));
