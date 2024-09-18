use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor_assert(
    fixed("FILE:notarized_transaction_v2_schema.txt"),
    settings(allow_name_changes)
)]
pub struct NotarizedTransactionV2 {
    pub signed_intent: SignedTransactionIntentV2,
    pub notary_signature: NotarySignatureV2,
}

impl NotarizedTransactionV2 {
    pub fn prepare_and_validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        self.prepare(validator.preparation_settings())?
            .validate(validator)
    }

    pub fn extract_manifest(&self) -> TransactionManifestV2 {
        TransactionManifestV2::from_intent_core(&self.signed_intent.root_intent.root_intent_core)
    }

    pub fn extract_manifests_with_names(
        &self,
        names: TransactionObjectNames,
    ) -> (UserTransactionManifest, Vec<UserSubintentManifest>) {
        let mut transaction_manifest = TransactionManifestV2::from_intent_core(
            &self.signed_intent.root_intent.root_intent_core,
        );
        transaction_manifest.set_names_if_known(names.root_intent);
        let subintents = &self.signed_intent.root_intent.subintents.0;
        if subintents.len() != names.subintents.len() {
            panic!(
                "The transaction object names have names for {} subintents but the transaction has {} subintents",
                names.subintents.len(),
                subintents.len(),
            )
        }
        let subintent_manifests = self
            .signed_intent
            .root_intent
            .subintents
            .0
            .iter()
            .zip(names.subintents.into_iter())
            .map(|(subintent, names)| {
                let mut manifest = SubintentManifestV2::from_intent_core(&subintent.intent_core);
                manifest.set_names_if_known(names);
                manifest.into()
            })
            .collect();
        (transaction_manifest.into(), subintent_manifests)
    }
}

define_transaction_payload!(
    NotarizedTransactionV2,
    RawNotarizedTransaction,
    PreparedNotarizedTransactionV2 {
        signed_intent: PreparedSignedTransactionIntentV2,
        notary_signature: PreparedNotarySignatureV2,
    },
    TransactionDiscriminator::V2Notarized,
);

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct NotarySignatureV2(pub SignatureV1);

#[allow(deprecated)]
pub type PreparedNotarySignatureV2 = SummarizedRawValueBody<NotarySignatureV2>;

impl PreparedNotarizedTransactionV2 {
    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        validator.validate_notarized_v2(self)
    }
}

impl IntoExecutable for NotarizedTransactionV2 {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self.prepare_and_validate(validator)?.create_executable();
        Ok(executable)
    }
}

impl HasTransactionIntentHash for PreparedNotarizedTransactionV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.signed_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedNotarizedTransactionV2 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        self.signed_intent.signed_transaction_intent_hash()
    }
}

impl HasNotarizedTransactionHash for PreparedNotarizedTransactionV2 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from_hash(self.summary.hash)
    }
}
