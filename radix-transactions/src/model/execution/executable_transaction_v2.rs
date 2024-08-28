use crate::internal_prelude::*;

/// This is an executable form of the transaction, post stateless validation.
///
/// [`ExecutableTransactionV2`] originally launched with Cuttlefish, as a validation
/// target for `NotarizedTransactionV2`.
///
/// It has support for subintents and [`InstructionV2`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableTransactionV2 {
    pub(crate) context: ExecutionContextV2,
    /// The first is the primary / transaction intent, the following are the subintents
    pub(crate) intents: Vec<ExecutableIntentV2>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContextV2 {
    /// This is used as a source of pseudo-randomness for the id allocator and RUID generation
    pub unique_hash: Hash,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub payload_size: usize,
    pub num_of_signature_validations: usize,
    pub costing_parameters: TransactionCostingParameters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableIntentV2 {
    pub encoded_instructions_v2: Vec<u8>,
    pub intent_hash_nullification: IntentHashNullification,
    pub epoch_range: Option<EpochRange>,
    pub start_timestamp_inclusive: Option<Instant>,
    pub end_timestamp_exclusive: Option<Instant>,
    pub auth_zone_init: AuthZoneInit,
}
