use std::iter;

use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableIntent {
    pub encoded_instructions: Vec<u8>,
    pub auth_zone_init: AuthZoneInit,
    pub references: IndexSet<Reference>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub children_subintent_indices: Vec<SubintentIndex>,
}

/// An index of the subintent in the parent ExecutableTransaction
/// Validation ensures that each subintent has a unique parent
/// and a unique path from the transaction intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ManifestSbor)]
#[sbor(transparent)]
pub struct SubintentIndex(pub usize);

pub trait IntoExecutable {
    type Error: Debug;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error>;

    /// For use in tests as a quick mechanism to get an executable.
    /// Validates with a network-independent validator, using the latest settings.
    fn into_executable_unwrap(self) -> ExecutableTransaction
    where
        Self: Sized,
    {
        self.into_executable(
            &TransactionValidator::new_with_static_config_network_agnostic(
                TransactionValidationConfig::latest(),
            ),
        )
        .unwrap()
    }
}

impl<'a, T: IntoExecutable + Clone> IntoExecutable for &'a T {
    type Error = T::Error;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        self.clone().into_executable(validator)
    }
}

impl IntoExecutable for ExecutableTransaction {
    type Error = core::convert::Infallible;

    fn into_executable(
        self,
        _validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        Ok(self)
    }
}

/// This is an executable form of the transaction, post stateless validation.
///
/// An `&ExecutableTransaction` is used to execute in the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableTransaction {
    pub(crate) transaction_intent: ExecutableIntent,
    pub(crate) subintents: Vec<ExecutableIntent>,
    pub(crate) context: ExecutionContext,
}

impl AsRef<ExecutableTransaction> for ExecutableTransaction {
    fn as_ref(&self) -> &ExecutableTransaction {
        self
    }
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
    pub proposer_timestamp_range: Option<ProposerTimestampRange>,
    pub disable_limits_and_costing_modules: bool,
    pub intent_hash_nullifications: Vec<IntentHashNullification>,
}

impl ExecutableTransaction {
    pub fn new_v1(
        encoded_instructions_v1: Vec<u8>,
        auth_zone_init: AuthZoneInit,
        references: IndexSet<Reference>,
        blobs: IndexMap<Hash, Vec<u8>>,
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
            transaction_intent: ExecutableIntent {
                encoded_instructions: encoded_instructions_v1,
                references,
                blobs,
                auth_zone_init,
                children_subintent_indices: vec![],
            },
            subintents: vec![],
        }
    }

    pub fn new_v2(
        mut transaction_intent: ExecutableIntent,
        mut subintents: Vec<ExecutableIntent>,
        context: ExecutionContext,
    ) -> Self {
        {
            let intent = &mut transaction_intent;
            for proof in &intent.auth_zone_init.initial_non_fungible_id_proofs {
                intent
                    .references
                    .insert(proof.resource_address().clone().into());
            }
            for resource in &intent.auth_zone_init.simulate_every_proof_under_resources {
                intent.references.insert(resource.clone().into());
            }
        }
        for intent in &mut subintents {
            for proof in &intent.auth_zone_init.initial_non_fungible_id_proofs {
                intent
                    .references
                    .insert(proof.resource_address().clone().into());
            }
            for resource in &intent.auth_zone_init.simulate_every_proof_under_resources {
                intent.references.insert(resource.clone().into());
            }
        }

        // Pre-allocated addresses are currently only used by the protocol (ie genesis + protocol updates).
        // Since there's no reason for the protocol to use child subintents, we only assign pre-allocated
        // addresses to the root subintent
        for preallocated_address in &context.pre_allocated_addresses {
            transaction_intent.references.insert(
                preallocated_address
                    .blueprint_id
                    .package_address
                    .clone()
                    .into(),
            );
        }

        Self {
            context,
            transaction_intent,
            subintents,
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

    pub fn unique_hash(&self) -> &Hash {
        &self.context.unique_hash
    }

    pub fn overall_epoch_range(&self) -> Option<&EpochRange> {
        self.context.epoch_range.as_ref()
    }

    pub fn overall_proposer_timestamp_range(&self) -> Option<&ProposerTimestampRange> {
        self.context.proposer_timestamp_range.as_ref()
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

    pub fn transaction_intent(&self) -> &ExecutableIntent {
        &self.transaction_intent
    }

    pub fn subintents(&self) -> &[ExecutableIntent] {
        &self.subintents
    }

    pub fn all_intents(&self) -> impl Iterator<Item = &ExecutableIntent> {
        iter::once(&self.transaction_intent).chain(self.subintents.iter())
    }

    pub fn intent_hash_nullifications(&self) -> &[IntentHashNullification] {
        &self.context.intent_hash_nullifications
    }

    pub fn all_blob_hashes(&self) -> IndexSet<Hash> {
        let mut hashes = indexset!();

        for intent in self.all_intents() {
            for hash in intent.blobs.keys() {
                hashes.insert(*hash);
            }
        }

        hashes
    }

    pub fn all_references(&self) -> IndexSet<Reference> {
        let mut references = indexset!();

        for intent in self.all_intents() {
            for reference in intent.references.iter() {
                references.insert(reference.clone());
            }
        }

        references
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutableTransaction;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn check_executable_transaction_can_be_cached_in_the_node_mempool_and_be_shared_between_threads(
    ) {
        assert_send::<ExecutableTransaction>();
        assert_sync::<ExecutableTransaction>();
    }
}
