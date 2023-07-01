use super::FeeSummary;
use crate::{
    errors::CanBeAbortion, track::interface::StoreCommit, transaction::AbortReason, types::*,
};
use radix_engine_constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE_IN_XRD, DEFAULT_SYSTEM_LOAN,
};
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use sbor::rust::cmp::min;

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
    LoanRepaymentFailed,
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
    fn consume_state_expansion(
        &mut self,
        store_commit: &StoreCommit,
    ) -> Result<(), FeeReserveError>;

    fn consume_royalty(
        &mut self,
        royalty_amount: RoyaltyAmount,
        recipient: RoyaltyRecipient,
        recipient_vault_id: NodeId,
    ) -> Result<(), FeeReserveError>;

    fn consume_execution(&mut self, cost_units: u32) -> Result<(), FeeReserveError>;

    fn lock_fee(
        &mut self,
        vault_id: NodeId,
        fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, FeeReserveError>;
}

pub trait FinalizingFeeReserve {
    fn finalize(self) -> FeeSummary;
}

pub trait FeeReserve: PreExecutionFeeReserve + ExecutionFeeReserve + FinalizingFeeReserve {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub enum RoyaltyRecipient {
    Package(PackageAddress),
    Component(ComponentAddress),
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct SystemLoanFeeReserve {
    /// The price of cost unit
    cost_unit_price: u128,
    /// The price of USD
    usd_price: u128,
    /// The price for adding a single byte to substate store
    state_expansion_price: u128,
    /// The tip percentage
    tip_percentage: u16,
    /// The number of cost units that can be consumed at most
    cost_unit_limit: u32,
    /// The number of cost units from system loan
    system_loan: u32,
    /// Whether to abort the transaction run when the loan is repaid.
    /// This is used when test-executing pending transactions.
    abort_when_loan_repaid: bool,

    /// (Cache) The effective execution price, with tips considered
    effective_price: u128,

    /// The XRD balance
    xrd_balance: u128,
    /// The amount of XRD owed to the system
    xrd_owed: u128,

    /// Execution costs
    execution_committed_sum: u32,
    execution_deferred_sum: u32,

    /// Royalty costs
    royalty_committed: BTreeMap<RoyaltyRecipient, (NodeId, u128)>,
    royalty_committed_sum: u128,

    /// State expansion costs
    state_expansion_committed: u128,

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
        cost_unit_price: Decimal,
        usd_price: Decimal,
        state_expansion_price: Decimal,
        tip_percentage: u16,
        cost_unit_limit: u32,
        system_loan: u32,
        abort_when_loan_repaid: bool,
    ) -> Self {
        let effective_price = transmute_decimal_as_u128(
            cost_unit_price + cost_unit_price * tip_percentage / dec!(100),
        )
        .unwrap();

        Self {
            cost_unit_price: transmute_decimal_as_u128(cost_unit_price).unwrap(),
            usd_price: transmute_decimal_as_u128(usd_price).unwrap(),
            state_expansion_price: transmute_decimal_as_u128(state_expansion_price).unwrap(),
            tip_percentage,
            cost_unit_limit,
            system_loan,
            abort_when_loan_repaid,

            effective_price,

            // System loan is used for both execution, royalty and state expansion
            xrd_balance: effective_price * system_loan as u128,
            xrd_owed: effective_price * system_loan as u128,

            execution_committed_sum: 0,
            execution_deferred_sum: 0,

            royalty_committed: BTreeMap::new(),
            royalty_committed_sum: 0,

            state_expansion_committed: 0,

            locked_fees: Vec::new(),
        }
    }

    pub fn with_free_credit(mut self, xrd_amount: Decimal) -> Self {
        self.xrd_balance += transmute_decimal_as_u128(xrd_amount).unwrap();
        self
    }

    pub fn cost_unit_limit(&self) -> u32 {
        self.cost_unit_limit
    }

    pub fn cost_unit_price(&self) -> Decimal {
        transmute_u128_as_decimal(self.cost_unit_price)
    }

    pub fn tip_price(&self) -> Decimal {
        self.cost_unit_price() * self.tip_percentage() / dec!(100)
    }

    pub fn usd_price(&self) -> Decimal {
        transmute_u128_as_decimal(self.usd_price)
    }

    pub fn tip_percentage(&self) -> u32 {
        self.tip_percentage.into()
    }

    pub fn fee_balance(&self) -> Decimal {
        transmute_u128_as_decimal(self.xrd_balance)
    }

    fn check_cost_unit_limit(&self, cost_units: u32) -> Result<(), FeeReserveError> {
        if checked_add(self.execution_committed_sum, cost_units)? > self.cost_unit_limit {
            return Err(FeeReserveError::LimitExceeded {
                limit: self.cost_unit_limit,
                committed: self.execution_committed_sum,
                new: cost_units,
            });
        }
        Ok(())
    }

    fn consume_execution_internal(&mut self, cost_units: u32) -> Result<(), FeeReserveError> {
        self.check_cost_unit_limit(cost_units)?;

        let amount = self.effective_price * cost_units as u128;
        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance {
                required: transmute_u128_as_decimal(amount),
                remaining: transmute_u128_as_decimal(self.xrd_balance),
            });
        } else {
            self.xrd_balance -= amount;
            self.execution_committed_sum += cost_units;
            Ok(())
        }
    }

    fn consume_royalty_internal(
        &mut self,
        royalty_amount: RoyaltyAmount,
        recipient: RoyaltyRecipient,
        recipient_vault_id: NodeId,
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
            self.royalty_committed
                .entry(recipient)
                .or_insert((recipient_vault_id, 0))
                .1
                .add_assign(amount);
            self.royalty_committed_sum += amount;
            Ok(())
        }
    }

    pub fn repay_all(&mut self) -> Result<(), FeeReserveError> {
        // Apply deferred execution cost
        self.consume_execution_internal(self.execution_deferred_sum)?;
        self.execution_deferred_sum = 0;

        // Repay owed with balance
        let amount = min(self.xrd_balance, self.xrd_owed);
        self.xrd_owed -= amount;
        self.xrd_balance -= amount; // not used afterwards

        // Check outstanding loan
        if self.xrd_owed != 0 {
            return Err(FeeReserveError::LoanRepaymentFailed);
        }

        if self.abort_when_loan_repaid {
            return Err(FeeReserveError::Abort(
                AbortReason::ConfiguredAbortTriggeredOnFeeLoanRepayment,
            ));
        }

        Ok(())
    }

    pub fn revert_royalty(&mut self) {
        self.xrd_balance += self.royalty_committed.values().map(|x| x.1).sum::<u128>();
        self.royalty_committed.clear();
        self.royalty_committed_sum = 0;
    }

    pub fn royalty_cost(&self) -> BTreeMap<RoyaltyRecipient, (NodeId, Decimal)> {
        self.royalty_committed
            .clone()
            .into_iter()
            .map(|(k, v)| (k, (v.0, transmute_u128_as_decimal(v.1))))
            .collect()
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

        checked_assign_add(&mut self.execution_deferred_sum, cost_units)?;

        Ok(())
    }
}

impl ExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_royalty(
        &mut self,
        royalty_amount: RoyaltyAmount,
        recipient: RoyaltyRecipient,
        recipient_vault_id: NodeId,
    ) -> Result<(), FeeReserveError> {
        if royalty_amount.is_zero() {
            return Ok(());
        }

        self.consume_royalty_internal(royalty_amount, recipient, recipient_vault_id)?;

        if !self.fully_repaid() && self.execution_committed_sum >= self.system_loan {
            self.repay_all()?;
        }

        Ok(())
    }

    fn consume_state_expansion(
        &mut self,
        store_commit: &StoreCommit,
    ) -> Result<(), FeeReserveError> {
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
        let amount = self.state_expansion_price.saturating_mul(delta as u128);

        if self.xrd_balance < amount {
            return Err(FeeReserveError::InsufficientBalance {
                required: transmute_u128_as_decimal(amount),
                remaining: transmute_u128_as_decimal(self.xrd_balance),
            });
        } else {
            self.xrd_balance -= amount;
            self.state_expansion_committed += amount;
            Ok(())
        }
    }

    fn consume_execution(&mut self, cost_units: u32) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        self.consume_execution_internal(cost_units)?;

        if !self.fully_repaid() && self.execution_committed_sum >= self.system_loan {
            self.repay_all()?;
        }

        Ok(())
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
    fn finalize(self) -> FeeSummary {
        let royalty_cost_breakdown = self.royalty_cost();
        let fee_summary = FeeSummary {
            cost_unit_limit: self.cost_unit_limit,
            cost_unit_price: transmute_u128_as_decimal(self.cost_unit_price),
            tip_percentage: self.tip_percentage,
            total_execution_cost_xrd: self.cost_unit_price() * self.execution_committed_sum,
            total_tipping_cost_xrd: self.tip_price() * self.execution_committed_sum,
            total_royalty_cost_xrd: transmute_u128_as_decimal(self.royalty_committed_sum),
            total_state_expansion_cost_xrd: transmute_u128_as_decimal(
                self.state_expansion_committed,
            ),
            total_bad_debt_xrd: transmute_u128_as_decimal(self.xrd_owed),
            locked_fees: self.locked_fees,
            execution_cost_breakdown: BTreeMap::new(),
            execution_cost_sum: self.execution_committed_sum,
            royalty_cost_breakdown,
            fee_payments: index_map_new(),
        };

        // Sanity check
        assert_eq!(
            fee_summary.total_execution_cost_xrd
                + fee_summary.total_tipping_cost_xrd
                + fee_summary.total_state_expansion_cost_xrd,
            fee_summary.fees_to_distribute() + fee_summary.tips_to_distribute()
        );
        fee_summary
    }
}

impl FeeReserve for SystemLoanFeeReserve {}

impl Default for SystemLoanFeeReserve {
    fn default() -> Self {
        Self::new(
            DEFAULT_COST_UNIT_PRICE_IN_XRD.try_into().unwrap(),
            DEFAULT_USD_PRICE_IN_XRD.try_into().unwrap(),
            DEFAULT_STATE_EXPANSION_PRICE_IN_XRD.try_into().unwrap(),
            0,
            DEFAULT_COST_UNIT_LIMIT,
            DEFAULT_SYSTEM_LOAN,
            false,
        )
    }
}

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

    #[test]
    fn test_consume_and_repay() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(dec!(1), dec!(1), dec!(0), 2, 100, 5, false);
        fee_reserve.consume_execution(2).unwrap();
        fee_reserve.lock_fee(TEST_VAULT_ID, xrd(3), false).unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.execution_cost_sum, 2);
        assert_eq!(summary.total_execution_cost_xrd, dec!("2"));
        assert_eq!(summary.total_tipping_cost_xrd, dec!("0.04"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(dec!(1), dec!(1), dec!(0), 2, 100, 5, false);
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
        assert_eq!(summary.execution_cost_sum, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_lock_fee() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(dec!(1), dec!(1), dec!(0), 2, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.execution_cost_sum, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(dec!(5), dec!(1), dec!(0), 0, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.execution_cost_sum, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
        assert_eq!(summary.locked_fees, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }

    #[test]
    fn test_bad_debt() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(dec!(5), dec!(1), dec!(0), 1, 100, 50, false);
        fee_reserve.consume_execution(2).unwrap();
        assert_eq!(
            fee_reserve.repay_all(),
            Err(FeeReserveError::LoanRepaymentFailed)
        );
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), false);
        assert_eq!(summary.execution_cost_sum, 2);
        assert_eq!(summary.total_execution_cost_xrd, dec!("10"));
        assert_eq!(summary.total_tipping_cost_xrd, dec!("0.1"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("10.1"));
        assert_eq!(summary.locked_fees, vec![],);
    }

    #[test]
    fn test_royalty_execution_mix() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(dec!(5), dec!(2), dec!(0), 1, 100, 50, false);
        fee_reserve.consume_execution(2).unwrap();
        fee_reserve
            .consume_royalty(
                RoyaltyAmount::Xrd(2.into()),
                RoyaltyRecipient::Package(PACKAGE_PACKAGE),
                TEST_VAULT_ID,
            )
            .unwrap();
        fee_reserve
            .consume_royalty(
                RoyaltyAmount::Usd(7.into()),
                RoyaltyRecipient::Package(PACKAGE_PACKAGE),
                TEST_VAULT_ID,
            )
            .unwrap();
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_execution_cost_xrd, dec!("10"));
        assert_eq!(summary.total_tipping_cost_xrd, dec!("0.1"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("16"));
        assert_eq!(summary.total_bad_debt_xrd, dec!("0"));
        assert_eq!(summary.locked_fees, vec![(TEST_VAULT_ID, xrd(100), false)]);
        assert_eq!(summary.execution_cost_sum, 2);
        assert_eq!(
            summary.royalty_cost_breakdown,
            btreemap!(
                RoyaltyRecipient::Package(PACKAGE_PACKAGE) => (TEST_VAULT_ID, dec!("16"))
            )
        );
    }

    #[test]
    fn test_royalty_insufficient_balance() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(dec!(1), dec!(1), dec!(0), 0, 1000, 50, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve
            .consume_royalty(
                RoyaltyAmount::Xrd(90.into()),
                RoyaltyRecipient::Package(PACKAGE_PACKAGE),
                TEST_VAULT_ID,
            )
            .unwrap();
        assert_eq!(
            fee_reserve.consume_royalty(
                RoyaltyAmount::Xrd(80.into()),
                RoyaltyRecipient::Component(TEST_COMPONENT),
                TEST_VAULT_ID_2
            ),
            Err(FeeReserveError::InsufficientBalance {
                required: dec!("80"),
                remaining: dec!("60"),
            }),
        );
    }
}
