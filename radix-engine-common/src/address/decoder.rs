use super::hrpset::HrpSet;
use crate::address::DecodeBech32AddressError;
use crate::network::NetworkDefinition;
use crate::types::EntityType;
use bech32::{self, FromBase32, Variant};
use sbor::rust::prelude::*;

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

    /// Low level method which performs the Bech32 validation and decoding of the data.
    pub fn validate_and_decode(
        &self,
        address: &str,
    ) -> Result<(EntityType, Vec<u8>), DecodeBech32AddressError> {
        // Decode the address string
        let (actual_hrp, data, variant) = bech32::decode(address)
            .map_err(|err| DecodeBech32AddressError::Bech32mDecodingError(err))?;

        // Validate the Bech32 variant to ensure that is is Bech32m
        match variant {
            Variant::Bech32m => {}
            _ => return Err(DecodeBech32AddressError::InvalidVariant(variant)),
        };

        // Convert the data to u8 from u5.
        let data = Vec::<u8>::from_base32(&data)
            .map_err(|err| DecodeBech32AddressError::Bech32mDecodingError(err))?;

        // Obtain the HRP based on the entity byte in the data
        let (entity_type, expected_hrp) = if let Some(entity_type_id) = data.get(0) {
            let entity_type = EntityType::from_repr(*entity_type_id).ok_or(
                DecodeBech32AddressError::InvalidEntityTypeId(*entity_type_id),
            )?;

            // Obtain the HRP corresponding to this entity type
            (entity_type, self.hrp_set.get_entity_hrp(&entity_type))
        } else {
            return Err(DecodeBech32AddressError::MissingEntityTypeByte);
        };

        // Validate that the decoded HRP matches that corresponding to the entity byte
        if actual_hrp != expected_hrp {
            return Err(DecodeBech32AddressError::InvalidHrp);
        }

        // Validation complete, return data bytes
        Ok((entity_type, data))
    }
}
