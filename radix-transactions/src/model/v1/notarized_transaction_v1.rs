use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor_assert(
    fixed("FILE:notarized_transaction_v1_schema.txt"),
    settings(allow_name_changes)
)]
pub struct NotarizedTransactionV1 {
    pub signed_intent: SignedIntentV1,
    pub notary_signature: NotarySignatureV1,
}

impl NotarizedTransactionV1 {
    pub fn prepare_and_validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
        self.prepare(validator.preparation_settings())?
            .validate(validator)
    }
}

impl IntoExecutable for NotarizedTransactionV1 {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self.prepare_and_validate(validator)?.get_executable();
        Ok(executable)
    }
}

impl TransactionPayload for NotarizedTransactionV1 {
    type Prepared = PreparedNotarizedTransactionV1;
    type Raw = RawNotarizedTransaction;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedNotarizedTransactionV1 {
    pub signed_intent: PreparedSignedIntentV1,
    pub notary_signature: PreparedNotarySignatureV1,
    pub summary: Summary,
}

impl PreparedNotarizedTransactionV1 {
    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
        validator.validate_notarized_v1(self)
    }
}

impl_has_summary!(PreparedNotarizedTransactionV1);

impl TransactionPreparableFromValue for PreparedNotarizedTransactionV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((signed_intent, notary_signature), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V1Notarized,
            )?;
        Ok(Self {
            signed_intent,
            notary_signature,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedNotarizedTransactionV1 {
    type Raw = RawNotarizedTransaction;

    fn prepare_from_transaction_enum(
        decoder: &mut TransactionDecoder,
    ) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((signed_intent, notary_signature), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V1Notarized,
            )?;
        Ok(Self {
            signed_intent,
            notary_signature,
            summary,
        })
    }
}

impl HasTransactionIntentHash for PreparedNotarizedTransactionV1 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.signed_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedNotarizedTransactionV1 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        self.signed_intent.signed_transaction_intent_hash()
    }
}

impl HasNotarizedTransactionHash for PreparedNotarizedTransactionV1 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from_hash(self.summary.hash)
    }
}
