use crate::fee::FeeSummary;
use crate::model::Resource;
use crate::types::*;
use radix_engine_constants::{
    DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_SYSTEM_LOAN,
};
use radix_engine_interface::api::types::{RENodeId, VaultId};
use radix_engine_interface::math::Decimal;

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
        amount: Decimal,
    ) -> Result<(), FeeReserveError>;

    fn consume_flat<T: ToString>(
        &mut self,
        cost: u32,
        reason: T,
        deferred: bool,
    ) -> Result<(), FeeReserveError>;

    fn consume_multiplied<T: ToString>(
        &mut self,
        quantity: u32,
        cost_multiplier: u32,
        reason: T,
        deferred: bool,
    ) -> Result<(), FeeReserveError>;

    fn consume_sized<T: ToString>(
        &mut self,
        length: usize,
        cost_multiplier: u32,
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

    fn cost_unit_limit(&self) -> u32;

    fn cost_unit_consumed(&self) -> u32;

    fn cost_unit_balance(&self) -> u32;

    fn xrd_balance(&self) -> Decimal;

    fn cost_unit_owed(&self) -> u32;
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
    tip_percentage: u32,
    /// Payments made during the execution of a transaction.
    payments: Vec<(VaultId, Resource, bool)>,
    /// The cost unit and XRD balances
    balance: (u32, Decimal),
    /// The number of cost units owed to the system
    owed: u32,
    /// The total cost units consumed
    consumed: u32,
    /// The total cost units deferred
    deferred: u32,
    /// The max number of cost units that can be consumed
    limit: u32,
    /// At which point the system loan repayment is checked
    check_point: u32,
    /// Cost breakdown
    cost_breakdown: HashMap<String, u32>,
    /// Royalty
    royalty: HashMap<RoyaltyReceiver, Decimal>,
}

impl SystemLoanFeeReserve {
    pub fn new(
        cost_unit_limit: u32,
        tip_percentage: u32,
        cost_unit_price: Decimal,
        system_loan: u32,
    ) -> Self {
        Self {
            cost_unit_price,
            tip_percentage,
            payments: Vec::new(),
            balance: (system_loan, Decimal::zero()),
            owed: system_loan,
            consumed: 0,
            deferred: 0,
            limit: cost_unit_limit,
            check_point: system_loan,
            cost_breakdown: HashMap::new(),
            royalty: HashMap::new(),
        }
    }

    /// Credits cost units.
    pub fn credit_cost_units(&mut self, n: u32) -> Result<(), FeeReserveError> {
        self.balance.0 = Self::checked_add(self.balance.0, n)?;
        self.attempt_to_repay_all();
        Ok(())
    }

    /// Debits cost units.
    fn debt_cost_units(&mut self, amount: u32) -> Result<(), FeeReserveError> {
        // First, attempt to debt from cost unit balance
        if self.balance.0 >= amount {
            self.balance.0 -= amount;
            Ok(())
        } else {
            // Then, attempt to debt from XRD balance
            //
            // Assumption: no overflow is expected given the limited supply of XRD, and max value of u32.
            let needed_xrd =
                (self.cost_unit_price + self.tip_price()) * Decimal::from(amount - self.balance.0);
            if self.balance.1 >= needed_xrd {
                self.balance.0 = 0;
                self.balance.1 -= needed_xrd;
                Ok(())
            } else {
                Err(FeeReserveError::InsufficientBalance)?
            }
        }
    }

    /// Repays loan and deferred costs in full.
    fn repay_all(&mut self) -> Result<(), FeeReserveError> {
        self.debt_cost_units(self.owed)
            .map_err(|_| FeeReserveError::LoanRepaymentFailed)?;
        self.owed = 0;

        self.debt_cost_units(self.deferred)
            .map_err(|_| FeeReserveError::LoanRepaymentFailed)?;
        self.consumed += self.deferred;
        self.deferred = 0;

        Ok(())
    }

    fn attempt_to_repay_all(&mut self) {
        self.repay_all().ok();
    }

    fn checked_add(a: u32, b: u32) -> Result<u32, FeeReserveError> {
        a.checked_add(b).ok_or(FeeReserveError::Overflow)
    }

    fn checked_add3(a: u32, b: u32, c: u32) -> Result<u32, FeeReserveError> {
        Self::checked_add(Self::checked_add(a, b)?, c)
    }

    fn tip_price(&self) -> Decimal {
        self.cost_unit_price * self.tip_percentage / 100
    }
}

impl FeeReserve for SystemLoanFeeReserve {
    fn consume_royalty(
        &mut self,
        receiver: RoyaltyReceiver,
        amount: Decimal,
    ) -> Result<(), FeeReserveError> {
        if self.balance.1 >= amount {
            self.balance.1 -= amount;
            self.royalty.entry(receiver).or_default().add_assign(amount);
            Ok(())
        } else {
            Err(FeeReserveError::InsufficientBalance)
        }
    }

    fn consume_flat<T: ToString>(
        &mut self,
        n: u32,
        reason: T,
        deferred: bool,
    ) -> Result<(), FeeReserveError> {
        // Check limit
        let total = Self::checked_add3(self.consumed, self.deferred, n)?;
        if total > self.limit {
            return Err(FeeReserveError::LimitExceeded);
        }

        // Update balance or owed
        if !deferred {
            self.debt_cost_units(n)?;
            self.consumed += n;
        } else {
            assert!(
                self.consumed < self.check_point,
                "All deferred fee consumption must be before system loan checkpoint"
            );
            self.deferred += n;
        }

        // Update cost breakdown
        self.cost_breakdown
            .entry(reason.to_string())
            .or_default()
            .add_assign(n);

        // Check system loan
        if self.consumed >= self.check_point && (self.owed > 0 || self.deferred > 0) {
            self.repay_all()?;
        }
        Ok(())
    }

    fn consume_multiplied<T: ToString>(
        &mut self,
        amount: u32,
        cost_multiplier: u32,
        reason: T,
        deferred: bool,
    ) -> Result<(), FeeReserveError> {
        let total = cost_multiplier
            .checked_mul(amount)
            .ok_or(FeeReserveError::Overflow)?;
        self.consume_flat(total, reason, deferred)
    }

    fn consume_sized<T: ToString>(
        &mut self,
        size: usize,
        cost_multiplier: u32,
        reason: T,
        deferred: bool,
    ) -> Result<(), FeeReserveError> {
        let amount: u32 = size.try_into().map_err(|_| FeeReserveError::Overflow)?;
        self.consume_multiplied(amount, cost_multiplier, reason, deferred)
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
            self.balance.1 += fee.amount();
        }

        // Move resource
        self.payments.push((vault_id, fee.take_all(), contingent));

        Ok(fee)
    }

    fn finalize(mut self) -> FeeSummary {
        // In case transaction finishes before system loan checkpoint.
        self.attempt_to_repay_all();

        // println!("{:?}", self);

        let mut total_royalty = Decimal::ZERO;
        self.royalty.values().for_each(|x| {
            total_royalty += *x;
        });
        FeeSummary {
            loan_fully_repaid: self.owed == 0 && self.deferred == 0,
            cost_unit_limit: self.limit,
            cost_unit_consumed: self.consumed,
            cost_unit_price: self.cost_unit_price,
            tip_percentage: self.tip_percentage,
            burned: self.cost_unit_price * self.consumed,
            tipped: Decimal::from(self.tip_price()) * self.consumed,
            royalty: total_royalty,
            payments: self.payments,
            cost_breakdown: self.cost_breakdown,
            royalty_breakdown: self.royalty,
        }
    }

    fn cost_unit_limit(&self) -> u32 {
        self.limit
    }

    fn cost_unit_consumed(&self) -> u32 {
        self.consumed
    }

    fn cost_unit_balance(&self) -> u32 {
        self.balance.0
    }

    fn xrd_balance(&self) -> Decimal {
        self.balance.1.clone()
    }

    fn cost_unit_owed(&self) -> u32 {
        self.owed
    }
}

impl Default for SystemLoanFeeReserve {
    fn default() -> Self {
        Self::new(
            DEFAULT_COST_UNIT_LIMIT,
            0,
            DEFAULT_COST_UNIT_PRICE
                .parse()
                .expect("Invalid DEFAULT_COST_UNIT_PRICE"),
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
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 2, Decimal::ONE, 5);
        fee_reserve.consume_flat(2, "test", false).unwrap();
        fee_reserve.lock_fee(TEST_VAULT_ID, xrd(3), false).unwrap();
        assert_eq!(3, fee_reserve.cost_unit_balance());
        assert_eq!(2, fee_reserve.cost_unit_consumed());
        assert_eq!(5, fee_reserve.cost_unit_owed());
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid, true);
        assert_eq!(summary.cost_unit_consumed, 2);
        assert_eq!(summary.burned, dec!("2"));
        assert_eq!(summary.tipped, dec!("0.04"));
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 2, Decimal::ONE, 5);
        assert_eq!(
            Err(FeeReserveError::InsufficientBalance),
            fee_reserve.consume_flat(6, "test", false)
        );
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid, true);
        assert_eq!(summary.cost_unit_consumed, 0);
        assert_eq!(summary.burned, dec!("0"));
        assert_eq!(summary.tipped, dec!("0"));
    }

    #[test]
    fn test_lock_fee() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 2, Decimal::ONE, 500);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        assert_eq!(500, fee_reserve.cost_unit_balance());
        assert_eq!(Decimal::from(100u32), fee_reserve.xrd_balance());
        assert_eq!(500, fee_reserve.cost_unit_owed());
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid, true);
        assert_eq!(summary.cost_unit_consumed, 0);
        assert_eq!(summary.burned, dec!("0"));
        assert_eq!(summary.tipped, dec!("0"));
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 0, 5.into(), 500);
        fee_reserve
            .lock_fee(TEST_VAULT_ID, xrd(100), false)
            .unwrap();
        assert_eq!(500, fee_reserve.cost_unit_balance());
        assert_eq!(500, fee_reserve.cost_unit_owed());
        let summary = fee_reserve.finalize();
        assert_eq!(summary.loan_fully_repaid, true);
        assert_eq!(summary.cost_unit_consumed, 0);
        assert_eq!(summary.burned, dec!("0"));
        assert_eq!(summary.tipped, dec!("0"));
        assert_eq!(summary.payments, vec![(TEST_VAULT_ID, xrd(100), false)],);
    }
}
