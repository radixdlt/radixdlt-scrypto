use crate::internal_prelude::*;

/// This is an executable form of the transaction, post stateless validation.
///
/// [`ExecutableTransactionV2`] originally launched with Cuttlefish, as a validation
/// target for `NotarizedTransactionV2`.
///
/// It has support for subintents and [`InstructionV2`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableTransactionV2 {
    pub(crate) context: ExecutionContext,
    /// The first is the primary / transaction intent, the following are the subintents
    pub(crate) intents: Vec<ExecutableIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableIntent {
    pub(crate) encoded_instructions: Rc<Vec<u8>>,
    pub(crate) auth_zone_init: AuthZoneInit,
    pub(crate) references: Rc<IndexSet<Reference>>,
    pub(crate) blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub(crate) children_intent_indices: Vec<usize>,
}

impl Executable for ExecutableTransactionV2 {
    type Intent = ExecutableIntent;

    fn unique_hash(&self) -> &Hash {
        &self.context.unique_hash
    }

    fn overall_epoch_range(&self) -> Option<&EpochRange> {
        self.context.epoch_range.as_ref()
    }

    fn overall_start_timestamp_inclusive(&self) -> Option<Instant> {
        self.context.start_timestamp_inclusive
    }

    fn overall_end_timestamp_exclusive(&self) -> Option<Instant> {
        self.context.end_timestamp_exclusive
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

    fn intents(&self) -> &ExecutableIntents {
        unimplemented!();
    }
}

impl IntentDetails for ExecutableIntent {
    fn executable_instructions(&self) -> ExecutableInstructions {
        ExecutableInstructions::V2Processor(self.encoded_instructions.clone())
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
