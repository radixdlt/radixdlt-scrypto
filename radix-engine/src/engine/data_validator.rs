use sbor::any::*;
use scrypto::buffer::*;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::*;

pub fn validate_data(slice: &[u8]) -> Result<ValidatedData, DataValidationError> {
    let value = decode_any(slice).map_err(DataValidationError::DecodeError)?;

    // TODO: We need to consider if SBOR should be Scrypto-specific or general purpose.
    // The benefits of the former is that we can integrate the custom value validation
    // logic to SBOR.
    let mut validator = CustomValueValidator::new();
    traverse_any(&value, &mut validator)?;

    Ok(ValidatedData {
        raw: slice.to_vec(),
        dom: value,
        buckets: validator.buckets,
        bucket_refs: validator.bucket_refs,
        vaults: validator.vaults,
        lazy_maps: validator.lazy_maps,
    })
}

pub struct CustomValueValidator {
    pub buckets: Vec<Bid>,
    pub bucket_refs: Vec<Rid>,
    pub vaults: Vec<Vid>,
    pub lazy_maps: Vec<Mid>,
}

impl CustomValueValidator {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            bucket_refs: Vec::new(),
            vaults: Vec::new(),
            lazy_maps: Vec::new(),
        }
    }
}

impl CustomValueVisitor for CustomValueValidator {
    type Err = DataValidationError;

    fn visit(&mut self, kind: u8, data: &[u8]) -> Result<(), Self::Err> {
        match kind {
            SCRYPTO_TYPE_DECIMAL => {
                Decimal::try_from(data).map_err(DataValidationError::InvalidDecimal)?;
            }
            SCRYPTO_TYPE_BIG_DECIMAL => {
                BigDecimal::try_from(data).map_err(DataValidationError::InvalidBigDecimal)?;
            }
            SCRYPTO_TYPE_ADDRESS => {
                Address::try_from(data).map_err(DataValidationError::InvalidAddress)?;
            }
            SCRYPTO_TYPE_H256 => {
                H256::try_from(data).map_err(DataValidationError::InvalidH256)?;
            }
            SCRYPTO_TYPE_BID => {
                self.buckets
                    .push(Bid::try_from(data).map_err(DataValidationError::InvalidBid)?);
            }
            SCRYPTO_TYPE_RID => {
                self.bucket_refs
                    .push(Rid::try_from(data).map_err(DataValidationError::InvalidRid)?);
            }
            SCRYPTO_TYPE_MID => {
                self.lazy_maps
                    .push(Mid::try_from(data).map_err(DataValidationError::InvalidMid)?);
            }
            SCRYPTO_TYPE_VID => {
                self.vaults
                    .push(Vid::try_from(data).map_err(DataValidationError::InvalidVid)?);
            }
            SCRYPTO_TYPE_NON_FUNGIBLE_KEY => {
                NonFungibleKey::try_from(data)
                    .map_err(DataValidationError::InvalidNonFungibleKey)?;
            }
            _ => {
                return Err(DataValidationError::InvalidTypeId(kind));
            }
        }
        Ok(())
    }
}
