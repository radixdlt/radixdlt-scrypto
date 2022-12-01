use crate::fee::FeeSummary;
use crate::model::Resource;
use crate::types::*;
use radix_engine_constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_SYSTEM_LOAN,
};
use radix_engine_interface::api::types::{RENodeId, VaultId};
use radix_engine_interface::math::Decimal;
use sbor::rust::cmp::min;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum FeeReserveError {
    InsufficientBalance,
    Overflow,
    LimitExceeded,
    LoanRepaymentFailed,
    NotXrd,
}

pub trait FeeReserve {
    fn consume_royalty(
        &mut self,
        receiver: RoyaltyReceiver,
        amount: u32,
    ) -> Result<(), FeeReserveError>;

    fn consume_execution<T: ToString>(
        &mut self,
        amount: u32,
        multiplier: usize,
        reason: T,
        deferred: bool,
    ) -> Result<(), FeeReserveError>;

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, FeeReserveError>;

    fn finalize(self) -> FeeSummary;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RoyaltyReceiver {
    Package(PackageAddress, RENodeId),
    Component(ComponentAddress, RENodeId),
}

#[derive(Debug)]
pub struct SystemLoanFeeReserve {
    /// The price of cost unit
    cost_unit_price: Decimal,
    /// The tip percentage
    tip_percentage: u8,

    /// Payments made during the execution of a transaction.
    payments: Vec<(VaultId, Resource, bool)>,

    /// The cost unit balance (from system loan)
    loan_balance: u32,
    /// The XRD balance (from `lock_fee` payments)
    xrd_balance: Decimal,
    /// The amount of XRD owed to the system
    xrd_owed: Decimal,

    /// The amount of cost units consumed
    cost_unit_consumed: u32,
    /// The max number of cost units that can be consumed
    cost_unit_limit: u32,
    /// At which point the system loan repayment is checked
    check_point: u32,

    /// Execution costs that are deferred
    execution_deferred: HashMap<String, u32>,
    /// Execution cost breakdown
    execution: HashMap<String, u32>,
    /// Royalty cost breakdown
    royalty: HashMap<RoyaltyReceiver, u32>,
}

fn checked_add(a: u32, b: u32) -> Result<u32, FeeReserveError> {
    a.checked_add(b).ok_or(FeeReserveError::Overflow)
}

fn checked_assign_add(a: &mut u32, b: u32) -> Result<(), FeeReserveError> {
    *a = checked_add(*a, b)?;
    Ok(())
}

impl SystemLoanFeeReserve {
    pub fn no_fee() -> Self {
        Self::new(
            Decimal::ZERO,
            0,
            DEFAULT_COST_UNIT_LIMIT,
            DEFAULT_SYSTEM_LOAN,
        )
    }

    pub fn new(
        cost_unit_price: Decimal,
        tip_percentage: u8,
        cost_unit_limit: u32,
        system_loan: u32,
    ) -> Self {
        Self {
            cost_unit_price,
            tip_percentage,
            payments: Vec::new(),
            loan_balance: system_loan.into(),
            xrd_balance: Decimal::zero(),
            xrd_owed: Decimal::ZERO,
            cost_unit_consumed: 0,
            cost_unit_limit: cost_unit_limit.into(),
            check_point: system_loan.into(),
            execution_deferred: HashMap::new(),
            execution: HashMap::new(),
            royalty: HashMap::new(),
        }
    }

    fn consume(&mut self, n: u32, price: Decimal) -> Result<(), FeeReserveError> {
        // Check limit
        if checked_add(self.cost_unit_consumed, n)? > self.cost_unit_limit {
            return Err(FeeReserveError::LimitExceeded);
        }

        // Sort out the amount from system loan
        let from_loan = min(self.loan_balance, n);

        // Sort out the amount from locked payments
        let from_locked = price * (n - from_loan);
        if self.xrd_balance < from_locked {
            return Err(FeeReserveError::InsufficientBalance);
        }

        // Finally, apply state updates
        self.loan_balance -= from_loan;
        self.xrd_balance -= from_locked;
        self.xrd_owed += price * from_loan;
        self.cost_unit_consumed += n;
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
            self.xrd_owed = Decimal::ZERO;
        }

        Ok(())
    }

    fn attempt_to_repay_all(&mut self) {
        self.repay_all().ok();
    }

    fn execution_price(&self) -> Decimal {
        self.cost_unit_price + self.cost_unit_price * self.tip_percentage / 100
    }

    fn royalty_price(&self) -> Decimal {
        self.cost_unit_price
    }

    fn fully_repaid(&self) -> bool {
        self.xrd_owed <= 0.into() && self.execution_deferred.is_empty()
    }
}

impl FeeReserve for SystemLoanFeeReserve {
    fn consume_royalty(
        &mut self,
        receiver: RoyaltyReceiver,
        amount: u32,
    ) -> Result<(), FeeReserveError> {
        self.consume(amount.into(), self.execution_price())?;
        checked_assign_add(self.royalty.entry(receiver).or_default(), amount)?;

        if self.cost_unit_consumed >= self.check_point && !self.fully_repaid() {
            self.repay_all()?;
        }
        Ok(())
    }

    fn consume_execution<T: ToString>(
        &mut self,
        amount: u32,
        multiplier: usize,
        reason: T,
        deferred: bool,
    ) -> Result<(), FeeReserveError> {
        let n = u32::try_from(multiplier)
            .map_err(|_| FeeReserveError::Overflow)
            .and_then(|x| x.checked_mul(amount).ok_or(FeeReserveError::Overflow))?;
        let reason = reason.to_string();

        if deferred {
            checked_assign_add(self.execution_deferred.entry(reason).or_default(), n)?;
        } else {
            self.consume(n, self.execution_price())?;
            checked_assign_add(self.execution.entry(reason).or_default(), n)?;
        }

        if self.cost_unit_consumed >= self.check_point && !self.fully_repaid() {
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
            self.xrd_balance += fee.amount();
        }

        // Move resource
        self.payments.push((vault_id, fee.take_all(), contingent));

        Ok(fee)
    }

    fn finalize(mut self) -> FeeSummary {
        // In case the transaction finishes before check point.
        self.attempt_to_repay_all();

        FeeSummary {
            cost_unit_limit: self.cost_unit_limit,
            cost_unit_consumed: self.cost_unit_consumed,
            cost_unit_price: self.cost_unit_price,
            tip_percentage: self.tip_percentage,
            execution: self.execution_price() * self.execution.values().sum::<u32>(),
            royalty: self.royalty_price() * self.royalty.values().sum::<u32>(),
            bad_debt: self.xrd_owed,
            payments: self.payments,
            execution_breakdown: self.execution,
            royalty_breakdown: self.royalty,
        }
    }
}

impl Default for SystemLoanFeeReserve {
    fn default() -> Self {
        Self::new(
            DEFAULT_COST_UNIT_PRICE
                .parse()
                .expect("Invalid DEFAULT_COST_UNIT_PRICE"),
            0,
            DEFAULT_COST_UNIT_LIMIT,
            DEFAULT_SYSTEM_LOAN,
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
        let mut fee_reserve = SystemLoanFeeReserve::new(Decimal::ONE, 2, 100, 5);
        fee_reserve.consume_execution(2, 1, "test", false).unwrap();
        fee_reserve.lock_fee(TEST_VAULT_ID, xrd(3), false).unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.cost_unit_consumed, 2);
        assert_eq!(summary.execution, dec!("2") + dec!("0.04"));
        assert_eq!(summary.royalty, dec!("0"));
        assert_eq!(summary.bad_debt, dec!("0"));
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve = SystemLoanFeeReserve::new(Decimal::ONE, 2, 100, 5);
        assert_eq!(
            Err(FeeReserveError::InsufficientBalance),
            fee_reserve.consume_execution(6, 1, "test", false)
        );
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.cost_unit_consumed, 0);
        assert_eq!(summary.execution, dec!("0"));
        assert_eq!(summary.royalty, dec!("0"));
        assert_eq!(summary.bad_debt, dec!("0"));
    }

    #[test]
    fn test_lock_fee() {
        let mut fee_reserve = SystemLoanFeeReserve::new(Decimal::ONE, 2, 100, 500);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.cost_unit_consumed, 0);
        assert_eq!(summary.execution, dec!("0"));
        assert_eq!(summary.royalty, dec!("0"));
        assert_eq!(summary.bad_debt, dec!("0"));
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut fee_reserve = SystemLoanFeeReserve::new(5.into(), 0, 100, 500);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), true);
        assert_eq!(summary.cost_unit_consumed, 0);
        assert_eq!(summary.execution, dec!("0"));
        assert_eq!(summary.royalty, dec!("0"));
        assert_eq!(summary.bad_debt, dec!("0"));
        assert_eq!(summary.payments, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }

    #[test]
    fn test_bad_debt() {
        let mut fee_reserve = SystemLoanFeeReserve::new(5.into(), 1, 100, 50);
        fee_reserve.consume_execution(2, 1, "test", false).unwrap();
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid(), false);
        assert_eq!(summary.cost_unit_consumed, 2);
        assert_eq!(summary.execution, dec!("10"));
        assert_eq!(summary.royalty, dec!("0"));
        assert_eq!(summary.bad_debt, dec!("10"));
        assert_eq!(summary.payments, vec![],);
    }

    #[test]
    fn test_royalty_execution_mix() {
        let mut fee_reserve = SystemLoanFeeReserve::new(5.into(), 1, 100, 50);
        fee_reserve.consume_execution(2, 1, "test", false).unwrap();
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
        assert_eq!(summary.execution, dec!("10.1"));
        assert_eq!(summary.royalty, dec!("10"));
        assert_eq!(summary.bad_debt, dec!("0"));
        assert_eq!(summary.payments, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }
}
