use crate::internal_prelude::*;

/// This is used in the node to increment rounds.
#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, ScryptoDescribe)]
pub struct RoundUpdateTransactionV1 {
    pub proposer_timestamp_ms: i64,
    pub epoch: Epoch,
    pub round: Round,
    pub leader_proposal_history: LeaderProposalHistory,
}

impl RoundUpdateTransactionV1 {
    /// Note - we purposefully restrict what the content of a Round Update transaction can do
    /// so we convert it to instructions at run-time.
    pub fn create_instructions(&self) -> Vec<InstructionV1> {
        vec![InstructionV1::CallMethod(CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
            args: to_manifest_value(&ConsensusManagerNextRoundInput {
                round: self.round,
                proposer_timestamp_ms: self.proposer_timestamp_ms,
                leader_proposal_history: self.leader_proposal_history.clone(),
            })
            .expect("round update input encoding should succeed"),
        })]
    }

    #[allow(deprecated)]
    pub fn prepare(&self) -> Result<PreparedRoundUpdateTransactionV1, PrepareError> {
        let prepared_instructions =
            InstructionsV1(Rc::new(self.create_instructions())).prepare_partial()?;
        let encoded_source = manifest_encode(&self)?;
        // Minor TODO - for a slight performance improvement, change this to be read from the decoder
        // As per the other hashes, don't include the prefix byte
        let source_hash = hash(&encoded_source[1..]);
        let instructions_hash = prepared_instructions.summary.hash;
        let round_update_hash = HashAccumulator::new()
            .concat([
                TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                TransactionDiscriminator::V1RoundUpdate as u8,
            ])
            // We include the full source transaction contents
            .concat(source_hash)
            // We also include the instructions hash, so the exact instructions can be proven
            .concat(instructions_hash)
            .finalize();
        Ok(PreparedRoundUpdateTransactionV1 {
            encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
            references: prepared_instructions.references,
            blobs: Rc::new(index_map_new()),
            summary: Summary {
                effective_length: prepared_instructions.summary.effective_length,
                total_bytes_hashed: prepared_instructions.summary.total_bytes_hashed,
                hash: round_update_hash,
            },
        })
    }
}

impl TransactionPayload for RoundUpdateTransactionV1 {
    type Prepared = PreparedRoundUpdateTransactionV1;
    type Raw = RawRoundUpdateTransactionV1;
}

pub struct PreparedRoundUpdateTransactionV1 {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub references: IndexSet<Reference>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub summary: Summary,
}

impl_has_summary!(PreparedRoundUpdateTransactionV1);

define_raw_transaction_payload!(RawRoundUpdateTransactionV1);

impl TransactionPayloadPreparable for PreparedRoundUpdateTransactionV1 {
    type Raw = RawRoundUpdateTransactionV1;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let decoded = RoundUpdateTransactionV1::from_payload_variant(decoder.decode()?);
        decoded.prepare()
    }
}

impl TransactionPreparableFromValue for PreparedRoundUpdateTransactionV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let decoded = decoder.decode::<RoundUpdateTransactionV1>()?;
        decoded.prepare()
    }
}

impl PreparedRoundUpdateTransactionV1 {
    pub fn get_executable(&self) -> ExecutableTransactionV1 {
        ExecutableTransactionV1::new(
            self.encoded_instructions.clone(),
            AuthZoneInit::proofs(btreeset!(system_execution(
                SystemExecution::Validator
            ))),
            self.references.clone(),
            self.blobs.clone(),
            ExecutionContext {
                unique_hash: self.summary.hash,
                intent_hash_nullifications: vec![],
                epoch_range: None,
                payload_size: 0,
                num_of_signature_validations: 0,
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::None,
                    free_credit_in_xrd: Decimal::ZERO,
                    abort_when_loan_repaid: false,
                },
                pre_allocated_addresses: vec![],
                disable_limits_and_costing_modules: true,
                start_timestamp_inclusive: None,
                end_timestamp_exclusive: None,
            },
            true,
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
