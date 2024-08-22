use crate::internal_prelude::*;

pub trait Executable {
    type Intent: IntentDetails;

    /// This is used as a source of pseudo-randomness for the id allocator and RUID generation
    fn unique_hash(&self) -> &Hash;
    fn overall_epoch_range(&self) -> Option<&EpochRange>;
    fn costing_parameters(&self) -> &TransactionCostingParameters;
    fn pre_allocated_addresses(&self) -> &Vec<PreAllocatedAddress>;
    fn payload_size(&self) -> usize;
    fn num_of_signature_validations(&self) -> usize;
    fn disable_limits_and_costing_modules(&self) -> bool;
    fn intents(&self) -> Vec<&Self::Intent>;

    fn all_blob_hashes(&self) -> IndexSet<Hash> {
        self.intents()
            .iter()
            .flat_map(|i| i.blobs().keys())
            .cloned()
            .collect()
    }
    fn all_references(&self) -> IndexSet<Reference> {
        self.intents()
            .iter()
            .flat_map(|i| i.references())
            .cloned()
            .collect()
    }
}

pub trait IntentDetails {
    fn executable_instructions(&self) -> ExecutableInstructions;
    fn intent_hash_nullification(&self) -> &IntentHashNullification;
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IntentHashNullification {
    /// Should be checked with transaction tracker.
    /// Will be written
    TransactionIntent {
        intent_hash: TransactionIntentHash,
        expiry_epoch: Epoch,
        ignore_duplicate: bool,
    },
    /// Subintent - should only be written on failure
    Subintent {
        intent_hash: SubintentHash,
        expiry_epoch: Epoch,
        ignore_duplicate: bool,
    },
    /// For where there's no intent hash
    None,
}

impl IntentHashNullification {
    pub fn intent_hash(&self) -> Option<IntentHash> {
        match self {
            IntentHashNullification::TransactionIntent { intent_hash, .. } => {
                Some(IntentHash::Transaction(*intent_hash))
            }
            IntentHashNullification::Subintent { intent_hash, .. } => {
                Some(IntentHash::Sub(*intent_hash))
            }
            IntentHashNullification::None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor, Default)]
pub struct TransactionCostingParameters {
    pub tip: TipSpecifier,

    /// Free credit for execution, for preview only!
    pub free_credit_in_xrd: Decimal,

    /// Whether to abort the transaction run when the loan is repaid.
    /// This is used when test-executing pending transactions.
    pub abort_when_loan_repaid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum TipSpecifier {
    None,
    Percentage(u16),
    BasisPoints(u32),
}

impl TipSpecifier {
    pub fn basis_points(&self) -> u32 {
        match self {
            TipSpecifier::None => 0,
            TipSpecifier::Percentage(percentage) => (*percentage as u32) * 100,
            TipSpecifier::BasisPoints(basis_points) => *basis_points,
        }
    }

    pub fn proportion(&self) -> Decimal {
        // Notes:
        // * We don't use checked math because it can't overfow
        // * In order to make this math a bit cheaper, we multiply in I192 space to save a division
        match self {
            TipSpecifier::None => Decimal::ZERO,
            TipSpecifier::Percentage(percentage) => Decimal(I192::from(*percentage) * dec!(0.01).0),
            TipSpecifier::BasisPoints(basis_points) => {
                Decimal(I192::from(*basis_points) * dec!(0.0001).0)
            }
        }
    }

    pub fn fee_multiplier(&self) -> Decimal {
        Decimal::ONE + self.proportion()
    }

    #[deprecated = "Need to remove this function before releasing cuttlefish; once we can change the receipt"]
    pub fn truncate_to_percentage_u16(&self) -> u16 {
        match self {
            TipSpecifier::None => 0,
            TipSpecifier::Percentage(percentage) => *percentage,
            TipSpecifier::BasisPoints(basis_points) => {
                let truncated_percentage = *basis_points / 100;
                truncated_percentage.try_into().unwrap_or(u16::MAX)
            }
        }
    }

    pub fn truncate_to_percentage_u32(&self) -> u32 {
        match self {
            TipSpecifier::None => 0,
            TipSpecifier::Percentage(percentage) => *percentage as u32,
            TipSpecifier::BasisPoints(basis_points) => {
                let truncated_percentage = *basis_points / 100;
                truncated_percentage
            }
        }
    }
}

impl Default for TipSpecifier {
    fn default() -> Self {
        TipSpecifier::None
    }
}

// Note: TransactionCostingParametersReceiptV1 has diverged from TransactionCostingParameters because
// with the bottlenose release and the addition of abort_when_loan_repaid, we broke compatibility of
// the encoded transaction receipt.
//
// Relevant discussion:
// https://rdxworks.slack.com/archives/C060RCS9MPW/p1715762426579329?thread_ts=1714585544.709299&cid=C060RCS9MPW
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor, Default)]
pub struct TransactionCostingParametersReceiptV1 {
    pub tip_percentage: u16,
    pub free_credit_in_xrd: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor, Default)]
pub struct TransactionCostingParametersReceiptV2 {
    pub tip_basis_points: u32,
    pub free_credit_in_xrd: Decimal,
}

impl From<TransactionCostingParametersReceiptV1> for TransactionCostingParametersReceiptV2 {
    fn from(value: TransactionCostingParametersReceiptV1) -> Self {
        Self {
            tip_basis_points: TipSpecifier::Percentage(value.tip_percentage).basis_points(),
            free_credit_in_xrd: value.free_credit_in_xrd,
        }
    }
}

pub enum ExecutableInstructions<'a> {
    /// Instructions V1, using Babylon processor
    V1Processor(&'a [u8]),
    /// Instructions V2, using V2 capable subintent processor
    V2Processor(&'a [u8]),
}
