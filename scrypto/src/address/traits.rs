use super::ParseAddressError;
use crate::core::Network;
use bech32::{self, FromBase32, ToBase32, Variant};

use super::{entity::EntityType, hrpset::get_network_hrp_set};

pub trait Bech32Addressable
where
    Self: for<'a> TryFrom<&'a [u8], Error = ParseAddressError> + Sized,
{
    // Returns an array slice of the allowed entity bytes for this type
    fn allowed_entity_types() -> &'static [EntityType];
    
    /// Returns the data to be Bech32 encoded.
    fn data(&self) -> &[u8];

    /// Returns the encoded Bech32 encoded data.
    fn to_bech32_string(&self, network: &Network) -> Result<String, ParseAddressError> {
        let hrp = match self.data().get(0) {
            Some(entity_type_id) => {
                let entity_type = EntityType::try_from(*entity_type_id)
                    .map_err(|_| ParseAddressError::InvalidEntityTypeId(*entity_type_id))?;
                get_network_hrp_set(network).get_entity_hrp(&entity_type)
            }
            None => return Err(ParseAddressError::DataSectionTooShort),
        };

        Ok(
            bech32::encode(hrp, self.data().to_base32(), Variant::Bech32m)
                .map_err(|err| ParseAddressError::EncodingError(err))?,
        )
    }

    /// Returns an object instantiated from the Bec32 string.
    fn from_bech32_string(address: &str, network: &Network) -> Result<Self, ParseAddressError> {
        let (hrp, data, variant) =
            bech32::decode(address).map_err(|err| ParseAddressError::DecodingError(err))?;

        // Validate Variant
        match variant {
            Variant::Bech32m => {}
            _ => return Err(ParseAddressError::InvalidVariant(variant)),
        };

        // Convert the data to u8 from u5.
        let data =
            Vec::<u8>::from_base32(&data).map_err(|err| ParseAddressError::DecodingError(err))?;

        // Validating the actual HRP with the expected HRP
        match data.get(0) {
            Some(entity_type_id) => {
                let entity_type = EntityType::try_from(*entity_type_id)
                    .map_err(|_| ParseAddressError::InvalidEntityTypeId(*entity_type_id))?;
                
                if !Self::allowed_entity_types().contains(&entity_type) {
                    return Err(ParseAddressError::InvalidEntityTypeId(*entity_type_id))
                }
                
                let expected_hrp: &'static str =
                    get_network_hrp_set(network).get_entity_hrp(&entity_type);

                if expected_hrp == hrp {
                    Ok(Self::try_from(&data).map_err(|_| ParseAddressError::TryFromError)?)
                } else {
                    Err(ParseAddressError::InvalidHrp)
                }
            }
            None => Err(ParseAddressError::DataSectionTooShort),
        }
    }
}
