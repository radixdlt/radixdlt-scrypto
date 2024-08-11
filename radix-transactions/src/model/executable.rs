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
    pub nullifier_updates: BTreeMap<Hash, NullifierUpdate>,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub payload_size: usize,
    pub num_of_signature_validations: usize,
    pub costing_parameters: TransactionCostingParameters,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NullifierUpdate {
    CheckAndUpdate { epoch_range: EpochRange },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableIntent {
    pub intent_hash: Hash,
    pub encoded_instructions: Rc<Vec<u8>>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub auth_zone_params: AuthZoneParams,
}

/// Executable form of transaction, post stateless validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    pub intents: Vec<ExecutableIntent>,
    pub references: IndexSet<Reference>,
    pub context: ExecutionContext,
    pub system: bool,
}

impl Executable {
    pub fn new(
        intents: Vec<ExecutableIntent>,
        mut references: IndexSet<Reference>,
        context: ExecutionContext,
        system: bool,
    ) -> Self {
        for intent in &intents {
            for proof in &intent.auth_zone_params.initial_proofs {
                references.insert(proof.resource_address().clone().into());
            }
            for resource in &intent.auth_zone_params.virtual_resources {
                references.insert(resource.clone().into());
            }
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
            intents,
            references,
            context,
            system,
        }
    }

    // Consuming builder-like customization methods:

    pub fn is_system(&self) -> bool {
        self.system
    }

    pub fn skip_epoch_range_check_and_update(mut self) -> Self {
        self.context.nullifier_updates.clear();
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

    pub fn root_intent_hash(&self) -> Hash {
        self.intents.get(0).unwrap().intent_hash
    }

    pub fn intent_tracker_updates(&self) -> &BTreeMap<Hash, NullifierUpdate> {
        &self.context.nullifier_updates
    }

    pub fn costing_parameters(&self) -> &TransactionCostingParameters {
        &self.context.costing_parameters
    }

    pub fn references(&self) -> &IndexSet<Reference> {
        &self.references
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
