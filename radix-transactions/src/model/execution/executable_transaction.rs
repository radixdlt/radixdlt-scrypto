use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableIntent {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub auth_zone_init: AuthZoneInit,
    pub references: IndexSet<Reference>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    /// Indices against the parent Executable.
    /// It's a required invariant from validation that each non-root intent is included in exactly one parent.
    pub children_intent_indices: Vec<usize>,
}

/// This is an executable form of the transaction, post stateless validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableTransaction {
    pub(crate) intents: Vec<ExecutableIntent>,
    pub(crate) context: ExecutionContext,
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

impl ExecutableTransaction {
    pub fn new_v1(
        encoded_instructions_v1: Rc<Vec<u8>>,
        auth_zone_init: AuthZoneInit,
        references: IndexSet<Reference>,
        blobs: Rc<IndexMap<Hash, Vec<u8>>>,
        context: ExecutionContext,
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
            context,
            intents: vec![ExecutableIntent {
                encoded_instructions: encoded_instructions_v1,
                references,
                blobs,
                auth_zone_init,
                children_intent_indices: vec![],
            }],
        }
    }

    pub fn new_v2(mut intents: Vec<ExecutableIntent>, context: ExecutionContext) -> Self {
        for intent in &mut intents {
            for proof in &intent.auth_zone_init.initial_non_fungible_id_proofs {
                intent
                    .references
                    .insert(proof.resource_address().clone().into());
            }
            for resource in &intent.auth_zone_init.simulate_every_proof_under_resources {
                intent.references.insert(resource.clone().into());
            }
        }

        if let Some(root) = intents.get_mut(0) {
            for preallocated_address in &context.pre_allocated_addresses {
                root.references.insert(
                    preallocated_address
                        .blueprint_id
                        .package_address
                        .clone()
                        .into(),
                );
            }
        }

        Self { context, intents }
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

    pub fn unique_hash(&self) -> &Hash {
        &self.context.unique_hash
    }

    pub fn overall_epoch_range(&self) -> Option<&EpochRange> {
        self.context.epoch_range.as_ref()
    }

    pub fn overall_start_timestamp_inclusive(&self) -> Option<Instant> {
        self.context.start_timestamp_inclusive
    }

    pub fn overall_end_timestamp_exclusive(&self) -> Option<Instant> {
        self.context.end_timestamp_exclusive
    }

    pub fn costing_parameters(&self) -> &TransactionCostingParameters {
        &self.context.costing_parameters
    }

    pub fn pre_allocated_addresses(&self) -> &[PreAllocatedAddress] {
        &self.context.pre_allocated_addresses
    }

    pub fn payload_size(&self) -> usize {
        self.context.payload_size
    }

    pub fn num_of_signature_validations(&self) -> usize {
        self.context.num_of_signature_validations
    }

    pub fn disable_limits_and_costing_modules(&self) -> bool {
        self.context.disable_limits_and_costing_modules
    }

    pub fn intents(&self) -> &Vec<ExecutableIntent> {
        &self.intents
    }

    pub fn intent_hash_nullifications(&self) -> &Vec<IntentHashNullification> {
        &self.context.intent_hash_nullifications
    }

    pub fn all_blob_hashes(&self) -> IndexSet<Hash> {
        let mut hashes = indexset!();

        for intent in self.intents() {
            for hash in intent.blobs.keys() {
                hashes.insert(*hash);
            }
        }

        hashes
    }
    pub fn all_references(&self) -> IndexSet<Reference> {
        let mut references = indexset!();

        for intent in self.intents() {
            for reference in intent.references.iter() {
                references.insert(reference.clone());
            }
        }

        references
    }
}
