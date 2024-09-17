use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UserTransactionManifest {
    V1(TransactionManifestV1),
    V2(TransactionManifestV2),
}

impl From<TransactionManifestV1> for UserTransactionManifest {
    fn from(value: TransactionManifestV1) -> Self {
        Self::V1(value)
    }
}

impl From<TransactionManifestV2> for UserTransactionManifest {
    fn from(value: TransactionManifestV2) -> Self {
        Self::V2(value)
    }
}

impl UserTransactionManifest {
    pub fn set_names(&mut self, names: KnownManifestObjectNames) {
        match self {
            Self::V1(m) => m.set_names(names),
            Self::V2(m) => m.set_names(names),
        }
    }

    pub fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        match self {
            Self::V1(m) => m.get_blobs(),
            Self::V2(m) => m.get_blobs(),
        }
    }
}

pub trait UserTransactionPayload:
    Into<UserTransaction> + TransactionPayload<Raw = RawNotarizedTransaction>
{
}

impl<T: Into<UserTransaction> + TransactionPayload<Raw = RawNotarizedTransaction>>
    UserTransactionPayload for T
{
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UserSubintentManifest {
    V2(SubintentManifestV2),
}

impl From<SubintentManifestV2> for UserSubintentManifest {
    fn from(value: SubintentManifestV2) -> Self {
        Self::V2(value)
    }
}

impl UserSubintentManifest {
    pub fn set_names(&mut self, names: KnownManifestObjectNames) {
        match self {
            Self::V2(m) => m.set_names(names),
        }
    }

    pub fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        match self {
            Self::V2(m) => m.get_blobs(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UserTransaction {
    V1(NotarizedTransactionV1),
    V2(NotarizedTransactionV2),
}

impl From<NotarizedTransactionV1> for UserTransaction {
    fn from(value: NotarizedTransactionV1) -> Self {
        Self::V1(value)
    }
}

impl From<NotarizedTransactionV2> for UserTransaction {
    fn from(value: NotarizedTransactionV2) -> Self {
        Self::V2(value)
    }
}

impl UserTransaction {
    pub fn prepare(
        self,
        settings: &PreparationSettings,
    ) -> Result<PreparedUserTransaction, PrepareError> {
        Ok(match self {
            Self::V1(t) => PreparedUserTransaction::V1(t.prepare(settings)?),
            Self::V2(t) => PreparedUserTransaction::V2(t.prepare(settings)?),
        })
    }

    pub fn extract_manifests_with_names(
        &self,
        names: TransactionObjectNames,
    ) -> (UserTransactionManifest, Vec<UserSubintentManifest>) {
        match self {
            UserTransaction::V1(t) => t.extract_manifests_with_names(names).into(),
            UserTransaction::V2(t) => t.extract_manifests_with_names(names).into(),
        }
    }
}

impl UserTransaction {
    pub fn prepare_and_validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedUserTransaction, TransactionValidationError> {
        Ok(match self {
            UserTransaction::V1(t) => {
                ValidatedUserTransaction::V1(t.prepare_and_validate(validator)?)
            }
            UserTransaction::V2(t) => {
                ValidatedUserTransaction::V2(t.prepare_and_validate(validator)?)
            }
        })
    }
}

impl IntoExecutable for UserTransaction {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self.prepare_and_validate(validator)?.get_executable();
        Ok(executable)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreparedUserTransaction {
    V1(PreparedNotarizedTransactionV1),
    V2(PreparedNotarizedTransactionV2),
}

impl PreparedUserTransaction {
    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedUserTransaction, TransactionValidationError> {
        Ok(match self {
            PreparedUserTransaction::V1(t) => ValidatedUserTransaction::V1(t.validate(validator)?),
            PreparedUserTransaction::V2(t) => ValidatedUserTransaction::V2(t.validate(validator)?),
        })
    }
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

impl PreparedTransaction for PreparedUserTransaction {
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
