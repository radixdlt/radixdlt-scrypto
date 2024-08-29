use crate::internal_prelude::*;

// NOTE: Unlike LedgerTransaction, there isn't a distinct concept of a UserTransaction payload
//       so we only have `PreparedUserTransaction` and `ValidatedUserTransaction` which are
//       just sum types of the two different encoded payloads.

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreparedUserTransaction {
    V1(PreparedNotarizedTransactionV1),
    V2(PreparedNotarizedTransactionV2),
}

impl TransactionPayloadPreparable for PreparedUserTransaction {
    type Raw = RawNotarizedTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let offset = decoder.get_offset();
        let slice = decoder.get_input_slice();
        let discriminator_byte = slice.get(offset + 1).ok_or(PrepareError::Other(
            "Could not read transaction payload discriminator byte".to_string(),
        ))?;

        // Can't use a match with constants
        let prepared = if *discriminator_byte == TransactionDiscriminator::V1Notarized as u8 {
            PreparedUserTransaction::V1(PreparedNotarizedTransactionV1::prepare_for_payload(
                decoder,
            )?)
        } else if *discriminator_byte == TransactionDiscriminator::V2Notarized as u8 {
            PreparedUserTransaction::V2(PreparedNotarizedTransactionV2::prepare_for_payload(
                decoder,
            )?)
        } else {
            return Err(PrepareError::Other(format!(
                "Unknown transaction payload discriminator byte: {discriminator_byte}"
            )));
        };
        Ok(prepared)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValidatedUserTransaction {
    V1(ValidatedNotarizedTransactionV1),
    V2(ValidatedNotarizedTransactionV2),
}
