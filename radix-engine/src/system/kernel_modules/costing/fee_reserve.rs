use super::FeeSummary;
use crate::{errors::CanBeAbortion, transaction::AbortReason, types::*};
use radix_engine_constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_SYSTEM_LOAN,
};
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use strum::EnumCount;

// Note: for performance reason, `u128` is used to represent decimal in this file.

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FeeReserveError {
    InsufficientBalance,
    Overflow,
    LimitExceeded,
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
    fn consume_deferred(
        &mut self,
        amount: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError>;
}

pub trait ExecutionFeeReserve {
    fn consume_royalty(
        &mut self,
        cost_units: u32,
        recipient_vault_id: ObjectId,
    ) -> Result<(), FeeReserveError>;

    fn consume_multiplied_execution(
        &mut self,
        cost_units_per_multiple: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError>;

    fn consume_execution(
        &mut self,
        cost_units: u32,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError>;

    fn lock_fee(
        &mut self,
        vault_id: ObjectId,
        fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, FeeReserveError>;
}

pub trait FinalizingFeeReserve {
    fn finalize(self) -> FeeSummary;
}

pub trait FeeReserve: PreExecutionFeeReserve + ExecutionFeeReserve + FinalizingFeeReserve {}

#[repr(usize)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoSbor,
    IntoStaticStr,
    EnumCount,
    Display,
    FromRepr,
)]
pub enum CostingReason {
    TxBaseCost,
    TxPayloadCost,
    TxSignatureVerification,
    Invoke,
    DropNode,
    CreateNode,
    LockSubstate,
    ReadSubstate,
    WriteSubstate,
    DropLock,
    RunWasm,
    RunNative,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct SystemLoanFeeReserve {
    /// The price of cost unit
    cost_unit_price: u128,
    /// The tip percentage
    tip_percentage: u16,

    /// Payments made during the execution of a transaction.
    payments: Vec<(ObjectId, LiquidFungibleResource, bool)>,

    /// The cost unit balance (from system loan)
    remaining_loan_balance: u32,
    /// The XRD balance (from `lock_fee` payments)
    remaining_xrd_balance: u128,
    /// The amount of XRD owed to the system
    xrd_owed: u128,

    /// The amount of cost units consumed
    total_cost_units_consumed: u32,
    /// The max number of cost units that can be consumed
    cost_unit_limit: u32,

    /// Execution costs that are deferred
    execution_deferred: [u32; CostingReason::COUNT],
    execution_deferred_total: u32,
    /// Execution cost breakdown
    execution: [u32; CostingReason::COUNT],
    /// Royalty cost breakdown
    royalty: HashMap<ObjectId, u32>,

    /// Cache: effective execution price
    effective_execution_price: u128,
    /// Cache: effective royalty price
    effective_royalty_price: u128,

    /// Cache: Whether to abort the transaction run when the loan is repaid.
    /// This is used when test-executing pending transactions.
    abort_when_loan_repaid: bool,
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

#[inline]
fn checked_multiply(amount: u32, multiplier: usize) -> Result<u32, FeeReserveError> {
    u32::try_from(multiplier)
        .map_err(|_| FeeReserveError::Overflow)
        .and_then(|x| x.checked_mul(amount).ok_or(FeeReserveError::Overflow))
}

pub fn u128_to_decimal(a: u128) -> Decimal {
    Decimal(a.into())
}

pub fn decimal_to_u128(a: Decimal) -> u128 {
    let i256 = a.0;
    i256.try_into().expect("Overflow")
}

impl SystemLoanFeeReserve {
    pub fn no_fee() -> Self {
        Self::new(0, 0, DEFAULT_COST_UNIT_LIMIT, DEFAULT_SYSTEM_LOAN, false)
    }

    pub fn new(
        cost_unit_price: u128,
        tip_percentage: u16,
        cost_unit_limit: u32,
        system_loan: u32,
        abort_when_loan_repaid: bool,
    ) -> Self {
        Self {
            cost_unit_price,
            tip_percentage,
            payments: Vec::new(),
            remaining_loan_balance: system_loan.into(),
            remaining_xrd_balance: 0,
            xrd_owed: 0,
            total_cost_units_consumed: 0,
            cost_unit_limit: cost_unit_limit.into(),
            execution_deferred: [0u32; CostingReason::COUNT],
            execution_deferred_total: 0,
            execution: [0u32; CostingReason::COUNT],
            royalty: HashMap::new(),
            effective_execution_price: cost_unit_price
                + cost_unit_price * tip_percentage as u128 / 100,
            effective_royalty_price: cost_unit_price,
            abort_when_loan_repaid,
        }
    }

    fn consume(&mut self, cost_units_to_consume: u32, price: u128) -> Result<(), FeeReserveError> {
        // Check limit
        if checked_add(self.total_cost_units_consumed, cost_units_to_consume)?
            > self.cost_unit_limit
        {
            return Err(FeeReserveError::LimitExceeded);
        }

        /* To achieve the best performance, we may need to tweak the order of the three branches based on SYSTEM_LOAN_AMOUNT */

        if self.remaining_loan_balance >= cost_units_to_consume {
            // Finally, apply state updates
            self.xrd_owed += price * cost_units_to_consume as u128;
            self.remaining_loan_balance -= cost_units_to_consume;
            self.total_cost_units_consumed += cost_units_to_consume;
        } else if self.remaining_loan_balance == 0 {
            // Sort out the amount from balance
            let from_balance = price * cost_units_to_consume as u128;
            if self.remaining_xrd_balance < from_balance {
                return Err(FeeReserveError::InsufficientBalance);
            }

            // Finally, apply state updates
            self.remaining_xrd_balance -= from_balance;
            self.total_cost_units_consumed += cost_units_to_consume;
        } else {
            // Sort out the amount from balance
            let from_balance =
                price * (cost_units_to_consume - self.remaining_loan_balance) as u128;
            if self.remaining_xrd_balance < from_balance {
                return Err(FeeReserveError::InsufficientBalance);
            }

            // Finally, apply state updates
            self.xrd_owed += price * self.remaining_loan_balance as u128;
            self.remaining_loan_balance = 0;
            self.remaining_xrd_balance -= from_balance;
            self.total_cost_units_consumed += cost_units_to_consume;
        }
        Ok(())
    }

    /// Repays loan and deferred costs in full.
    pub fn repay_all(&mut self) -> Result<(), FeeReserveError> {
        // Apply deferred execution costs
        let mut sum = 0;
        for v in self.execution_deferred.iter() {
            checked_assign_add(&mut sum, *v)?;
        }
        self.consume(sum, self.execution_price())?;
        for i in 0..CostingReason::COUNT {
            self.execution[i] += self.execution_deferred[i];
            self.execution_deferred[i] = 0;
        }
        self.execution_deferred_total = 0;

        // Repay owed
        if self.remaining_xrd_balance < self.xrd_owed {
            return Err(FeeReserveError::LoanRepaymentFailed);
        } else {
            self.remaining_xrd_balance -= self.xrd_owed;
            self.xrd_owed = 0;
        }

        if self.abort_when_loan_repaid {
            return Err(FeeReserveError::Abort(
                AbortReason::ConfiguredAbortTriggeredOnFeeLoanRepayment,
            ));
        }

        Ok(())
    }

    pub fn revert_royalty_charges(&mut self) {
        let royalty = self.total_royalty_cost();
        self.total_cost_units_consumed -= royalty;
        self.royalty.clear();
    }

    pub fn royalty(&self) -> &HashMap<ObjectId, u32> {
        &self.royalty
    }

    #[inline]
    pub fn fully_repaid(&self) -> bool {
        self.xrd_owed == 0 && self.execution_deferred_total == 0
    }

    #[inline]
    fn execution_price(&self) -> u128 {
        self.effective_execution_price
    }

    #[inline]
    fn royalty_price(&self) -> u128 {
        self.effective_royalty_price
    }

    #[inline]
    fn total_execution_cost(&self) -> u32 {
        self.execution.iter().sum()
    }

    #[inline]
    fn total_royalty_cost(&self) -> u32 {
        self.royalty.values().sum()
    }
}

impl PreExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_deferred(
        &mut self,
        cost_units: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        let units_consumed = checked_multiply(cost_units, multiplier)?;

        checked_assign_add(
            &mut self.execution_deferred[reason as usize],
            units_consumed,
        )?;
        checked_assign_add(&mut self.execution_deferred_total, units_consumed)?;

        Ok(())
    }
}

impl ExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_royalty(
        &mut self,
        cost_units: u32,
        recipient_vault_id: ObjectId,
    ) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        self.consume(cost_units.into(), self.execution_price())?;
        checked_assign_add(
            self.royalty.entry(recipient_vault_id).or_default(),
            cost_units,
        )?;

        if self.remaining_loan_balance == 0 && !self.fully_repaid() {
            self.repay_all()?;
        }
        Ok(())
    }

    fn consume_multiplied_execution(
        &mut self,
        cost_units_per_multiple: u32,
        multiplier: usize,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError> {
        if multiplier == 0 {
            return Ok(());
        }

        self.consume_execution(
            checked_multiply(cost_units_per_multiple, multiplier)?,
            reason,
        )
    }

    fn consume_execution(
        &mut self,
        cost_units_to_consume: u32,
        reason: CostingReason,
    ) -> Result<(), FeeReserveError> {
        if cost_units_to_consume == 0 {
            return Ok(());
        }

        self.consume(cost_units_to_consume, self.execution_price())?;
        checked_assign_add(&mut self.execution[reason as usize], cost_units_to_consume)?;

        if self.remaining_loan_balance == 0 && !self.fully_repaid() {
            self.repay_all()?;
        }

        Ok(())
    }

    fn lock_fee(
        &mut self,
        vault_id: ObjectId,
        mut fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, FeeReserveError> {
        // Update balance
        if !contingent {
            // Assumption: no overflow due to limited XRD supply
            self.remaining_xrd_balance += decimal_to_u128(fee.amount());
        }

        // Move resource
        self.payments.push((vault_id, fee.take_all(), contingent));

        Ok(fee)
    }
}

impl FinalizingFeeReserve for SystemLoanFeeReserve {
    fn finalize(self) -> FeeSummary {
        FeeSummary {
            cost_unit_limit: self.cost_unit_limit,
            cost_unit_price: u128_to_decimal(self.cost_unit_price),
            tip_percentage: self.tip_percentage,
            total_cost_units_consumed: self.total_cost_units_consumed,
            total_execution_cost_xrd: u128_to_decimal(
                self.execution_price() * self.total_execution_cost() as u128,
            ),
            total_royalty_cost_xrd: u128_to_decimal(
                self.royalty_price() * self.total_royalty_cost() as u128,
            ),
            bad_debt_xrd: u128_to_decimal(self.xrd_owed),
            vault_locks: self.payments,
            execution_cost_unit_breakdown: self
                .execution
                .into_iter()
                .enumerate()
                .map(|(i, sum)| (CostingReason::from_repr(i).unwrap(), sum))
                .collect(),
            royalty_cost_unit_breakdown: self.royalty.into_iter().collect(),
        }
    }
}

impl FeeReserve for SystemLoanFeeReserve {}

impl Default for SystemLoanFeeReserve {
    fn default() -> Self {
        Self::new(
            DEFAULT_COST_UNIT_PRICE,
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

    const TEST_VAULT_ID: ObjectId = [0u8; 36];

    fn xrd<T: Into<Decimal>>(amount: T) -> LiquidFungibleResource {
        LiquidFungibleResource::new(amount.into())
    }

    #[test]
    fn test_consume_and_repay() {
        let mut fee_reserve = SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 5, false);
        fee_reserve
            .consume_multiplied_execution(2, 1, CostingReason::Invoke)
            .unwrap();
        fee_reserve.lock_fee(TEST_VAULT_ID, xrd(3), false).unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_cost_units_consumed, 2);
        assert_eq!(summary.total_execution_cost_xrd, dec!("2") + dec!("0.04"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve = SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 5, false);
        assert_eq!(
            Err(FeeReserveError::InsufficientBalance),
            fee_reserve.consume_multiplied_execution(6, 1, CostingReason::Invoke)
        );
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_cost_units_consumed, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_lock_fee() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_cost_units_consumed, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(5)), 0, 100, 500, false);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_cost_units_consumed, 0);
        assert_eq!(summary.total_execution_cost_xrd, dec!("0"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.bad_debt_xrd, dec!("0"));
        assert_eq!(summary.vault_locks, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }

    #[test]
    fn test_bad_debt() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(5)), 1, 100, 50, false);
        fee_reserve
            .consume_multiplied_execution(2, 1, CostingReason::Invoke)
            .unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), false);
        assert_eq!(summary.total_cost_units_consumed, 2);
        assert_eq!(summary.total_execution_cost_xrd, dec!("10.1"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.bad_debt_xrd, dec!("10.1"));
        assert_eq!(summary.vault_locks, vec![],);
    }

    #[test]
    fn test_royalty_execution_mix() {
        let mut fee_reserve =
            SystemLoanFeeReserve::new(decimal_to_u128(dec!(5)), 1, 100, 50, false);
        fee_reserve
            .consume_multiplied_execution(2, 1, CostingReason::Invoke)
            .unwrap();
        fee_reserve.consume_royalty(2, TEST_VAULT_ID).unwrap();
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        fee_reserve.repay_all().unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.total_cost_units_consumed, 4);
        assert_eq!(summary.total_execution_cost_xrd, dec!("10.1"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("10"));
        assert_eq!(summary.bad_debt_xrd, dec!("0"));
        assert_eq!(summary.vault_locks, vec![(TEST_VAULT_ID, xrd(100), false)]);
    }
}
