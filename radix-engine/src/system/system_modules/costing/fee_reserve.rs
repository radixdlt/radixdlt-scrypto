use super::FeeReserveFinalizationSummary;
use crate::{
    errors::CanBeAbortion,
    track::interface::StoreCommit,
    transaction::{AbortReason, CostingParameters},
    types::*,
};
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use sbor::rust::cmp::min;
use transaction::prelude::TransactionCostingParameters;

// Note: for performance reason, `u128` is used to represent decimal in this file.

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FeeReserveError {
    InsufficientBalance {
        required: Decimal,
        remaining: Decimal,
    },
    Overflow,
    LimitExceeded {
        limit: u32,
        committed: u32,
        new: u32,
    },
    LoanRepaymentFailed {
        xrd_owed: Decimal,
    },
    Abort(AbortReason),
}

impl CanBeAbortion for FeeReserveError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::Abort(reason) => Some(reason),
            _ => None,
        }
    }
}

pub trait PreExecutionFeeReserve {
    /// This is only allowed before a transaction properly begins.
    /// After any other methods are called, this cannot be called again.
    fn consume_deferred(&mut self, cost_units: u32) -> Result<(), FeeReserveError>;
}

pub trait ExecutionFeeReserve {
    fn consume_execution(&mut self, cost_units: u32) -> Result<(), FeeReserveError>;

    fn consume_finalization(&mut self, cost_units: u32) -> Result<(), FeeReserveError>;

    fn consume_storage(&mut self, store_commit: &StoreCommit) -> Result<(), FeeReserveError>;

    fn consume_royalty(
        &mut self,
        royalty_amount: RoyaltyAmount,
        recipient: RoyaltyRecipient,
    ) -> Result<(), FeeReserveError>;

    fn lock_fee(
        &mut self,
        vault_id: NodeId,
        fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, FeeReserveError>;
}

pub trait FinalizingFeeReserve {
    fn finalize(self) -> FeeReserveFinalizationSummary;
}

pub trait FeeReserve: PreExecutionFeeReserve + ExecutionFeeReserve + FinalizingFeeReserve {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub enum RoyaltyRecipient {
    Package(PackageAddress, NodeId),
    Component(ComponentAddress, NodeId),
}

impl RoyaltyRecipient {
    pub fn vault_id(&self) -> NodeId {
        match self {
            RoyaltyRecipient::Package(_, v) | RoyaltyRecipient::Component(_, v) => *v,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct SystemLoanFeeReserve {
    execution_cost_unit_price: u128,
    execution_cost_unit_limit: u32,
    execution_cost_unit_loan: u32,

    finalization_cost_unit_price: u128,
    finalization_cost_unit_limit: u32,
    finalization_cost_unit_loan: u32,

    usd_price: u128,
    storage_price: u128,

    tip_percentage: u16,

    /// Whether to abort the transaction run when the loan is repaid.
    /// This is used when test-executing pending transactions.
    abort_when_loan_repaid: bool,

    /// (Cache) The effective execution cost unit price, with tips considered
    effective_execution_cost_unit_price: u128,
    /// (Cache) The effective execution cost unit price, with tips considered
    effective_finalization_cost_unit_price: u128,

    /// The XRD balance
    xrd_balance: u128,
    /// The amount of XRD owed to the system
    xrd_owed: u128,

    /// Execution costs
    execution_cost_units_committed: u32,
    execution_cost_units_deferred: u32,

    // Finalization costs
    finalization_cost_units_committed: u32,

    /// Royalty costs
    royalty_cost: u128,
    royalty_cost_breakdown: BTreeMap<RoyaltyRecipient, u128>,

    /// State expansion costs
    storage_cost: u128,

    /// Payments made during the execution of a transaction.
    locked_fees: Vec<(NodeId, LiquidFungibleResource, bool)>,
}

#[inline]
fn checked_add(a: u32, b: u32) -> Result<u32, FeeReserveError> {
    a.checked_add(b).ok_or(FeeReserveError::Overflow)
}

#[inline]
fn checked_assign_add(value: &mut u32, summand: u32) -> Result<(), FeeReserveError> {
    *value = checked_add(*value, summand)?;
    Ok(())
}

fn transmute_u128_as_decimal(a: u128) -> Decimal {
    Decimal(a.into())
}

fn transmute_decimal_as_u128(a: Decimal) -> Result<u128, FeeReserveError> {
    let i256 = a.0;
    i256.try_into().map_err(|_| FeeReserveError::Overflow)
}

impl SystemLoanFeeReserve {
    pub fn new(
        costing_parameters: &CostingParameters,
        transaction_costing_parameters: &TransactionCostingParameters,
        abort_when_loan_repaid: bool,
    ) -> Self {
        let effective_execution_cost_unit_price = transmute_decimal_as_u128(
            costing_parameters.execution_cost_unit_price
                + costing_parameters.execution_cost_unit_price
                    * transaction_costing_parameters.tip_percentage
                    / dec!(100),
        )
        .unwrap();
        let effective_finalization_cost_unit_price = transmute_decimal_as_u128(
            costing_parameters.finalization_cost_unit_price
                + costing_parameters.finalization_cost_unit_price
                    * transaction_costing_parameters.tip_percentage
                    / dec!(100),
        )
        .unwrap();
        let system_loan_in_xrd = effective_execution_cost_unit_price
            * costing_parameters.execution_cost_unit_loan as u128
            + effective_finalization_cost_unit_price
                * costing_parameters.finalization_cost_unit_loan as u128;

        Self {
            // Execution costing parameters
            execution_cost_unit_price: transmute_decimal_as_u128(
                costing_parameters.execution_cost_unit_price,
            )
            .unwrap(),
            execution_cost_unit_limit: costing_parameters.execution_cost_unit_limit,
            execution_cost_unit_loan: costing_parameters.execution_cost_unit_loan,

            // Finalization costing parameters
            finalization_cost_unit_price: transmute_decimal_as_u128(
                costing_parameters.finalization_cost_unit_price,
            )
            .unwrap(),
            finalization_cost_unit_limit: costing_parameters.finalization_cost_unit_limit,
            finalization_cost_unit_loan: costing_parameters.finalization_cost_unit_loan,

            // USD and storage price
            usd_price: transmute_decimal_as_u128(costing_parameters.usd_price).unwrap(),
            storage_price: transmute_decimal_as_u128(costing_parameters.storage_price).unwrap(),

            // Tipping percentage
            tip_percentage: transaction_costing_parameters.tip_percentage,

            // Aborting support
            abort_when_loan_repaid,

            // Cache
            effective_execution_cost_unit_price,
            effective_finalization_cost_unit_price,

            // Running balance
            xrd_balance: system_loan_in_xrd
                + transmute_decimal_as_u128(transaction_costing_parameters.free_credit_in_xrd)
                    .unwrap(),
            xrd_owed: system_loan_in_xrd,

            // Internal states
            execution_cost_units_committed: 0,
            execution_cost_units_deferred: 0,

            finalization_cost_units_committed: 0,

            royalty_cost_breakdown: BTreeMap::new(),
            royalty_cost: 0,

            storage_cost: 0,

            locked_fees: Vec::new(),
        }
    }

    pub fn cost_unit_limit(&self) -> u32 {
        self.execution_cost_unit_limit
    }

    pub fn cost_unit_price(&self) -> Decimal {
        transmute_u128_as_decimal(self.execution_cost_unit_price)
    }

    pub fn usd_price(&self) -> Decimal {
        transmute_u128_as_decimal(self.usd_price)
    }

    pub fn storage_price(&self) -> Decimal {
        transmute_u128_as_decimal(self.storage_price)
    }

    pub fn tip_percentage(&self) -> u32 {
        self.tip_percentage.into()
    }

    pub fn fee_balance(&self) -> Decimal {
        transmute_u128_as_decimal(self.xrd_balance)
    }

    pub fn royalty_cost_breakdown(&self) -> IndexMap<RoyaltyRecipient, Decimal> {
        self.royalty_cost_breakdown
            .clone()
            .into_iter()
            .map(|(k, v)| (k, transmute_u128_as_decimal(v)))
            .collect()
    }

    fn check_execution_cost_unit_limit(&self, cost_units: u32) -> Result<(), FeeReserveError> {
        if checked_add(self.execution_cost_units_committed, cost_units)?
            > self.execution_cost_unit_limit
        {
            return Err(FeeReserveError::LimitExceeded {
                limit: self.execution_cost_unit_limit,
                committed: self.execution_cost_units_committed,
                new: cost_units,
            });
        }
        Ok(())
    }

    fn check_finalization_cost_unit_limit(&self, cost_units: u32) -> Result<(), FeeReserveError> {
        if checked_add(self.finalization_cost_units_committed, cost_units)?
            > self.finalization_cost_unit_limit
        {
            return Err(FeeReserveError::LimitExceeded {
                limit: self.finalization_cost_unit_limit,
                committed: self.finalization_cost_units_committed,
                new: cost_units,
            });
        }
        Ok(())
    }

    fn consume_execution_internal(&mut self, cost_units: u32) -> Result<(), FeeReserveError> {
        self.check_execution_cost_unit_limit(cost_units)?;

        let amount = self.effective_execution_cost_unit_price * cost_units as u128;
        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance {
                required: transmute_u128_as_decimal(amount),
                remaining: transmute_u128_as_decimal(self.xrd_balance),
            });
        } else {
            self.xrd_balance -= amount;
            self.execution_cost_units_committed += cost_units;
            Ok(())
        }
    }

    fn consume_finalization_internal(&mut self, cost_units: u32) -> Result<(), FeeReserveError> {
        self.check_finalization_cost_unit_limit(cost_units)?;

        let amount = self.effective_finalization_cost_unit_price * cost_units as u128;
        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance {
                required: transmute_u128_as_decimal(amount),
                remaining: transmute_u128_as_decimal(self.xrd_balance),
            });
        } else {
            self.xrd_balance -= amount;
            self.finalization_cost_units_committed += cost_units;
            Ok(())
        }
    }

    fn consume_royalty_internal(
        &mut self,
        royalty_amount: RoyaltyAmount,
        recipient: RoyaltyRecipient,
    ) -> Result<(), FeeReserveError> {
        let amount = match royalty_amount {
            RoyaltyAmount::Xrd(xrd_amount) => transmute_decimal_as_u128(xrd_amount)?,
            RoyaltyAmount::Usd(usd_amount) => {
                transmute_decimal_as_u128(usd_amount)?
                    .checked_mul(self.usd_price)
                    .ok_or(FeeReserveError::Overflow)?
                    / 1_000_000_000_000_000_000
            }
            RoyaltyAmount::Free => 0u128,
        };
        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance {
                required: transmute_u128_as_decimal(amount),
                remaining: transmute_u128_as_decimal(self.xrd_balance),
            });
        } else {
            self.xrd_balance -= amount;
            self.royalty_cost_breakdown
                .entry(recipient)
                .or_default()
                .add_assign(amount);
            self.royalty_cost += amount;
            Ok(())
        }
    }

    pub fn repay_all(&mut self) -> Result<(), FeeReserveError> {
        // Apply deferred execution cost
        self.consume_execution_internal(self.execution_cost_units_deferred)?;
        self.execution_cost_units_deferred = 0;

        // Repay owed with balance
        let amount = min(self.xrd_balance, self.xrd_owed);
        self.xrd_owed -= amount;
        self.xrd_balance -= amount; // not used afterwards

        // Check outstanding loan
        if self.xrd_owed != 0 {
            return Err(FeeReserveError::LoanRepaymentFailed {
                xrd_owed: transmute_u128_as_decimal(self.xrd_owed),
            });
        }

        if self.abort_when_loan_repaid {
            return Err(FeeReserveError::Abort(
                AbortReason::ConfiguredAbortTriggeredOnFeeLoanRepayment,
            ));
        }

        Ok(())
    }

    pub fn revert_royalty(&mut self) {
        self.xrd_balance += self.royalty_cost_breakdown.values().sum::<u128>();
        self.royalty_cost_breakdown.clear();
        self.royalty_cost = 0;
    }

    #[inline]
    pub fn fully_repaid(&self) -> bool {
        self.xrd_owed == 0
    }
}

impl PreExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_deferred(&mut self, cost_units: u32) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        checked_assign_add(&mut self.execution_cost_units_deferred, cost_units)?;

        Ok(())
    }
}

impl ExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_execution(&mut self, cost_units: u32) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        self.consume_execution_internal(cost_units)?;

        if !self.fully_repaid()
            && self.execution_cost_units_committed >= self.execution_cost_unit_loan
        {
            self.repay_all()?;
        }

        Ok(())
    }

    fn consume_finalization(&mut self, cost_units: u32) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        self.consume_finalization_internal(cost_units)?;

        if !self.fully_repaid()
            && self.finalization_cost_units_committed >= self.finalization_cost_unit_loan
        {
            self.repay_all()?;
        }

        Ok(())
    }

    fn consume_royalty(
        &mut self,
        royalty_amount: RoyaltyAmount,
        recipient: RoyaltyRecipient,
    ) -> Result<(), FeeReserveError> {
        if royalty_amount.is_zero() {
            return Ok(());
        }

        self.consume_royalty_internal(royalty_amount, recipient)?;

        Ok(())
    }

    fn consume_storage(&mut self, store_commit: &StoreCommit) -> Result<(), FeeReserveError> {
        let delta = match store_commit {
            StoreCommit::Insert { size, .. } => *size,
            StoreCommit::Update { size, old_size, .. } => {
                if *size > *old_size {
                    *size - *old_size
                } else {
                    0
                }
            }
            StoreCommit::Delete { .. } => 0, // TODO: refund?
        };
        let amount = self.storage_price.saturating_mul(delta as u128);

        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance {
                required: transmute_u128_as_decimal(amount),
                remaining: transmute_u128_as_decimal(self.xrd_balance),
            });
        } else {
            self.xrd_balance -= amount;
            self.storage_cost += amount;
            Ok(())
        }
    }

    fn lock_fee(
        &mut self,
        vault_id: NodeId,
        mut fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, FeeReserveError> {
        // Update balance
        if !contingent {
            // Assumption: no overflow due to limited XRD supply
            self.xrd_balance += transmute_decimal_as_u128(fee.amount())?;
        }

        // Move resource
        self.locked_fees
            .push((vault_id, fee.take_all(), contingent));

        Ok(fee)
    }
}

impl FinalizingFeeReserve for SystemLoanFeeReserve {
    fn finalize(self) -> FeeReserveFinalizationSummary {
        let total_execution_cost_in_xrd = transmute_u128_as_decimal(self.execution_cost_unit_price)
            * self.execution_cost_units_committed;
        let total_finalization_cost_in_xrd =
            transmute_u128_as_decimal(self.finalization_cost_unit_price)
                * self.finalization_cost_units_committed;
        let total_tipping_cost_in_xrd = total_execution_cost_in_xrd * self.tip_percentage
            / dec!(100)
            + total_finalization_cost_in_xrd * self.tip_percentage / dec!(100);
        let royalty_cost_breakdown = self.royalty_cost_breakdown();

        FeeReserveFinalizationSummary {
            total_execution_cost_units_consumed: self.execution_cost_units_committed,
            total_finalization_cost_units_consumed: self.finalization_cost_units_committed,

            total_execution_cost_in_xrd,
            total_finalization_cost_in_xrd,
            total_tipping_cost_in_xrd,
            total_royalty_cost_in_xrd: transmute_u128_as_decimal(self.royalty_cost),
            total_storage_cost_in_xrd: transmute_u128_as_decimal(self.storage_cost),
            total_bad_debt_in_xrd: transmute_u128_as_decimal(self.xrd_owed),
            locked_fees: self.locked_fees,
            royalty_cost_breakdown,
        }
    }
}

impl FeeReserve for SystemLoanFeeReserve {}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_COMPONENT: ComponentAddress =
        component_address(EntityType::GlobalGenericComponent, 5);
    const TEST_VAULT_ID: NodeId = NodeId([0u8; NodeId::LENGTH]);
    const TEST_VAULT_ID_2: NodeId = NodeId([1u8; NodeId::LENGTH]);

    fn xrd<T: Into<Decimal>>(amount: T) -> LiquidFungibleResource {
        LiquidFungibleResource::new(amount.into())
    }

    fn new_test_fee_reserve(
        execution_cost_unit_price: Decimal,
        usd_price: Decimal,
        storage_price: Decimal,
        tip_percentage: u16,
        execution_cost_unit_limit: u32,
        execution_cost_unit_loan: u32,
        abort_when_loan_repaid: bool,
    ) -> SystemLoanFeeReserve {
        let mut costing_parameters = CostingParameters::default();
        costing_parameters.execution_cost_unit_price = execution_cost_unit_price;
        costing_parameters.execution_cost_unit_limit = execution_cost_unit_limit;
        costing_parameters.execution_cost_unit_loan = execution_cost_unit_loan;
        costing_parameters.usd_price = usd_price;
        costing_parameters.storage_price = storage_price;
        let mut transaction_costing_parameters = TransactionCostingParameters::default();
        transaction_costing_parameters.tip_percentage = tip_percentage;

        SystemLoanFeeReserve::new(
            &costing_parameters,
            &transaction_costing_parameters,
            abort_when_loan_repaid,
        )
    }

    #[test]
    fn test_consume_and_repay() {
        let mut fee_reserve = new_test_fee_reserve(dec!(1), dec!(1), dec!(0), 2, 100, 5, false);
        fee_reserve.consume_execution(2).unwrap();
        fee_reserve.lock_fee(TEST_VAULT_ID, xrd(3), false).unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_execution_cost_units_consumed, 2);
        assert_eq!(summary.total_execution_cost_in_xrd, dec!("2"));
        assert_eq!(summary.total_tipping_cost_in_xrd, dec!("0.04"));
        assert_eq!(summary.total_royalty_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_in_xrd, dec!("0"));
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve = new_test_fee_reserve(dec!(1), dec!(1), dec!(0), 2, 100, 5, false);
        assert_eq!(
            fee_reserve.consume_execution(6),
            Err(FeeReserveError::InsufficientBalance {
                required: dec!("6.12"),
                remaining: dec!("5.1"),
            }),
        );
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_execution_cost_units_consumed, 0);
        assert_eq!(summary.total_execution_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_in_xrd, dec!("0"));
    }

    #[test]
    fn test_lock_fee() {
        let mut fee_reserve = new_test_fee_reserve(dec!(1), dec!(1), dec!(0), 2, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_execution_cost_units_consumed, 0);
        assert_eq!(summary.total_execution_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_in_xrd, dec!("0"));
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut fee_reserve = new_test_fee_reserve(dec!(5), dec!(1), dec!(0), 0, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_execution_cost_units_consumed, 0);
        assert_eq!(summary.total_execution_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_in_xrd, dec!("0"));
        assert_eq!(summary.locked_fees, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }

    #[test]
    fn test_bad_debt() {
        let mut fee_reserve = new_test_fee_reserve(dec!(5), dec!(1), dec!(0), 1, 100, 50, false);
        fee_reserve.consume_execution(2).unwrap();
        assert_eq!(
            fee_reserve.repay_all(),
            Err(FeeReserveError::LoanRepaymentFailed { xrd_owed: dec!(1) })
        );
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), false);
        assert_eq!(summary.total_execution_cost_units_consumed, 2);
        assert_eq!(summary.total_execution_cost_in_xrd, dec!("10"));
        assert_eq!(summary.total_tipping_cost_in_xrd, dec!("0.1"));
        assert_eq!(summary.total_royalty_cost_in_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_in_xrd, dec!("10.1"));
        assert_eq!(summary.locked_fees, vec![],);
    }

    #[test]
    fn test_royalty_execution_mix() {
        let mut fee_reserve = new_test_fee_reserve(dec!(5), dec!(2), dec!(0), 1, 100, 50, false);
        fee_reserve.consume_execution(2).unwrap();
        fee_reserve
            .consume_royalty(
                RoyaltyAmount::Xrd(2.into()),
                RoyaltyRecipient::Package(PACKAGE_PACKAGE, TEST_VAULT_ID),
            )
            .unwrap();
        fee_reserve
            .consume_royalty(
                RoyaltyAmount::Usd(7.into()),
                RoyaltyRecipient::Package(PACKAGE_PACKAGE, TEST_VAULT_ID),
            )
            .unwrap();
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_execution_cost_in_xrd, dec!("10"));
        assert_eq!(summary.total_tipping_cost_in_xrd, dec!("0.1"));
        assert_eq!(summary.total_royalty_cost_in_xrd, dec!("16"));
        assert_eq!(summary.total_bad_debt_in_xrd, dec!("0"));
        assert_eq!(summary.locked_fees, vec![(TEST_VAULT_ID, xrd(100), false)]);
        assert_eq!(summary.total_execution_cost_units_consumed, 2);
        assert_eq!(
            summary.royalty_cost_breakdown,
            indexmap!(
                RoyaltyRecipient::Package(PACKAGE_PACKAGE, TEST_VAULT_ID) => dec!("16")
            )
        );
    }

    #[test]
    fn test_royalty_insufficient_balance() {
        let mut fee_reserve = new_test_fee_reserve(dec!(1), dec!(1), dec!(0), 0, 1000, 50, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve
            .consume_royalty(
                RoyaltyAmount::Xrd(90.into()),
                RoyaltyRecipient::Package(PACKAGE_PACKAGE, TEST_VAULT_ID),
            )
            .unwrap();
        assert_eq!(
            fee_reserve.consume_royalty(
                RoyaltyAmount::Xrd(80.into()),
                RoyaltyRecipient::Component(TEST_COMPONENT, TEST_VAULT_ID_2),
            ),
            Err(FeeReserveError::InsufficientBalance {
                required: dec!("80"),
                remaining: dec!("60"),
            }),
        );
    }
}
