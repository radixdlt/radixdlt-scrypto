use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum LedgerTransaction {
    #[sbor(discriminator(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR))]
    Genesis(Box<GenesisTransaction>),
    #[sbor(discriminator(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV1(Box<NotarizedTransactionV1>),
    #[sbor(discriminator(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    RoundUpdateV1(Box<RoundUpdateTransactionV1>),
    #[sbor(discriminator(FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    FlashV1(Box<FlashTransactionV1>),
    #[sbor(discriminator(USER_V2_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV2(Box<NotarizedTransactionV2>),
}

const GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 0;
const USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 1;
const ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 2;
const FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 3;
const USER_V2_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 4;

enum LedgerTransactionKind {
    Genesis,
    User,
    Validator,
    ProtocolUpdate,
}

impl LedgerTransactionKind {
    fn discriminator_for_hash(&self) -> u8 {
        match self {
            LedgerTransactionKind::Genesis => GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR,
            LedgerTransactionKind::User => USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR,
            LedgerTransactionKind::Validator => ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR,
            LedgerTransactionKind::ProtocolUpdate => FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR,
        }
    }
}

define_raw_transaction_payload!(
    RawLedgerTransaction,
    TransactionPayloadKind::LedgerTransaction
);

impl RawLedgerTransaction {
    pub fn prepare(
        &self,
        settings: &PreparationSettings,
    ) -> Result<PreparedLedgerTransaction, PrepareError> {
        PreparedLedgerTransaction::prepare(self, settings)
    }

    pub fn validate(
        &self,
        validator: &TransactionValidator,
        accepted_kind: AcceptedLedgerTransactionKind,
    ) -> Result<ValidatedLedgerTransaction, TransactionValidationError> {
        let prepared = PreparedLedgerTransaction::prepare(self, validator.preparation_settings())?;
        prepared.validate(validator, accepted_kind)
    }
}

impl TransactionPayload for LedgerTransaction {
    type Prepared = PreparedLedgerTransaction;
    type Raw = RawLedgerTransaction;
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum GenesisTransaction {
    #[sbor(discriminator(GENESIS_TRANSACTION_FLASH_DISCRIMINATOR))]
    Flash,
    #[sbor(discriminator(GENESIS_TRANSACTION_SYSTEM_TRANSACTION_DISCRIMINATOR))]
    Transaction(Box<SystemTransactionV1>),
}

const GENESIS_TRANSACTION_FLASH_DISCRIMINATOR: u8 = 0;
const GENESIS_TRANSACTION_SYSTEM_TRANSACTION_DISCRIMINATOR: u8 = 1;

pub struct PreparedLedgerTransaction {
    pub inner: PreparedLedgerTransactionInner,
    pub summary: Summary,
}

impl PreparedLedgerTransaction {
    pub fn into_user(self) -> Option<PreparedUserTransaction> {
        match self.inner {
            PreparedLedgerTransactionInner::User(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_user(&self) -> Option<&PreparedUserTransaction> {
        match &self.inner {
            PreparedLedgerTransactionInner::User(t) => Some(t),
            _ => None,
        }
    }

    pub fn create_identifiers(&self) -> PayloadIdentifiers {
        PayloadIdentifiers {
            ledger_transaction_hash: self.ledger_transaction_hash(),
            typed: match &self.inner {
                PreparedLedgerTransactionInner::Genesis(t) => {
                    TypedTransactionIdentifiers::Genesis {
                        system_transaction_hash: t.system_transaction_hash(),
                    }
                }
                PreparedLedgerTransactionInner::User(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.transaction_intent_hash(),
                    signed_intent_hash: t.signed_transaction_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                PreparedLedgerTransactionInner::Validator(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
                    }
                }
                PreparedLedgerTransactionInner::ProtocolUpdate(t) => {
                    TypedTransactionIdentifiers::FlashV1 {
                        flash_transaction_hash: t.flash_transaction_hash(),
                    }
                }
            },
        }
    }

    fn validate(
        self,
        validator: &TransactionValidator,
        accepted_kind: AcceptedLedgerTransactionKind,
    ) -> Result<ValidatedLedgerTransaction, TransactionValidationError> {
        let validated_inner = match self.inner {
            PreparedLedgerTransactionInner::Genesis(t) => {
                if !accepted_kind.permits_genesis() {
                    return Err(TransactionValidationError::Other(
                        "Genesis transaction not permitted at this point".to_string(),
                    ));
                }
                ValidatedLedgerTransactionInner::Genesis(t)
            }
            PreparedLedgerTransactionInner::User(t) => {
                if !accepted_kind.permits_user() {
                    return Err(TransactionValidationError::Other(
                        "User transaction not permitted at this point".to_string(),
                    ));
                }
                ValidatedLedgerTransactionInner::User(t.validate(validator)?)
            }
            PreparedLedgerTransactionInner::Validator(t) => {
                if !accepted_kind.permits_validator() {
                    return Err(TransactionValidationError::Other(
                        "Round update transaction not permitted at this point".to_string(),
                    ));
                }
                ValidatedLedgerTransactionInner::Validator(t)
            }
            PreparedLedgerTransactionInner::ProtocolUpdate(t) => {
                if !accepted_kind.permits_protocol_update() {
                    return Err(TransactionValidationError::Other(
                        "Protocol update transaction not permitted at this point".to_string(),
                    ));
                }
                ValidatedLedgerTransactionInner::ProtocolUpdate(t)
            }
        };
        Ok(ValidatedLedgerTransaction {
            inner: validated_inner,
            summary: self.summary,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AcceptedLedgerTransactionKind {
    Any,
    UserOnly,
    GenesisOnly,
    ValidatorOnly,
    ProtocolUpdateOnly,
    UserOrValidator,
}

impl AcceptedLedgerTransactionKind {
    fn permits_genesis(&self) -> bool {
        match self {
            AcceptedLedgerTransactionKind::Any => true,
            AcceptedLedgerTransactionKind::UserOnly => false,
            AcceptedLedgerTransactionKind::GenesisOnly => true,
            AcceptedLedgerTransactionKind::ValidatorOnly => false,
            AcceptedLedgerTransactionKind::ProtocolUpdateOnly => false,
            AcceptedLedgerTransactionKind::UserOrValidator => false,
        }
    }

    fn permits_user(&self) -> bool {
        match self {
            AcceptedLedgerTransactionKind::Any => true,
            AcceptedLedgerTransactionKind::UserOnly => true,
            AcceptedLedgerTransactionKind::GenesisOnly => false,
            AcceptedLedgerTransactionKind::ValidatorOnly => false,
            AcceptedLedgerTransactionKind::ProtocolUpdateOnly => false,
            AcceptedLedgerTransactionKind::UserOrValidator => true,
        }
    }

    fn permits_validator(&self) -> bool {
        match self {
            AcceptedLedgerTransactionKind::Any => true,
            AcceptedLedgerTransactionKind::UserOnly => false,
            AcceptedLedgerTransactionKind::GenesisOnly => false,
            AcceptedLedgerTransactionKind::ValidatorOnly => true,
            AcceptedLedgerTransactionKind::ProtocolUpdateOnly => false,
            AcceptedLedgerTransactionKind::UserOrValidator => true,
        }
    }

    fn permits_protocol_update(&self) -> bool {
        match self {
            AcceptedLedgerTransactionKind::Any => true,
            AcceptedLedgerTransactionKind::UserOnly => false,
            AcceptedLedgerTransactionKind::GenesisOnly => false,
            AcceptedLedgerTransactionKind::ValidatorOnly => false,
            AcceptedLedgerTransactionKind::ProtocolUpdateOnly => true,
            AcceptedLedgerTransactionKind::UserOrValidator => false,
        }
    }
}

impl_has_summary!(PreparedLedgerTransaction);

pub enum PreparedLedgerTransactionInner {
    Genesis(PreparedGenesisTransaction),
    User(PreparedUserTransaction),
    Validator(PreparedRoundUpdateTransactionV1),
    ProtocolUpdate(PreparedFlashTransactionV1),
}

impl PreparedLedgerTransactionInner {
    fn get_kind(&self) -> LedgerTransactionKind {
        match self {
            Self::Genesis(_) => LedgerTransactionKind::Genesis,
            Self::User(_) => LedgerTransactionKind::User,
            Self::Validator(_) => LedgerTransactionKind::Validator,
            Self::ProtocolUpdate(_) => LedgerTransactionKind::ProtocolUpdate,
        }
    }

    pub fn get_ledger_hash(&self) -> LedgerTransactionHash {
        LedgerTransactionHash::for_kind(self.get_kind(), &self.get_summary().hash)
    }
}

impl HasSummary for PreparedLedgerTransactionInner {
    fn get_summary(&self) -> &Summary {
        match self {
            Self::Genesis(t) => t.get_summary(),
            Self::User(t) => t.get_summary(),
            Self::Validator(t) => t.get_summary(),
            Self::ProtocolUpdate(t) => t.get_summary(),
        }
    }

    fn summary_mut(&mut self) -> &mut Summary {
        match self {
            Self::Genesis(t) => t.summary_mut(),
            Self::User(t) => t.summary_mut(),
            Self::Validator(t) => t.summary_mut(),
            Self::ProtocolUpdate(t) => t.summary_mut(),
        }
    }
}

impl TransactionPreparableFromValue for PreparedLedgerTransactionInner {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        decoder.track_stack_depth_increase()?;
        let (discriminator, length) = decoder.read_enum_header()?;
        let prepared_inner = match discriminator {
            GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let (discriminator, length) = decoder.read_enum_header()?;
                let genesis_transaction = match discriminator {
                    GENESIS_TRANSACTION_FLASH_DISCRIMINATOR => {
                        check_length(length, 0)?;
                        PreparedGenesisTransaction::Flash(Summary {
                            effective_length: 0,
                            total_bytes_hashed: 0,
                            hash: hash("Genesis Flash"),
                        })
                    }
                    GENESIS_TRANSACTION_SYSTEM_TRANSACTION_DISCRIMINATOR => {
                        check_length(length, 1)?;
                        let prepared = PreparedSystemTransactionV1::prepare_from_value(decoder)?;
                        PreparedGenesisTransaction::Transaction(prepared)
                    }
                    _ => return Err(unknown_discriminator(discriminator)),
                };
                PreparedLedgerTransactionInner::Genesis(genesis_transaction)
            }
            USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let prepared = PreparedNotarizedTransactionV1::prepare_from_value(decoder)?;
                PreparedLedgerTransactionInner::User(PreparedUserTransaction::V1(prepared))
            }
            ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let prepared = PreparedRoundUpdateTransactionV1::prepare_from_value(decoder)?;
                PreparedLedgerTransactionInner::Validator(prepared)
            }
            FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let prepared = PreparedFlashTransactionV1::prepare_from_value(decoder)?;
                PreparedLedgerTransactionInner::ProtocolUpdate(prepared)
            }
            USER_V2_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let prepared = PreparedNotarizedTransactionV2::prepare_from_value(decoder)?;
                PreparedLedgerTransactionInner::User(PreparedUserTransaction::V2(prepared))
            }
            _ => return Err(unknown_discriminator(discriminator)),
        };
        decoder.track_stack_depth_decrease()?;

        Ok(prepared_inner)
    }
}

fn check_length(actual: usize, expected: usize) -> Result<(), PrepareError> {
    if actual != expected {
        return Err(PrepareError::DecodeError(DecodeError::UnexpectedSize {
            expected,
            actual,
        }));
    }
    Ok(())
}

fn unknown_discriminator(discriminator: u8) -> PrepareError {
    PrepareError::DecodeError(DecodeError::UnknownDiscriminator(discriminator))
}

pub enum PreparedGenesisTransaction {
    Flash(Summary),
    Transaction(PreparedSystemTransactionV1),
}

impl HasSummary for PreparedGenesisTransaction {
    fn get_summary(&self) -> &Summary {
        match self {
            PreparedGenesisTransaction::Flash(summary) => summary,
            PreparedGenesisTransaction::Transaction(t) => t.get_summary(),
        }
    }

    fn summary_mut(&mut self) -> &mut Summary {
        match self {
            PreparedGenesisTransaction::Flash(summary) => summary,
            PreparedGenesisTransaction::Transaction(t) => t.summary_mut(),
        }
    }
}

impl HasSystemTransactionHash for PreparedGenesisTransaction {
    fn system_transaction_hash(&self) -> SystemTransactionHash {
        match self {
            PreparedGenesisTransaction::Flash(summary) => SystemTransactionHash(summary.hash),
            PreparedGenesisTransaction::Transaction(transaction) => {
                transaction.system_transaction_hash()
            }
        }
    }
}

impl PreparedTransaction for PreparedLedgerTransaction {
    type Raw = RawLedgerTransaction;

    fn prepare_from_transaction_enum(
        decoder: &mut TransactionDecoder,
    ) -> Result<Self, PrepareError> {
        decoder.track_stack_depth_increase()?;
        decoder.read_header(
            ExpectedTupleHeader::EnumWithValueKind {
                discriminator: TransactionDiscriminator::Ledger as u8,
            },
            1,
        )?;
        let inner = PreparedLedgerTransactionInner::prepare_from_value(decoder)?;
        decoder.track_stack_depth_decrease()?;

        let summary = Summary {
            effective_length: inner.get_summary().effective_length,
            total_bytes_hashed: inner.get_summary().total_bytes_hashed,
            hash: inner.get_ledger_hash().0,
        };
        Ok(Self { inner, summary })
    }
}

impl IntoExecutable for PreparedLedgerTransaction {
    type Error = LedgerTransactionExecutableError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        self.validate(validator, AcceptedLedgerTransactionKind::Any)?
            .into_executable(validator)
    }
}

pub struct ValidatedLedgerTransaction {
    pub inner: ValidatedLedgerTransactionInner,
    pub summary: Summary,
}

pub enum ValidatedLedgerTransactionInner {
    Genesis(PreparedGenesisTransaction),
    User(ValidatedUserTransaction),
    Validator(PreparedRoundUpdateTransactionV1),
    ProtocolUpdate(PreparedFlashTransactionV1),
}

#[derive(Debug, Clone)]
pub enum LedgerTransactionExecutableError {
    IsFlashTransaction,
    ValidationError(TransactionValidationError),
}

impl From<TransactionValidationError> for LedgerTransactionExecutableError {
    fn from(value: TransactionValidationError) -> Self {
        Self::ValidationError(value)
    }
}

impl ValidatedLedgerTransaction {
    pub fn intent_hash_if_user(&self) -> Option<TransactionIntentHash> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(_) => None,
            ValidatedLedgerTransactionInner::User(t) => Some(t.transaction_intent_hash()),
            ValidatedLedgerTransactionInner::Validator(_) => None,
            ValidatedLedgerTransactionInner::ProtocolUpdate(_) => None,
        }
    }

    /// Note - returns None if it's a flash transaction
    pub fn get_executable(
        &self,
    ) -> Result<ExecutableTransaction, LedgerTransactionExecutableError> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(genesis) => match genesis {
                PreparedGenesisTransaction::Flash(_) => {
                    Err(LedgerTransactionExecutableError::IsFlashTransaction)
                }
                PreparedGenesisTransaction::Transaction(t) => {
                    Ok(t.get_executable(btreeset!(system_execution(SystemExecution::Protocol))))
                }
            },
            ValidatedLedgerTransactionInner::User(t) => Ok(t.get_executable()),
            ValidatedLedgerTransactionInner::Validator(t) => Ok(t.get_executable()),
            ValidatedLedgerTransactionInner::ProtocolUpdate(_) => {
                Err(LedgerTransactionExecutableError::IsFlashTransaction)
            }
        }
    }

    pub fn create_identifiers(&self) -> PayloadIdentifiers {
        PayloadIdentifiers {
            ledger_transaction_hash: self.ledger_transaction_hash(),
            typed: match &self.inner {
                ValidatedLedgerTransactionInner::Genesis(t) => {
                    TypedTransactionIdentifiers::Genesis {
                        system_transaction_hash: t.system_transaction_hash(),
                    }
                }
                ValidatedLedgerTransactionInner::User(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.transaction_intent_hash(),
                    signed_intent_hash: t.signed_transaction_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                ValidatedLedgerTransactionInner::Validator(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
                    }
                }
                ValidatedLedgerTransactionInner::ProtocolUpdate(t) => {
                    TypedTransactionIdentifiers::FlashV1 {
                        flash_transaction_hash: t.flash_transaction_hash(),
                    }
                }
            },
        }
    }
}

impl IntoExecutable for ValidatedLedgerTransaction {
    type Error = LedgerTransactionExecutableError;

    fn into_executable(
        self,
        _validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        self.get_executable()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct PayloadIdentifiers {
    pub ledger_transaction_hash: LedgerTransactionHash,
    pub typed: TypedTransactionIdentifiers,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum TypedTransactionIdentifiers {
    Genesis {
        system_transaction_hash: SystemTransactionHash,
    },
    User {
        intent_hash: TransactionIntentHash,
        signed_intent_hash: SignedTransactionIntentHash,
        notarized_transaction_hash: NotarizedTransactionHash,
    },
    RoundUpdateV1 {
        round_update_hash: RoundUpdateTransactionHash,
    },
    FlashV1 {
        flash_transaction_hash: FlashTransactionHash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserTransactionIdentifiers<'a> {
    pub intent_hash: &'a TransactionIntentHash,
    pub signed_intent_hash: &'a SignedTransactionIntentHash,
    pub notarized_transaction_hash: &'a NotarizedTransactionHash,
}

impl TypedTransactionIdentifiers {
    pub fn user(&self) -> Option<UserTransactionIdentifiers> {
        match self {
            TypedTransactionIdentifiers::User {
                intent_hash,
                signed_intent_hash,
                notarized_transaction_hash,
            } => Some(UserTransactionIdentifiers {
                intent_hash,
                signed_intent_hash,
                notarized_transaction_hash,
            }),
            _ => None,
        }
    }
}

impl HasLedgerTransactionHash for ValidatedLedgerTransaction {
    fn ledger_transaction_hash(&self) -> LedgerTransactionHash {
        LedgerTransactionHash::from_hash(self.summary.hash)
    }
}

define_wrapped_hash!(LedgerTransactionHash);

impl LedgerTransactionHash {
    pub fn for_genesis(hash: &SystemTransactionHash) -> Self {
        Self::for_kind(LedgerTransactionKind::Genesis, &hash.0)
    }

    pub fn for_user(hash: &NotarizedTransactionHash) -> Self {
        Self::for_kind(LedgerTransactionKind::User, &hash.0)
    }

    pub fn for_round_update(hash: &RoundUpdateTransactionHash) -> Self {
        Self::for_kind(LedgerTransactionKind::Validator, &hash.0)
    }

    fn for_kind(kind: LedgerTransactionKind, inner: &Hash) -> Self {
        Self(
            HashAccumulator::new()
                .concat([
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::Ledger as u8,
                    kind.discriminator_for_hash(),
                ])
                .concat(inner.as_slice())
                .finalize(),
        )
    }
}

impl IsTransactionHashWithStaticHrp for LedgerTransactionHash {
    fn static_hrp(hrp_set: &HrpSet) -> &str {
        &hrp_set.ledger_transaction
    }
}

pub trait HasLedgerTransactionHash {
    fn ledger_transaction_hash(&self) -> LedgerTransactionHash;
}

impl HasLedgerTransactionHash for PreparedLedgerTransaction {
    fn ledger_transaction_hash(&self) -> LedgerTransactionHash {
        LedgerTransactionHash::from_hash(self.summary.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn v1_ledger_transaction_structure() {
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let sig_2_private_key = Ed25519PrivateKey::from_u64(2).unwrap();
        let notary_private_key = Ed25519PrivateKey::from_u64(3).unwrap();

        let notarized = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: 21,
                start_epoch_inclusive: Epoch::of(0),
                end_epoch_exclusive: Epoch::of(100),
                nonce: 0,
                notary_public_key: notary_private_key.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(ManifestBuilder::new().drop_all_proofs().build())
            .sign(&sig_1_private_key)
            .sign(&sig_2_private_key)
            .notarize(&notary_private_key)
            .build();

        let prepared_notarized = notarized
            .prepare(PreparationSettings::latest_ref())
            .expect("Notarized can be prepared");

        let ledger = LedgerTransaction::UserV1(Box::new(notarized));
        let raw_ledger_transaction = ledger.to_raw().expect("Can be encoded");
        LedgerTransaction::from_raw(&raw_ledger_transaction).expect("Can be decoded");
        let prepared_ledger_transaction = raw_ledger_transaction
            .prepare(PreparationSettings::latest_ref())
            .expect("Can be prepared");

        let expected_intent_hash = LedgerTransactionHash::from_hash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::Ledger as u8,
                    USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR,
                ]
                .as_slice(),
                prepared_notarized.notarized_transaction_hash().0.as_slice(),
            ]
            .concat(),
        ));
        assert_eq!(
            prepared_ledger_transaction.ledger_transaction_hash(),
            expected_intent_hash
        );
        assert_eq!(
            LedgerTransactionHash::for_user(&prepared_notarized.notarized_transaction_hash()),
            expected_intent_hash
        );
    }
}
