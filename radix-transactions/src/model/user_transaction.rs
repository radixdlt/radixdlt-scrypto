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

    pub fn get_blobs<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Hash, &'a Vec<u8>)> + 'a> {
        match self {
            Self::V1(m) => Box::new(m.get_blobs()),
            Self::V2(m) => Box::new(m.get_blobs()),
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

    pub fn get_blobs<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Hash, &'a Vec<u8>)> + 'a> {
        match self {
            Self::V2(m) => Box::new(m.get_blobs()),
        }
    }
}

const V1_DISCRIMINATOR: u8 = TransactionDiscriminator::V1Notarized as u8;
const V2_DISCRIMINATOR: u8 = TransactionDiscriminator::V2Notarized as u8;

/// This can be used like [`AnyTransaction`], but just for notarized transactions.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub enum UserTransaction {
    #[sbor(discriminator(V1_DISCRIMINATOR))]
    V1(#[sbor(flatten)] NotarizedTransactionV1),
    #[sbor(discriminator(V2_DISCRIMINATOR))]
    V2(#[sbor(flatten)] NotarizedTransactionV2),
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

impl From<UserTransaction> for LedgerTransaction {
    fn from(value: UserTransaction) -> Self {
        match value {
            UserTransaction::V1(tx) => LedgerTransaction::UserV1(Box::new(tx)),
            UserTransaction::V2(tx) => LedgerTransaction::UserV2(Box::new(tx)),
        }
    }
}

impl UserTransaction {
    pub fn from_raw(raw: &RawNotarizedTransaction) -> Result<Self, DecodeError> {
        manifest_decode(raw.as_slice())
    }

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
        let executable = self.prepare_and_validate(validator)?.create_executable();
        Ok(executable)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreparedUserTransaction {
    V1(PreparedNotarizedTransactionV1),
    V2(PreparedNotarizedTransactionV2),
}

impl PreparedUserTransaction {
    pub fn end_epoch_exclusive(&self) -> Epoch {
        match self {
            PreparedUserTransaction::V1(t) => t.end_epoch_exclusive(),
            PreparedUserTransaction::V2(t) => t.end_epoch_exclusive(),
        }
    }

    pub fn hashes(&self) -> UserTransactionHashes {
        match self {
            PreparedUserTransaction::V1(t) => t.hashes(),
            PreparedUserTransaction::V2(t) => t.hashes(),
        }
    }

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

impl HasNonRootSubintentHashes for PreparedUserTransaction {
    fn non_root_subintent_hashes(&self) -> Vec<SubintentHash> {
        match self {
            Self::V1(_) => Default::default(),
            Self::V2(t) => t.non_root_subintent_hashes(),
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
        let discriminator_byte = slice
            .get(offset + 1)
            .ok_or(PrepareError::UnexpectedTransactionDiscriminator { actual: None })?;

        let prepared = match TransactionDiscriminator::from_repr(*discriminator_byte) {
            Some(TransactionDiscriminator::V1Notarized) => PreparedUserTransaction::V1(
                PreparedNotarizedTransactionV1::prepare_from_transaction_enum(decoder)?,
            ),
            Some(TransactionDiscriminator::V2Notarized) => PreparedUserTransaction::V2(
                PreparedNotarizedTransactionV2::prepare_from_transaction_enum(decoder)?,
            ),
            _ => {
                return Err(PrepareError::UnexpectedTransactionDiscriminator {
                    actual: Some(*discriminator_byte),
                })
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

impl HasNonRootSubintentHashes for ValidatedUserTransaction {
    fn non_root_subintent_hashes(&self) -> Vec<SubintentHash> {
        match self {
            Self::V1(_) => Default::default(),
            Self::V2(t) => t.non_root_subintent_hashes(),
        }
    }
}

impl IntoExecutable for ValidatedUserTransaction {
    type Error = core::convert::Infallible;

    fn into_executable(
        self,
        _validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        Ok(self.create_executable())
    }
}

impl ValidatedUserTransaction {
    pub fn end_epoch_exclusive(&self) -> Epoch {
        match self {
            ValidatedUserTransaction::V1(t) => t.prepared.end_epoch_exclusive(),
            ValidatedUserTransaction::V2(t) => {
                t.overall_validity_range.epoch_range.end_epoch_exclusive
            }
        }
    }

    pub fn create_executable(self) -> ExecutableTransaction {
        match self {
            Self::V1(t) => t.create_executable(),
            Self::V2(t) => t.create_executable(),
        }
    }

    pub fn hashes(&self) -> UserTransactionHashes {
        match self {
            Self::V1(t) => t.hashes(),
            Self::V2(t) => t.hashes(),
        }
    }
}

pub type UserTransactionHashes = UserTransactionHashesV2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub struct UserTransactionHashesV1 {
    pub transaction_intent_hash: TransactionIntentHash,
    pub signed_transaction_intent_hash: SignedTransactionIntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct UserTransactionHashesV2 {
    pub transaction_intent_hash: TransactionIntentHash,
    /// ## Validity Note
    /// Preparable but invalid transactions may contain non-root subintents with duplicate [`SubintentHash`]es.
    /// Therefore we return a `Vec` instead of an `IndexSet` here.
    pub non_root_subintent_hashes: Vec<SubintentHash>,
    pub signed_transaction_intent_hash: SignedTransactionIntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
}

impl From<UserTransactionHashesV1> for UserTransactionHashesV2 {
    fn from(value: UserTransactionHashesV1) -> Self {
        let UserTransactionHashesV1 {
            transaction_intent_hash,
            signed_transaction_intent_hash,
            notarized_transaction_hash,
        } = value;
        UserTransactionHashesV2 {
            transaction_intent_hash,
            non_root_subintent_hashes: vec![],
            signed_transaction_intent_hash,
            notarized_transaction_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notarized_transaction_v1_can_be_decoded_as_user_transaction() {
        let network = NetworkDefinition::simulator();

        let notary_private_key = Ed25519PrivateKey::from_u64(3).unwrap();

        let header = TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::of(1),
            end_epoch_exclusive: Epoch::of(5),
            nonce: 0,
            notary_public_key: notary_private_key.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 0,
        };

        let notarized = TransactionBuilder::new()
            .header(header)
            .manifest(ManifestBuilder::new_v1().build())
            .notarize(&notary_private_key)
            .build();

        let raw = notarized.to_raw().unwrap();

        let user_transaction = raw.into_typed().unwrap();

        let UserTransaction::V1(decoded_notarized) = user_transaction else {
            panic!("Was not v1");
        };

        assert_eq!(notarized, decoded_notarized);
    }

    #[test]
    fn notarized_transaction_v2_can_be_decoded_as_user_transaction() {
        let network = NetworkDefinition::simulator();

        let notary_private_key = Ed25519PrivateKey::from_u64(3).unwrap();

        let header = TransactionHeaderV2 {
            notary_public_key: notary_private_key.public_key().into(),
            notary_is_signatory: false,
            tip_basis_points: 51,
        };

        let intent_header = IntentHeaderV2 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::of(1),
            end_epoch_exclusive: Epoch::of(5),
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
            intent_discriminator: 21,
        };

        let notarized = TransactionV2Builder::new()
            .transaction_header(header)
            .intent_header(intent_header)
            .manifest_builder(|builder| builder)
            .notarize(&notary_private_key)
            .build_minimal();

        let raw = notarized.to_raw().unwrap();

        let user_transaction = raw.into_typed().unwrap();

        let UserTransaction::V2(decoded_notarized) = user_transaction else {
            panic!("Was not v2");
        };

        assert_eq!(notarized, decoded_notarized);
    }
}
