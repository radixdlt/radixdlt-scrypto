use super::hrpset::HrpSet;
use crate::address::DecodeBech32AddressError;
use crate::network::NetworkDefinition;
use crate::types::EntityType;
use bech32::{self, FromBase32, Variant};
use sbor::rust::prelude::*;

/// Represents a decoder which understands how to decode Scrypto addresses in Bech32.
pub struct Bech32Decoder {
    pub hrp_set: Option<HrpSet>,
}

impl Bech32Decoder {
    pub fn for_simulator() -> Self {
        Self::new(&NetworkDefinition::simulator())
    }

    /// Instantiates a new Bech32Decoder with the HRP corresponding to the passed network.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            hrp_set: Some(network.into()),
        }
    }

    pub fn without_network() -> Self {
        Self { hrp_set: None }
    }

    pub fn validate_and_decode_ignore_hrp(
        &self,
        address: &str,
    ) -> Result<(String, EntityType, Vec<u8>), DecodeBech32AddressError> {
        // Decode the address string
        let (hrp, data, variant) =
            bech32::decode(address).map_err(DecodeBech32AddressError::Bech32mDecodingError)?;

        // Validate the Bech32 variant to ensure that is is Bech32m
        match variant {
            Variant::Bech32m => {}
            _ => return Err(DecodeBech32AddressError::InvalidVariant(variant)),
        };

        // Convert the data to u8 from u5.
        let data = Vec::<u8>::from_base32(&data)
            .map_err(DecodeBech32AddressError::Bech32mDecodingError)?;

        // Obtain the HRP based on the entity byte in the data
        let entity_type = if let Some(entity_type_id) = data.first() {
            EntityType::from_repr(*entity_type_id).ok_or(
                DecodeBech32AddressError::InvalidEntityTypeId(*entity_type_id),
            )?
        } else {
            return Err(DecodeBech32AddressError::MissingEntityTypeByte);
        };

        // Validation complete, return data bytes
        Ok((hrp, entity_type, data))
    }

    /// Low level method which performs the Bech32 validation and decoding of the data.
    pub fn validate_and_decode(
        &self,
        address: &str,
    ) -> Result<(EntityType, Vec<u8>), DecodeBech32AddressError> {
        let (actual_hrp, entity_type, data) = self.validate_and_decode_ignore_hrp(address)?;

        // Validate that the decoded HRP matches that corresponding to the entity byte
        if !self.hrp_set.as_ref().map_or(true, |hrp_set| {
            hrp_set.get_entity_hrp(&entity_type) == actual_hrp
        }) {
            return Err(DecodeBech32AddressError::InvalidHrp);
        }

        // Validation complete, return data bytes
        Ok((entity_type, data))
    }
}
