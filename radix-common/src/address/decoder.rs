use super::hrpset::HrpSet;
use crate::address::AddressBech32DecodeError;
use crate::network::NetworkDefinition;
use crate::types::EntityType;
use bech32::{self, FromBase32, Variant};
use sbor::rust::prelude::*;

/// Represents a decoder which understands how to decode Scrypto addresses in Bech32.
pub struct AddressBech32Decoder {
    pub hrp_set: HrpSet,
}

impl AddressBech32Decoder {
    pub fn for_simulator() -> Self {
        Self::new(&NetworkDefinition::simulator())
    }

    /// Instantiates a new AddressBech32Decoder with the HRP corresponding to the passed network.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            hrp_set: network.into(),
        }
    }

    pub fn validate_and_decode_ignore_hrp(
        address: &str,
    ) -> Result<(String, EntityType, Vec<u8>), AddressBech32DecodeError> {
        // Decode the address string
        let (hrp, data, variant) = bech32::decode(address)
            .map_err(|err| AddressBech32DecodeError::Bech32mDecodingError(err))?;

        // Validate the Bech32 variant to ensure that is is Bech32m
        match variant {
            Variant::Bech32m => {}
            _ => return Err(AddressBech32DecodeError::InvalidVariant(variant)),
        };

        // Convert the data to u8 from u5.
        let data = Vec::<u8>::from_base32(&data)
            .map_err(|err| AddressBech32DecodeError::Bech32mDecodingError(err))?;

        // Obtain the HRP based on the entity byte in the data
        let entity_type = if let Some(entity_type_id) = data.get(0) {
            EntityType::from_repr(*entity_type_id).ok_or(
                AddressBech32DecodeError::InvalidEntityTypeId(*entity_type_id),
            )?
        } else {
            return Err(AddressBech32DecodeError::MissingEntityTypeByte);
        };

        // Validation complete, return data bytes
        Ok((hrp, entity_type, data))
    }

    /// Low level method which performs the Bech32 validation and decoding of the data.
    pub fn validate_and_decode(
        &self,
        address: &str,
    ) -> Result<(EntityType, Vec<u8>), AddressBech32DecodeError> {
        let (actual_hrp, entity_type, data) = Self::validate_and_decode_ignore_hrp(address)?;
        let expected_hrp = self.hrp_set.get_entity_hrp(&entity_type);

        // Validate that the decoded HRP matches that corresponding to the entity byte
        if actual_hrp != expected_hrp {
            return Err(AddressBech32DecodeError::InvalidHrp);
        }

        // Validation complete, return data bytes
        Ok((entity_type, data))
    }
}
