use bech32::{self, ToBase32, Variant};
use once_cell::unsync::Lazy;
use sbor::rust::string::String;

use super::entity::EntityType;
use super::errors::AddressError;
use super::hrpset::HrpSet;
use crate::component::{ComponentAddress, PackageAddress};
use crate::core::NetworkDefinition;
use crate::misc::combine;
use crate::resource::ResourceAddress;

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
    pub fn encode_package_address(&self, package_address: &PackageAddress) -> String {
        match package_address {
            PackageAddress::Normal(data) => self.encode(EntityType::package(package_address), data),
        }
        .expect("Failed to encode package address as Bech32")
    }

    /// Encodes a component address in Bech32 and returns a String on success or an `AddressError` on failure.
    pub fn encode_component_address(&self, component_address: &ComponentAddress) -> String {
        match component_address {
            ComponentAddress::Normal(data)
            | ComponentAddress::Account(data)
            | ComponentAddress::System(data) => {
                self.encode(EntityType::component(component_address), data)
            }
        }
        .expect("Failed to encode component address as Bech32")
    }

    /// Encodes a resource address in Bech32 and returns a String on success or an `AddressError` on failure.
    pub fn encode_resource_address(&self, resource_address: &ResourceAddress) -> String {
        match resource_address {
            ResourceAddress::Normal(data) => {
                self.encode(EntityType::resource(resource_address), data)
            }
        }
        .expect("Failed to encode resource address as Bech32")
    }

    /// Low level method which performs the Bech32 encoding of the data.
    fn encode(&self, entity_type: EntityType, other_data: &[u8]) -> Result<String, AddressError> {
        // Obtain the HRP corresponding to this entity type
        let hrp = self.hrp_set.get_entity_hrp(&entity_type);

        let full_data = combine(entity_type.id(), other_data);

        let bech32_string = bech32::encode(hrp, full_data.to_base32(), Variant::Bech32m)
            .map_err(|err| AddressError::EncodingError(err))?;

        Ok(bech32_string)
    }
}

pub const BECH32_ENCODER: Lazy<Bech32Encoder> =
    Lazy::new(|| Bech32Encoder::new(&NetworkDefinition::local_simulator()));
