use sbor::any::*;
use scrypto::buffer::*;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::*;

pub fn validate_data(
    slice: &[u8],
) -> Result<(ValidatedData, Vec<Bid>, Vec<Rid>), DataValidationError> {
    let value = decode_any(slice).map_err(DataValidationError::DecodeError)?;

    // TODO: We need to consider if SBOR should be Scrypto-specific or general purpose.
    // The benefits of the former is that we can integrate the custom value validation
    // logic to SBOR.
    let mut validator = CustomValueValidator::new();
    traverse_any(&value, &mut validator)?;

    Ok((
        ValidatedData {
            raw: slice.to_vec(),
            value,
        },
        validator.bucket_ids,
        validator.bucket_ref_ids,
    ))
}

pub struct CustomValueValidator {
    pub bucket_ids: Vec<Bid>,
    pub bucket_ref_ids: Vec<Rid>,
}

impl CustomValueValidator {
    pub fn new() -> Self {
        Self {
            bucket_ids: Vec::new(),
            bucket_ref_ids: Vec::new(),
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
                self.bucket_ids
                    .push(Bid::try_from(data).map_err(DataValidationError::InvalidBid)?);
            }
            SCRYPTO_TYPE_RID => {
                self.bucket_ref_ids
                    .push(Rid::try_from(data).map_err(DataValidationError::InvalidRid)?);
            }
            SCRYPTO_TYPE_MID => {
                Mid::try_from(data).map_err(DataValidationError::InvalidMid)?;
            }
            SCRYPTO_TYPE_VID => {
                Vid::try_from(data).map_err(DataValidationError::InvalidVid)?;
            }
            _ => {
                return Err(DataValidationError::InvalidTypeId(kind));
            }
        }
        Ok(())
    }
}
