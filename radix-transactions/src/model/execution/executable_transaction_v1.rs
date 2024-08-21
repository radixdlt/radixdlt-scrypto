use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    /// This is used as a source of pseudo-randomness for the id allocator and RUID generation
    pub unique_hash: Hash,
    pub intent_hash_check: IntentHashCheck,
    pub epoch_range: Option<EpochRange>,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub payload_size: usize,
    pub num_of_signature_validations: usize,
    pub auth_zone_init: AuthZoneInit,
    pub costing_parameters: TransactionCostingParameters,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IntentHashCheck {
    /// Should be checked with transaction tracker.
    TransactionIntent {
        intent_hash: TransactionIntentHash,
        expiry_epoch: Epoch,
    },
    /// Subintent
    Subintent {
        intent_hash: SubintentHash,
        expiry_epoch: Epoch,
    },
    /// For where there's no intent hash
    None,
}

impl IntentHashCheck {
    pub fn intent_hash(&self) -> Option<IntentHash> {
        match self {
            IntentHashCheck::TransactionIntent { intent_hash, .. } => {
                Some(IntentHash::Transaction(*intent_hash))
            }
            IntentHashCheck::Subintent { intent_hash, .. } => Some(IntentHash::Sub(*intent_hash)),
            IntentHashCheck::None => None,
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

    /// Whether to abort the transaction run when the loan is repaid.
    /// This is used when test-executing pending transactions.
    pub abort_when_loan_repaid: bool,
}

impl Default for TransactionCostingParameters {
    fn default() -> Self {
        Self {
            tip_percentage: DEFAULT_TIP_PERCENTAGE,
            free_credit_in_xrd: Decimal::ZERO,
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
pub struct ExecutableTransactionV1 {
    pub(crate) encoded_instructions: Rc<Vec<u8>>,
    pub(crate) references: IndexSet<Reference>,
    pub(crate) blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub(crate) context: ExecutionContext,
    pub(crate) system: bool,
}

impl ExecutableTransactionV1 {
    pub fn new(
        encoded_instructions: Rc<Vec<u8>>,
        references: IndexSet<Reference>,
        blobs: Rc<IndexMap<Hash, Vec<u8>>>,
        context: ExecutionContext,
        system: bool,
    ) -> Self {
        let mut references = references;

        for proof in &context.auth_zone_init.initial_non_fungible_id_proofs {
            references.insert(proof.resource_address().clone().into());
        }
        for resource in &context.auth_zone_init.simulate_every_proof_under_resources {
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

    pub fn skip_epoch_range_check(mut self) -> Self {
        self.context.epoch_range = None;
        self
    }

    pub fn skip_intent_hash_check(mut self) -> Self {
        self.context.intent_hash_check = IntentHashCheck::None;
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

impl TransactionParameters for ExecutableTransactionV1 {
    type Intent = Self;

    fn unique_hash(&self) -> &Hash {
        &self.context.unique_hash
    }

    fn overall_epoch_range(&self) -> Option<&EpochRange> {
        self.context.epoch_range.as_ref()
    }

    fn costing_parameters(&self) -> &TransactionCostingParameters {
        &self.context.costing_parameters
    }

    fn pre_allocated_addresses(&self) -> &Vec<PreAllocatedAddress> {
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
}

impl IntentParameters for ExecutableTransactionV1 {
    fn intent_hash_check(&self) -> &IntentHashCheck {
        &self.context.intent_hash_check
    }

    fn auth_zone_init(&self) -> &AuthZoneInit {
        &self.context.auth_zone_init
    }

    fn blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        &self.blobs
    }

    fn encoded_instructions(&self) -> &[u8] {
        &self.encoded_instructions
    }

    fn references(&self) -> &IndexSet<Reference> {
        &self.references
    }
}
