use crate::internal_prelude::*;

/// This is an executable form of the transaction, post stateless validation.
///
/// [`ExecutableTransactionV1`] originally launched with Babylon.
/// Uses [`InstructionV1`] and [`NotarizedTransactionV1`]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableTransactionV1 {
    pub(crate) encoded_instructions_v1: Rc<Vec<u8>>,
    pub(crate) references: Rc<IndexSet<Reference>>,
    pub(crate) blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub(crate) auth_zone_init: AuthZoneInit,
    pub(crate) context: ExecutionContext,
    pub(crate) system: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    /// This is used as a source of pseudo-randomness for the id allocator and RUID generation
    pub unique_hash: Hash,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub payload_size: usize,
    pub num_of_signature_validations: usize,
    pub costing_parameters: TransactionCostingParameters,
    pub epoch_range: Option<EpochRange>,
    pub start_timestamp_inclusive: Option<Instant>,
    pub end_timestamp_exclusive: Option<Instant>,
    pub disable_limits_and_costing_modules: bool,
    pub intent_hash_nullifications: Vec<IntentHashNullification>,
}

impl ExecutableTransactionV1 {
    pub fn new(
        encoded_instructions_v1: Rc<Vec<u8>>,
        auth_zone_init: AuthZoneInit,
        references: IndexSet<Reference>,
        blobs: Rc<IndexMap<Hash, Vec<u8>>>,
        context: ExecutionContext,
        system: bool,
    ) -> Self {
        let mut references = references;

        for proof in &auth_zone_init.initial_non_fungible_id_proofs {
            references.insert(proof.resource_address().clone().into());
        }
        for resource in &auth_zone_init.simulate_every_proof_under_resources {
            references.insert(resource.clone().into());
        }

        for preallocated_address in &context.pre_allocated_addresses {
            references.insert(
                preallocated_address
                    .blueprint_id
                    .package_address
                    .clone()
                    .into(),
            );
        }

        Self {
            encoded_instructions_v1,
            references: Rc::new(references),
            blobs,
            context,
            auth_zone_init,
            system,
        }
    }

    // Consuming builder-like customization methods:

    pub fn skip_epoch_range_check(mut self) -> Self {
        self.context.epoch_range = None;
        self
    }

    pub fn skip_intent_hash_nullification(mut self) -> Self {
        self.context.intent_hash_nullifications.clear();
        self
    }

    pub fn apply_free_credit(mut self, free_credit_in_xrd: Decimal) -> Self {
        self.context.costing_parameters.free_credit_in_xrd = free_credit_in_xrd;
        self
    }

    pub fn abort_when_loan_repaid(mut self) -> Self {
        self.context.costing_parameters.abort_when_loan_repaid = true;
        self
    }
}

impl Executable for ExecutableTransactionV1 {
    type Intent = Self;

    fn unique_hash(&self) -> &Hash {
        &self.context.unique_hash
    }

    fn overall_epoch_range(&self) -> Option<&EpochRange> {
        self.context.epoch_range.as_ref()
    }

    fn overall_start_timestamp_inclusive(&self) -> Option<Instant> {
        None
    }

    fn overall_end_timestamp_exclusive(&self) -> Option<Instant> {
        None
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
        self.system
    }

    fn intents(&self) -> Vec<&Self::Intent> {
        vec![&self]
    }

    fn intent_hash_nullifications(&self) -> &Vec<IntentHashNullification> {
        &self.context.intent_hash_nullifications
    }

}

impl IntentDetails for ExecutableTransactionV1 {
    fn executable_instructions(&self) -> ExecutableInstructions {
        ExecutableInstructions::V1Processor(self.encoded_instructions_v1.clone())
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
        &NO_CHILDREN
    }
}

static NO_CHILDREN: [usize; 0] = [];
