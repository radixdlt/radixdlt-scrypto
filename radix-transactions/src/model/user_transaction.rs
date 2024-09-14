use crate::internal_prelude::*;

// NOTE: Unlike LedgerTransaction, there isn't a distinct concept of a UserTransaction payload
//       so we only have `PreparedUserTransaction` and `ValidatedUserTransaction` which are
//       just sum types of the two different encoded payloads.

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreparedUserTransaction {
    V1(PreparedNotarizedTransactionV1),
    V2(PreparedNotarizedTransactionV2),
}

impl HasTransactionIntentHash for PreparedUserTransaction {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        match self {
            Self::V1(t) => t.transaction_intent_hash(),
            Self::V2(t) => t.transaction_intent_hash(),
        }
    }
}

impl HasSignedTransactionIntentHash for PreparedUserTransaction {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        match self {
            Self::V1(t) => t.signed_transaction_intent_hash(),
            Self::V2(t) => t.signed_transaction_intent_hash(),
        }
    }
}

impl HasNotarizedTransactionHash for PreparedUserTransaction {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        match self {
            Self::V1(t) => t.notarized_transaction_hash(),
            Self::V2(t) => t.notarized_transaction_hash(),
        }
    }
}

impl HasSummary for PreparedUserTransaction {
    fn get_summary(&self) -> &Summary {
        match self {
            Self::V1(t) => t.get_summary(),
            Self::V2(t) => t.get_summary(),
        }
    }

    fn summary_mut(&mut self) -> &mut Summary {
        match self {
            Self::V1(t) => t.summary_mut(),
            Self::V2(t) => t.summary_mut(),
        }
    }
}

impl TransactionPayloadPreparable for PreparedUserTransaction {
    type Raw = RawNotarizedTransaction;

    fn prepare_from_transaction_enum(
        decoder: &mut TransactionDecoder,
    ) -> Result<Self, PrepareError> {
        let offset = decoder.get_offset();
        let slice = decoder.get_input_slice();
        let discriminator_byte = slice.get(offset + 1).ok_or(PrepareError::Other(
            "Could not read transaction payload discriminator byte".to_string(),
        ))?;

        let prepared = match TransactionDiscriminator::from_repr(*discriminator_byte) {
            Some(TransactionDiscriminator::V1Notarized) => PreparedUserTransaction::V1(
                PreparedNotarizedTransactionV1::prepare_from_transaction_enum(decoder)?,
            ),
            Some(TransactionDiscriminator::V2Notarized) => PreparedUserTransaction::V2(
                PreparedNotarizedTransactionV2::prepare_from_transaction_enum(decoder)?,
            ),
            _ => {
                return Err(PrepareError::Other(format!(
                    "Unknown transaction payload discriminator byte: {discriminator_byte}"
                )))
            }
        };

        Ok(prepared)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValidatedUserTransaction {
    V1(ValidatedNotarizedTransactionV1),
    V2(ValidatedNotarizedTransactionV2),
}

impl HasTransactionIntentHash for ValidatedUserTransaction {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        match self {
            Self::V1(t) => t.transaction_intent_hash(),
            Self::V2(t) => t.transaction_intent_hash(),
        }
    }
}

impl HasSignedTransactionIntentHash for ValidatedUserTransaction {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        match self {
            Self::V1(t) => t.signed_transaction_intent_hash(),
            Self::V2(t) => t.signed_transaction_intent_hash(),
        }
    }
}

impl HasNotarizedTransactionHash for ValidatedUserTransaction {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        match self {
            Self::V1(t) => t.notarized_transaction_hash(),
            Self::V2(t) => t.notarized_transaction_hash(),
        }
    }
}

impl ValidatedUserTransaction {
    pub fn get_executable(&self) -> ExecutableTransaction {
        match self {
            Self::V1(t) => t.get_executable(),
            Self::V2(t) => t.get_executable(),
        }
    }
}
