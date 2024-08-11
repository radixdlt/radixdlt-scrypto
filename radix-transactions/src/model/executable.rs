use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
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
    pub num_of_signature_validations: usize,
    pub auth_zone_params: AuthZoneParams,
    pub costing_parameters: TransactionCostingParameters,
}

impl ExecutionContext {
    pub fn mock() -> Self {
        Self {
            intent_hash: TransactionIntentHash::NotToCheck {
                intent_hash: Hash([0u8; Hash::LENGTH]),
            },
            epoch_range: Default::default(),
            pre_allocated_addresses: Default::default(),
            payload_size: 0usize,
            num_of_signature_validations: 0usize,
            auth_zone_params: AuthZoneParams::default(),
            costing_parameters: TransactionCostingParameters::default(),
        }
    }
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

// Note: we have the two models below after finding an issue where a new field was added to the
// transaction costing parameters struct, which is used in the receipt, without moving to a new
// version of the receipt.
//
// Relevant discussion:
// https://rdxworks.slack.com/archives/C060RCS9MPW/p1715762426579329?thread_ts=1714585544.709299&cid=C060RCS9MPW

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct TransactionCostingParameters {
    pub tip_percentage: u16,
    /// Free credit for execution, for preview only!
    pub free_credit_in_xrd: Decimal,
    pub abort_when_loan_repaid: bool,
}

impl Default for TransactionCostingParameters {
    fn default() -> Self {
        Self {
            tip_percentage: DEFAULT_TIP_PERCENTAGE,
            free_credit_in_xrd: Default::default(),
            abort_when_loan_repaid: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct TransactionCostingParametersReceipt {
    pub tip_percentage: u16,
    /// Free credit for execution, for preview only!
    pub free_credit_in_xrd: Decimal,
}

impl Default for TransactionCostingParametersReceipt {
    fn default() -> Self {
        Self {
            tip_percentage: DEFAULT_TIP_PERCENTAGE,
            free_credit_in_xrd: Default::default(),
        }
    }
}

impl From<TransactionCostingParameters> for TransactionCostingParametersReceipt {
    fn from(value: TransactionCostingParameters) -> Self {
        Self {
            free_credit_in_xrd: value.free_credit_in_xrd,
            tip_percentage: value.tip_percentage,
        }
    }
}

/// Executable form of transaction, post stateless validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub references: IndexSet<Reference>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub context: ExecutionContext,
    pub system: bool,
}

impl Executable {
    pub fn new(
        encoded_instructions: Rc<Vec<u8>>,
        references: IndexSet<Reference>,
        blobs: Rc<IndexMap<Hash, Vec<u8>>>,
        context: ExecutionContext,
        system: bool,
    ) -> Self {
        let mut references = references;

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
            system,
        }
    }

    // Consuming builder-like customization methods:

    pub fn is_system(&self) -> bool {
        self.system
    }

    pub fn overwrite_intent_hash(mut self, hash: Hash) -> Self {
        match &mut self.context.intent_hash {
            TransactionIntentHash::ToCheck { intent_hash, .. }
            | TransactionIntentHash::NotToCheck { intent_hash } => {
                *intent_hash = hash;
            }
        }
        self
    }

    pub fn skip_epoch_range_check(mut self) -> Self {
        self.context.epoch_range = None;
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

    // Getters:

    pub fn intent_hash(&self) -> &TransactionIntentHash {
        &self.context.intent_hash
    }

    pub fn epoch_range(&self) -> Option<&EpochRange> {
        self.context.epoch_range.as_ref()
    }

    pub fn costing_parameters(&self) -> &TransactionCostingParameters {
        &self.context.costing_parameters
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

    pub fn num_of_signature_validations(&self) -> usize {
        self.context.num_of_signature_validations
    }
}
