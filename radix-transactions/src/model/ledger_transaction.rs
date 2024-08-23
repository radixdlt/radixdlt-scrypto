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
}

const GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 0;
const USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 1;
const ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 2;
const FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 3;

define_raw_transaction_payload!(RawLedgerTransaction);

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
    pub fn into_user(self) -> Option<Box<PreparedNotarizedTransactionV1>> {
        match self.inner {
            PreparedLedgerTransactionInner::UserV1(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_user(&self) -> Option<&PreparedNotarizedTransactionV1> {
        match &self.inner {
            PreparedLedgerTransactionInner::UserV1(t) => Some(t.as_ref()),
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
                PreparedLedgerTransactionInner::UserV1(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.transaction_intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                PreparedLedgerTransactionInner::RoundUpdateV1(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
                    }
                }
                PreparedLedgerTransactionInner::FlashV1(t) => {
                    TypedTransactionIdentifiers::FlashV1 {
                        flash_transaction_hash: t.flash_transaction_hash(),
                    }
                }
            },
        }
    }
}

impl HasSummary for PreparedLedgerTransaction {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

#[derive(BasicCategorize)]
pub enum PreparedLedgerTransactionInner {
    #[sbor(discriminator(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR))]
    Genesis(Box<PreparedGenesisTransaction>),
    #[sbor(discriminator(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV1(Box<PreparedNotarizedTransactionV1>),
    #[sbor(discriminator(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    RoundUpdateV1(Box<PreparedRoundUpdateTransactionV1>),
    #[sbor(discriminator(FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    FlashV1(Box<PreparedFlashTransactionV1>),
}

impl PreparedLedgerTransactionInner {
    pub fn get_ledger_hash(&self) -> LedgerTransactionHash {
        LedgerTransactionHash::for_kind(self.get_discriminator(), &self.get_summary().hash)
    }
}

impl HasSummary for PreparedLedgerTransactionInner {
    fn get_summary(&self) -> &Summary {
        match self {
            Self::Genesis(t) => t.get_summary(),
            Self::UserV1(t) => t.get_summary(),
            Self::RoundUpdateV1(t) => t.get_summary(),
            Self::FlashV1(t) => t.get_summary(),
        }
    }
}

impl TransactionFullChildPreparable for PreparedLedgerTransactionInner {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
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
                        let prepared =
                            PreparedSystemTransactionV1::prepare_as_full_body_child(decoder)?;
                        PreparedGenesisTransaction::Transaction(Box::new(prepared))
                    }
                    _ => return Err(unknown_discriminator(discriminator)),
                };
                PreparedLedgerTransactionInner::Genesis(Box::new(genesis_transaction))
            }
            USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let prepared = PreparedNotarizedTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::UserV1(Box::new(prepared))
            }
            ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let prepared =
                    PreparedRoundUpdateTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::RoundUpdateV1(Box::new(prepared))
            }
            FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                check_length(length, 1)?;
                let prepared = PreparedFlashTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::FlashV1(Box::new(prepared))
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
    Transaction(Box<PreparedSystemTransactionV1>),
}

impl HasSummary for PreparedGenesisTransaction {
    fn get_summary(&self) -> &Summary {
        match self {
            PreparedGenesisTransaction::Flash(summary) => summary,
            PreparedGenesisTransaction::Transaction(system_transaction) => {
                system_transaction.get_summary()
            }
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

impl TransactionPayloadPreparable for PreparedLedgerTransaction {
    type Raw = RawLedgerTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        decoder.track_stack_depth_increase()?;
        decoder.read_expected_enum_variant_header(TransactionDiscriminator::V1Ledger as u8, 1)?;
        let inner = PreparedLedgerTransactionInner::prepare_as_full_body_child(decoder)?;
        decoder.track_stack_depth_decrease()?;

        let summary = Summary {
            effective_length: inner.get_summary().effective_length,
            total_bytes_hashed: inner.get_summary().total_bytes_hashed,
            hash: inner.get_ledger_hash().0,
        };
        Ok(Self { inner, summary })
    }
}

pub struct ValidatedLedgerTransaction {
    pub inner: ValidatedLedgerTransactionInner,
    pub summary: Summary,
}

#[derive(BasicCategorize)]
pub enum ValidatedLedgerTransactionInner {
    #[sbor(discriminator(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR))]
    Genesis(Box<PreparedGenesisTransaction>),
    #[sbor(discriminator(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV1(Box<ValidatedNotarizedTransactionV1>),
    #[sbor(discriminator(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    RoundUpdateV1(Box<PreparedRoundUpdateTransactionV1>),
    #[sbor(discriminator(FLASH_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    FlashV1(Box<PreparedFlashTransactionV1>),
}

impl ValidatedLedgerTransaction {
    pub fn intent_hash_if_user(&self) -> Option<TransactionIntentHash> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(_) => None,
            ValidatedLedgerTransactionInner::UserV1(t) => Some(t.transaction_intent_hash()),
            ValidatedLedgerTransactionInner::RoundUpdateV1(_) => None,
            ValidatedLedgerTransactionInner::FlashV1(_) => None,
        }
    }

    /// Note - panics if it's a genesis flash
    pub fn get_executable(&self) -> ExecutableTransaction {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(genesis) => match genesis.as_ref() {
                PreparedGenesisTransaction::Flash(_) => {
                    panic!("Should not call get_executable on a genesis flash")
                }
                PreparedGenesisTransaction::Transaction(t) => t
                    .get_executable(btreeset!(system_execution(SystemExecution::Protocol)))
                    .into(),
            },
            ValidatedLedgerTransactionInner::UserV1(t) => t.get_executable().into(),
            ValidatedLedgerTransactionInner::RoundUpdateV1(t) => t.get_executable().into(),
            ValidatedLedgerTransactionInner::FlashV1(_) => {
                panic!("Should not call get_executable on a flash transaction")
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
                ValidatedLedgerTransactionInner::UserV1(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.transaction_intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                ValidatedLedgerTransactionInner::RoundUpdateV1(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
                    }
                }
                ValidatedLedgerTransactionInner::FlashV1(t) => {
                    TypedTransactionIdentifiers::FlashV1 {
                        flash_transaction_hash: t.flash_transaction_hash(),
                    }
                }
            },
        }
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
        Self::for_kind(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR, &hash.0)
    }

    pub fn for_user_v1(hash: &NotarizedTransactionHash) -> Self {
        Self::for_kind(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR, &hash.0)
    }

    pub fn for_round_update_v1(hash: &RoundUpdateTransactionHash) -> Self {
        Self::for_kind(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR, &hash.0)
    }

    fn for_kind(discriminator: u8, inner: &Hash) -> Self {
        Self(
            HashAccumulator::new()
                .update([
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1Ledger as u8,
                    discriminator,
                ])
                .update(inner.as_slice())
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

        let prepared_notarized = notarized.prepare().expect("Notarized can be prepared");

        let ledger = LedgerTransaction::UserV1(Box::new(notarized));
        let ledger_transaction_bytes = ledger.to_payload_bytes().expect("Can be encoded");
        LedgerTransaction::from_payload_bytes(&ledger_transaction_bytes).expect("Can be decoded");
        let prepared_ledger_transaction =
            PreparedLedgerTransaction::prepare_from_payload(&ledger_transaction_bytes)
                .expect("Can be prepared");

        let expected_intent_hash = LedgerTransactionHash::from_hash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1Ledger as u8,
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
            LedgerTransactionHash::for_user_v1(&prepared_notarized.notarized_transaction_hash()),
            expected_intent_hash
        );
    }
}
