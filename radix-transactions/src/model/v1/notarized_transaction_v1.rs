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

    pub fn extract_manifests_with_names(
        &self,
        names: TransactionObjectNames,
    ) -> (UserTransactionManifest, Vec<UserSubintentManifest>) {
        let mut transaction_manifest =
            TransactionManifestV1::from_intent(&self.signed_intent.intent);
        transaction_manifest.set_names_if_known(names.root_intent);
        let subintent_manifests = vec![];
        (transaction_manifest.into(), subintent_manifests)
    }
}

impl ResolveAsRawNotarizedTransaction for NotarizedTransactionV1 {
    type Intermediate = RawNotarizedTransaction;

    fn resolve_raw_notarized_transaction(self) -> Self::Intermediate {
        self.to_raw().expect("Transaction should be encodable")
    }
}

impl<'a> ResolveAsRawNotarizedTransaction for &'a NotarizedTransactionV1 {
    type Intermediate = RawNotarizedTransaction;

    fn resolve_raw_notarized_transaction(self) -> Self::Intermediate {
        self.to_raw().expect("Transaction should be encodable")
    }
}

impl IntoExecutable for NotarizedTransactionV1 {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self.prepare_and_validate(validator)?.create_executable();
        Ok(executable)
    }
}

define_transaction_payload!(
    NotarizedTransactionV1,
    RawNotarizedTransaction,
    PreparedNotarizedTransactionV1 {
        signed_intent: PreparedSignedIntentV1,
        notary_signature: PreparedNotarySignatureV1,
    },
    TransactionDiscriminator::V1Notarized,
);

impl PreparedNotarizedTransactionV1 {
    #[allow(deprecated)]
    pub fn end_epoch_exclusive(&self) -> Epoch {
        self.signed_intent.intent.header.inner.end_epoch_exclusive
    }

    pub fn hashes(&self) -> UserTransactionHashes {
        UserTransactionHashes {
            transaction_intent_hash: self.transaction_intent_hash(),
            signed_transaction_intent_hash: self.signed_transaction_intent_hash(),
            notarized_transaction_hash: self.notarized_transaction_hash(),
            non_root_subintent_hashes: Default::default(),
        }
    }

    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
        validator.validate_notarized_v1(self)
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
