use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AuthZoneParams {
    pub initial_proofs: BTreeSet<NonFungibleGlobalId>,
    pub virtual_resources: BTreeSet<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EpochRange {
    pub start_epoch_inclusive: Epoch,
    pub end_epoch_exclusive: Epoch,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ExecutionContext {
    pub intent_hash: TransactionIntentHash,
    pub epoch_range: Option<EpochRange>,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub payload_size: usize,
    pub auth_zone_params: AuthZoneParams,
    pub fee_payment: FeePayment,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TransactionIntentHash {
    /// Should be checked with transaction tracker.
    ToCheck {
        intent_hash: Hash,
        expiry_epoch: Epoch,
    },
    /// Should not be checked by transaction tracker.
    NotToCheck { intent_hash: Hash },
}

impl TransactionIntentHash {
    pub fn as_hash(&self) -> &Hash {
        match self {
            TransactionIntentHash::ToCheck { intent_hash, .. }
            | TransactionIntentHash::NotToCheck { intent_hash } => intent_hash,
        }
    }
    pub fn to_hash(&self) -> Hash {
        self.as_hash().clone()
    }
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct FeePayment {
    pub tip_percentage: u16,
    /// Free credit for execution, for preview only!
    pub free_credit_in_xrd: Decimal,
}

/// Executable form of transaction, post stateless validation.
#[derive(Debug)]
pub struct Executable<'a> {
    encoded_instructions: &'a [u8],
    references: IndexSet<Reference>,
    blobs: &'a IndexMap<Hash, Vec<u8>>,
    context: ExecutionContext,
}

impl<'a> Executable<'a> {
    pub fn new(
        encoded_instructions: &'a [u8],
        references: &IndexSet<Reference>,
        blobs: &'a IndexMap<Hash, Vec<u8>>,
        context: ExecutionContext,
    ) -> Self {
        let mut references = references.clone();

        for proof in &context.auth_zone_params.initial_proofs {
            references.insert(proof.resource_address().clone().into());
        }
        for resource in &context.auth_zone_params.virtual_resources {
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
            encoded_instructions,
            references,
            blobs,
            context,
        }
    }

    pub fn intent_hash(&self) -> &TransactionIntentHash {
        &self.context.intent_hash
    }

    pub fn epoch_range(&self) -> Option<&EpochRange> {
        self.context.epoch_range.as_ref()
    }

    pub fn overwrite_intent_hash(&mut self, hash: Hash) {
        match &mut self.context.intent_hash {
            TransactionIntentHash::ToCheck { intent_hash, .. }
            | TransactionIntentHash::NotToCheck { intent_hash } => {
                *intent_hash = hash;
            }
        }
    }

    pub fn skip_epoch_range_check(&mut self) {
        self.context.epoch_range = None;
    }

    pub fn fee_payment(&self) -> &FeePayment {
        &self.context.fee_payment
    }

    pub fn blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        &self.blobs
    }

    pub fn encoded_instructions(&self) -> &[u8] {
        &self.encoded_instructions
    }

    pub fn references(&self) -> &IndexSet<Reference> {
        &self.references
    }

    pub fn auth_zone_params(&self) -> &AuthZoneParams {
        &self.context.auth_zone_params
    }

    pub fn pre_allocated_addresses(&self) -> &Vec<PreAllocatedAddress> {
        &self.context.pre_allocated_addresses
    }

    pub fn payload_size(&self) -> usize {
        self.context.payload_size
    }
}
