use crate::engine::CanBeAbortion;
use crate::fee::FeeSummary;
use crate::model::Resource;
use crate::transaction::AbortReason;
use crate::types::*;
use radix_engine_constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_SYSTEM_LOAN,
};
use radix_engine_interface::api::types::{RENodeId, VaultId};
use sbor::rust::cmp::min;

// Note: for performance reason, `u128` is used to represent decimal in this file.

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Categorize)]
pub enum FeeReserveError {
    InsufficientBalance,
    Overflow,
    LimitExceeded,
    LoanRepaymentFailed,
    NotXrd,
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
        reason: &'static str,
    ) -> Result<(), FeeReserveError>;
}

pub trait ExecutionFeeReserve {
    fn consume_royalty(
        &mut self,
        receiver: RoyaltyReceiver,
        cost_units: u32,
    ) -> Result<(), FeeReserveError>;

    fn consume_multiplied_execution(
        &mut self,
        cost_units_per_multiple: u32,
        multiplier: usize,
        reason: &'static str,
    ) -> Result<(), FeeReserveError>;

    fn consume_execution(
        &mut self,
        cost_units: u32,
        reason: &'static str,
    ) -> Result<(), FeeReserveError>;

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, FeeReserveError>;
}

pub trait FinalizingFeeReserve {
    fn finalize(self) -> FeeSummary;
}

pub trait FeeReserve: PreExecutionFeeReserve + ExecutionFeeReserve + FinalizingFeeReserve {}

#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum RoyaltyReceiver {
    Package(PackageAddress, RENodeId),
    Component(ComponentAddress, RENodeId),
}

#[derive(Debug)]
pub struct SystemLoanFeeReserve {
    /// The price of cost unit
    cost_unit_price: u128,
    /// The tip percentage
    tip_percentage: u16,

    /// Payments made during the execution of a transaction.
    payments: Vec<(VaultId, Resource, bool)>,

    /// The cost unit balance (from system loan)
    loan_balance: u32,
    /// The XRD balance (from `lock_fee` payments)
    xrd_balance: u128,
    /// The amount of XRD owed to the system
    xrd_owed: u128,

    /// The amount of cost units consumed
    cost_units_consumed: u32,
    /// The max number of cost units that can be consumed
    cost_unit_limit: u32,
    /// At which point the system loan repayment is checked
    check_point: u32,

    /// Execution costs that are deferred
    execution_deferred: HashMap<&'static str, u32>,
    /// Execution cost breakdown
    execution: HashMap<&'static str, u32>,
    /// Royalty cost breakdown
    royalty: HashMap<RoyaltyReceiver, u32>,

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
            loan_balance: system_loan.into(),
            xrd_balance: 0,
            xrd_owed: 0,
            cost_units_consumed: 0,
            cost_unit_limit: cost_unit_limit.into(),
            check_point: system_loan.into(),
            execution_deferred: HashMap::new(),
            execution: HashMap::new(),
            royalty: HashMap::new(),
            effective_execution_price: cost_unit_price
                + cost_unit_price * tip_percentage as u128 / 100,
            effective_royalty_price: cost_unit_price,
            abort_when_loan_repaid,
        }
    }

    fn consume(&mut self, cost_units: u32, price: u128) -> Result<(), FeeReserveError> {
        // Check limit
        if checked_add(self.cost_units_consumed, cost_units)? > self.cost_unit_limit {
            return Err(FeeReserveError::LimitExceeded);
        }

        // Sort out the amount from system loan
        let from_loan = min(self.loan_balance, cost_units);

        // Sort out the amount from locked payments
        let from_locked = price * (cost_units - from_loan) as u128;
        if self.xrd_balance < from_locked {
            return Err(FeeReserveError::InsufficientBalance);
        }

        // Finally, apply state updates
        self.loan_balance -= from_loan;
        self.xrd_balance -= from_locked;
        self.xrd_owed += price * from_loan as u128;
        self.cost_units_consumed += cost_units;
        Ok(())
    }

    /// Repays loan and deferred costs in full.
    fn repay_all(&mut self) -> Result<(), FeeReserveError> {
        // Apply deferred execution costs
        let mut sum = 0;
        for v in self.execution_deferred.values() {
            checked_assign_add(&mut sum, *v)?;
        }
        self.consume(sum, self.execution_price())?;
        for (k, v) in self.execution_deferred.drain() {
            self.execution.entry(k).or_default().add_assign(v);
        }

        // Repay owed
        if self.xrd_balance < self.xrd_owed {
            return Err(FeeReserveError::LoanRepaymentFailed);
        } else {
            self.xrd_balance -= self.xrd_owed;
            self.xrd_owed = 0;
        }

        if self.abort_when_loan_repaid {
            return Err(FeeReserveError::Abort(
                AbortReason::ConfiguredAbortTriggeredOnFeeLoanRepayment,
            ));
        }

        Ok(())
    }

    fn attempt_to_repay_all(&mut self) {
        self.repay_all().ok();
    }

    fn execution_price(&self) -> u128 {
        self.effective_execution_price
    }

    fn royalty_price(&self) -> u128 {
        self.effective_royalty_price
    }

    fn fully_repaid(&self) -> bool {
        self.xrd_owed <= 0 && self.execution_deferred.is_empty()
    }
}

impl PreExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_deferred(
        &mut self,
        amount: u32,
        multiplier: usize,
        reason: &'static str,
    ) -> Result<(), FeeReserveError> {
        if amount == 0 {
            return Ok(());
        }

        let units_consumed = checked_multiply(amount, multiplier)?;

        checked_assign_add(
            self.execution_deferred.entry(reason).or_default(),
            units_consumed,
        )?;

        Ok(())
    }
}

impl ExecutionFeeReserve for SystemLoanFeeReserve {
    fn consume_royalty(
        &mut self,
        receiver: RoyaltyReceiver,
        amount: u32,
    ) -> Result<(), FeeReserveError> {
        if amount == 0 {
            return Ok(());
        }

        self.consume(amount.into(), self.execution_price())?;
        checked_assign_add(self.royalty.entry(receiver).or_default(), amount)?;

        if self.cost_units_consumed >= self.check_point && !self.fully_repaid() {
            self.repay_all()?;
        }
        Ok(())
    }

    fn consume_multiplied_execution(
        &mut self,
        cost_units_per_multiple: u32,
        multiplier: usize,
        reason: &'static str,
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
        cost_units: u32,
        reason: &'static str,
    ) -> Result<(), FeeReserveError> {
        if cost_units == 0 {
            return Ok(());
        }

        self.consume(cost_units, self.execution_price())?;
        checked_assign_add(self.execution.entry(reason).or_default(), cost_units)?;

        if self.cost_units_consumed >= self.check_point && !self.fully_repaid() {
            self.repay_all()?;
        }

        Ok(())
    }

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        mut fee: Resource,
        contingent: bool,
    ) -> Result<Resource, FeeReserveError> {
        if fee.resource_address() != RADIX_TOKEN {
            return Err(FeeReserveError::NotXrd);
        }

        // Update balance
        if !contingent {
            // Assumption: no overflow due to limited XRD supply
            self.xrd_balance += decimal_to_u128(fee.amount());
        }

        // Move resource
        self.payments.push((vault_id, fee.take_all(), contingent));

        Ok(fee)
    }
}

impl FinalizingFeeReserve for SystemLoanFeeReserve {
    fn finalize(mut self) -> FeeSummary {
        // In case the transaction finishes before check point.
        self.attempt_to_repay_all();

        FeeSummary {
            cost_unit_limit: self.cost_unit_limit,
            cost_unit_consumed: self.cost_units_consumed,
            cost_unit_price: u128_to_decimal(self.cost_unit_price),
            tip_percentage: self.tip_percentage,
            total_execution_cost_xrd: u128_to_decimal(
                self.execution_price() * self.execution.values().sum::<u32>() as u128,
            ),
            total_royalty_cost_xrd: u128_to_decimal(
                self.royalty_price() * self.royalty.values().sum::<u32>() as u128,
            ),
            bad_debt_xrd: u128_to_decimal(self.xrd_owed),
            vault_locks: self.payments,
            vault_payments_xrd: None, // Resolved later
            execution_cost_unit_breakdown: self
                .execution
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            royalty_cost_unit_breakdown: self.royalty,
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
    use radix_engine_interface::constants::RADIX_TOKEN;

    const TEST_VAULT_ID: VaultId = [0u8; 36];

    fn xrd<T: Into<Decimal>>(amount: T) -> Resource {
        Resource::new_fungible(RADIX_TOKEN, 18, amount.into())
    }

    #[test]
    fn test_consume_and_repay() {
        let mut fee_reserve = SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 5, false);
        fee_reserve
            .consume_multiplied_execution(2, 1, "test")
            .unwrap();
        fee_reserve.lock_fee(TEST_VAULT_ID, xrd(3), false).unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.cost_unit_consumed, 2);
        assert_eq!(summary.total_execution_cost_xrd, dec!("2") + dec!("0.04"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("0"));
        assert_eq!(summary.bad_debt_xrd, dec!("0"));
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve = SystemLoanFeeReserve::new(decimal_to_u128(dec!(1)), 2, 100, 5, false);
        assert_eq!(
            Err(FeeReserveError::InsufficientBalance),
            fee_reserve.consume_multiplied_execution(6, 1, "test")
        );
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.cost_unit_consumed, 0);
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
        assert_eq!(summary.cost_unit_consumed, 0);
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
        assert_eq!(summary.cost_unit_consumed, 0);
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
            .consume_multiplied_execution(2, 1, "test")
            .unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), false);
        assert_eq!(summary.cost_unit_consumed, 2);
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
            .consume_multiplied_execution(2, 1, "test")
            .unwrap();
        fee_reserve
            .consume_royalty(
                RoyaltyReceiver::Package(FAUCET_PACKAGE, RENodeId::Package([0u8; 36])),
                2,
            )
            .unwrap();
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.cost_unit_consumed, 4);
        assert_eq!(summary.total_execution_cost_xrd, dec!("10.1"));
        assert_eq!(summary.total_royalty_cost_xrd, dec!("10"));
        assert_eq!(summary.bad_debt_xrd, dec!("0"));
        assert_eq!(summary.vault_locks, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }
}
