use crate::internal_prelude::*;
use decompiler::*;

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

impl PreAllocatedAddress {
    pub fn decompile_as_pseudo_instruction(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        // This aligns with AllocateGlobalAddress
        let instruction = DecompiledInstruction::new("USE_PREALLOCATED_ADDRESS")
            .add_argument(&self.blueprint_id.package_address)
            .add_argument(&self.blueprint_id.blueprint_name)
            .add_argument(context.new_address_reservation())
            .add_argument(&self.address);
        Ok(instruction)
    }
}

impl From<(BlueprintId, GlobalAddress)> for PreAllocatedAddress {
    fn from((blueprint_id, address): (BlueprintId, GlobalAddress)) -> Self {
        PreAllocatedAddress {
            blueprint_id,
            address,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// For system transactions which currently need to go through
    /// nullification process.
    /// TODO: Cleanup hash nullification and remove this
    System,
}

impl IntentHashNullification {
    pub fn intent_hash(&self) -> Option<IntentHash> {
        match self {
            IntentHashNullification::TransactionIntent { intent_hash, .. } => {
                Some(IntentHash::Transaction(*intent_hash))
            }
            IntentHashNullification::Subintent { intent_hash, .. } => {
                Some(IntentHash::Subintent(*intent_hash))
            }
            IntentHashNullification::System => None,
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
    pub tip_proportion: Decimal,
    pub free_credit_in_xrd: Decimal,
}

impl From<TransactionCostingParametersReceiptV1> for TransactionCostingParametersReceiptV2 {
    fn from(value: TransactionCostingParametersReceiptV1) -> Self {
        Self {
            tip_proportion: TipSpecifier::Percentage(value.tip_percentage).proportion(),
            free_credit_in_xrd: value.free_credit_in_xrd,
        }
    }
}
