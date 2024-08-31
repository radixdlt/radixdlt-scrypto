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
    pub(crate) unique_hash: Hash,
    pub(crate) pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub(crate) payload_size: usize,
    pub(crate) num_of_signature_validations: usize,
    pub(crate) costing_parameters: TransactionCostingParameters,
    pub(crate) overall_epoch_range: Option<EpochRange>,
    pub(crate) overall_start_timestamp_inclusive: Option<Instant>,
    pub(crate) overall_end_timestamp_exclusive: Option<Instant>,
    pub(crate) disable_limits_and_costing_modules: bool,
    pub(crate) intent_hash_nullifications: Vec<IntentHashNullification>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableIntentV2 {
    pub(crate) encoded_instructions_v2: Rc<Vec<u8>>,
    pub(crate) auth_zone_init: AuthZoneInit,
    pub(crate) references: Rc<IndexSet<Reference>>,
    pub(crate) blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub(crate) children_intent_indices: Vec<usize>,
}

impl Executable for ExecutableTransactionV2 {
    type Intent = ExecutableIntentV2;

    fn unique_hash(&self) -> &Hash {
        &self.context.unique_hash
    }

    fn overall_epoch_range(&self) -> Option<&EpochRange> {
        self.context.overall_epoch_range.as_ref()
    }

    fn overall_start_timestamp_inclusive(&self) -> Option<Instant> {
        self.context.overall_start_timestamp_inclusive
    }

    fn overall_end_timestamp_exclusive(&self) -> Option<Instant> {
        self.context.overall_end_timestamp_exclusive
    }

    fn costing_parameters(&self) -> &TransactionCostingParameters {
        &self.context.costing_parameters
    }

    fn pre_allocated_addresses(&self) -> &[PreAllocatedAddress] {
        &self.context.pre_allocated_addresses
    }

    fn payload_size(&self) -> usize {
        self.context.payload_size
    }

    fn num_of_signature_validations(&self) -> usize {
        self.context.num_of_signature_validations
    }

    fn disable_limits_and_costing_modules(&self) -> bool {
        self.context.disable_limits_and_costing_modules
    }

    fn intent_hash_nullifications(&self) -> &Vec<IntentHashNullification> {
        &self.context.intent_hash_nullifications
    }

    fn intents(&self) -> Vec<&Self::Intent> {
        self.intents.iter().collect()
    }
}

impl IntentDetails for ExecutableIntentV2 {
    fn executable_instructions(&self) -> ExecutableInstructions {
        ExecutableInstructions::V2Processor(self.encoded_instructions_v2.clone())
    }

    fn auth_zone_init(&self) -> &AuthZoneInit {
        &self.auth_zone_init
    }

    fn blobs(&self) -> Rc<IndexMap<Hash, Vec<u8>>> {
        self.blobs.clone()
    }

    fn references(&self) -> Rc<IndexSet<Reference>> {
        self.references.clone()
    }

    fn children_intent_indices(&self) -> &[usize] {
        &self.children_intent_indices
    }
}
