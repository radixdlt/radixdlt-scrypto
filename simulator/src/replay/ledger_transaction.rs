use radix_engine_common::prelude::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_transactions::define_raw_transaction_payload;
use radix_transactions::prelude::*;
use sbor::FixedEnumVariant;

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct RoundUpdateTransactionV1 {
    pub proposer_timestamp_ms: i64,
    pub epoch: Epoch,
    pub round: Round,
    pub leader_proposal_history: LeaderProposalHistory,
}

impl RoundUpdateTransactionV1 {
    pub fn create_instructions(&self) -> Vec<InstructionV1> {
        vec![InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
            args: to_manifest_value(&ConsensusManagerNextRoundInput {
                round: self.round,
                proposer_timestamp_ms: self.proposer_timestamp_ms,
                leader_proposal_history: self.leader_proposal_history.clone(),
            })
            .expect("round update input encoding should succeed"),
        }]
    }

    pub fn prepare(&self) -> Result<PreparedRoundUpdateTransactionV1, PrepareError> {
        let prepared_instructions = InstructionsV1(self.create_instructions()).prepare_partial()?;
        let encoded_source = manifest_encode(&self)?;
        // Minor TODO - for a slight performance improvement, change this to be read from the decoder
        // As per the other hashes, don't include the prefix byte
        let source_hash = hash(&encoded_source[1..]);
        let instructions_hash = prepared_instructions.summary.hash;
        let round_update_hash = HashAccumulator::new()
            .update([
                TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                TransactionDiscriminator::V1RoundUpdate as u8,
            ])
            // We include the full source transaction contents
            .update(source_hash)
            // We also include the instructions hash, so the exact instructions can be proven
            .update(instructions_hash)
            .finalize();
        Ok(PreparedRoundUpdateTransactionV1 {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs: index_map_new(),
            summary: Summary {
                effective_length: prepared_instructions.summary.effective_length,
                total_bytes_hashed: prepared_instructions.summary.total_bytes_hashed,
                hash: round_update_hash,
            },
        })
    }
}

impl TransactionPayload for RoundUpdateTransactionV1 {
    type Versioned = FixedEnumVariant<{ TransactionDiscriminator::V1RoundUpdate as u8 }, Self>;
    type Prepared = PreparedRoundUpdateTransactionV1;
    type Raw = RawRoundUpdateTransactionV1;
}

pub struct PreparedRoundUpdateTransactionV1 {
    pub encoded_instructions: Vec<u8>,
    pub references: IndexSet<Reference>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub summary: Summary,
}

impl HasSummary for PreparedRoundUpdateTransactionV1 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

define_raw_transaction_payload!(RawRoundUpdateTransactionV1);

impl TransactionPayloadPreparable for PreparedRoundUpdateTransactionV1 {
    type Raw = RawRoundUpdateTransactionV1;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let decoded = decoder
            .decode::<<RoundUpdateTransactionV1 as TransactionPayload>::Versioned>()?
            .fields;
        decoded.prepare()
    }
}

impl TransactionFullChildPreparable for PreparedRoundUpdateTransactionV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let decoded = decoder.decode::<RoundUpdateTransactionV1>()?;
        decoded.prepare()
    }
}

impl PreparedRoundUpdateTransactionV1 {
    pub fn get_executable(&self) -> Executable<'_> {
        Executable::new(
            &self.encoded_instructions,
            &self.references,
            &self.blobs,
            ExecutionContext {
                intent_hash: TransactionIntentHash::NotToCheck {
                    intent_hash: self.summary.hash,
                },
                epoch_range: None,
                payload_size: 0,
                num_of_signature_validations: 0,
                auth_zone_params: AuthZoneParams {
                    initial_proofs: btreeset!(AuthAddresses::validator_role()),
                    virtual_resources: BTreeSet::new(),
                },
                costing_parameters: TransactionCostingParameters {
                    tip_percentage: 0,
                    free_credit_in_xrd: Decimal::ZERO,
                },
                pre_allocated_addresses: vec![],
            },
        )
    }
}

define_wrapped_hash!(RoundUpdateTransactionHash);

impl HasRoundUpdateTransactionHash for PreparedRoundUpdateTransactionV1 {
    fn round_update_transaction_hash(&self) -> RoundUpdateTransactionHash {
        RoundUpdateTransactionHash::from_hash(self.summary.hash)
    }
}

pub trait HasRoundUpdateTransactionHash {
    fn round_update_transaction_hash(&self) -> RoundUpdateTransactionHash;
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
        intent_hash: IntentHash,
        signed_intent_hash: SignedIntentHash,
        notarized_transaction_hash: NotarizedTransactionHash,
    },
    RoundUpdateV1 {
        round_update_hash: RoundUpdateTransactionHash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserTransactionIdentifiers<'a> {
    pub intent_hash: &'a IntentHash,
    pub signed_intent_hash: &'a SignedIntentHash,
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

#[derive(Debug, Clone, PartialEq, Eq, ManifestCategorize, ManifestEncode, ManifestDecode)]
pub enum LedgerTransaction {
    #[sbor(discriminator(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR))]
    Genesis(Box<GenesisTransaction>),
    #[sbor(discriminator(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV1(Box<NotarizedTransactionV1>),
    #[sbor(discriminator(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    RoundUpdateV1(Box<RoundUpdateTransactionV1>),
}

const GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 0;
const USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 1;
const ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 2;

define_raw_transaction_payload!(RawLedgerTransaction);

impl LedgerTransaction {
    pub fn to_raw(&self) -> Result<RawLedgerTransaction, EncodeError> {
        Ok(self.to_payload_bytes()?.into())
    }

    pub fn to_payload_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(&FixedEnumVariant::<
            { TransactionDiscriminator::V1Ledger as u8 },
            (&LedgerTransaction,),
        >::new((self,)))
    }

    pub fn from_raw(raw: &RawLedgerTransaction) -> Result<Self, DecodeError> {
        Self::from_payload_bytes(&raw.0)
    }

    pub fn from_raw_user(raw: &RawNotarizedTransaction) -> Result<Self, DecodeError> {
        Ok(LedgerTransaction::UserV1(Box::new(
            NotarizedTransactionV1::from_raw(raw)?,
        )))
    }

    pub fn from_payload_bytes(payload_bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(manifest_decode::<
            FixedEnumVariant<{ TransactionDiscriminator::V1Ledger as u8 }, (LedgerTransaction,)>,
        >(payload_bytes)?
        .into_fields()
        .0)
    }

    pub fn prepare(&self) -> Result<PreparedLedgerTransaction, PrepareError> {
        PreparedLedgerTransaction::prepare_from_payload(&self.to_payload_bytes()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestCategorize, ManifestEncode, ManifestDecode)]
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
                    intent_hash: t.intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                PreparedLedgerTransactionInner::RoundUpdateV1(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
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
}

impl ValidatedLedgerTransaction {
    pub fn intent_hash_if_user(&self) -> Option<IntentHash> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(_) => None,
            ValidatedLedgerTransactionInner::UserV1(t) => Some(t.intent_hash()),
            ValidatedLedgerTransactionInner::RoundUpdateV1(_) => None,
        }
    }

    pub fn as_genesis_flash(&self) -> Option<&Summary> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(genesis) => match genesis.as_ref() {
                PreparedGenesisTransaction::Flash(summary) => Some(summary),
                PreparedGenesisTransaction::Transaction(_) => None,
            },
            _ => None,
        }
    }

    pub fn get_executable(&self) -> Executable<'_> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(genesis) => match genesis.as_ref() {
                PreparedGenesisTransaction::Flash(_) => {
                    panic!("Should not call get_executable on a genesis flash")
                }
                PreparedGenesisTransaction::Transaction(t) => {
                    t.get_executable(btreeset!(AuthAddresses::system_role()))
                }
            },
            ValidatedLedgerTransactionInner::UserV1(t) => t.get_executable(),
            ValidatedLedgerTransactionInner::RoundUpdateV1(t) => t.get_executable(),
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
                    intent_hash: t.intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                ValidatedLedgerTransactionInner::RoundUpdateV1(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
                    }
                }
            },
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

impl HashHasHrp for LedgerTransactionHash {
    fn hrp(hrp_set: &HrpSet) -> &str {
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
