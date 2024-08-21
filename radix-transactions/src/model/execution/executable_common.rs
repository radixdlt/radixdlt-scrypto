use crate::internal_prelude::*;

pub trait TransactionParameters {
    type Intent: IntentParameters;

    /// This is used as a source of pseudo-randomness for the id allocator and RUID generation
    fn unique_hash(&self) -> &Hash;
    fn overall_epoch_range(&self) -> Option<&EpochRange>;
    fn costing_parameters(&self) -> &TransactionCostingParameters;
    fn pre_allocated_addresses(&self) -> &Vec<PreAllocatedAddress>;
    fn payload_size(&self) -> usize;
    fn num_of_signature_validations(&self) -> usize;
    fn disable_limits_and_costing_modules(&self) -> bool;
    fn intents(&self) -> Vec<&Self::Intent>;
}

pub trait IntentParameters {
    fn intent_hash_check(&self) -> &IntentHashCheck;
    fn auth_zone_init(&self) -> &AuthZoneInit;
    fn blobs(&self) -> &IndexMap<Hash, Vec<u8>>;
    fn encoded_instructions(&self) -> &[u8];
    fn references(&self) -> &IndexSet<Reference>;
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub struct AuthZoneInit {
    pub initial_non_fungible_id_proofs: BTreeSet<NonFungibleGlobalId>,
    /// For use by the "assume_all_signature_proofs" flag
    pub simulate_every_proof_under_resources: BTreeSet<ResourceAddress>,
}

impl AuthZoneInit {
    pub fn proofs(proofs: BTreeSet<NonFungibleGlobalId>) -> Self {
        Self::new(proofs, btreeset!())
    }

    pub fn new(
        proofs: BTreeSet<NonFungibleGlobalId>,
        resources: BTreeSet<ResourceAddress>,
    ) -> Self {
        Self {
            initial_non_fungible_id_proofs: proofs,
            simulate_every_proof_under_resources: resources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochRange {
    pub start_epoch_inclusive: Epoch,
    pub end_epoch_exclusive: Epoch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposerTimestampRange {
    pub start_timestamp_inclusive: Option<Instant>,
    pub end_timestamp_inclusive: Option<Instant>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoSbor)]
pub struct PreAllocatedAddress {
    pub blueprint_id: BlueprintId,
    pub address: GlobalAddress,
}

impl From<(BlueprintId, GlobalAddress)> for PreAllocatedAddress {
    fn from((blueprint_id, address): (BlueprintId, GlobalAddress)) -> Self {
        PreAllocatedAddress {
            blueprint_id,
            address,
        }
    }
}
