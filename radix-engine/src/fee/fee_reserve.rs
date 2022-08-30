use crate::constants::{DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_SYSTEM_LOAN};
use crate::fee::FeeSummary;
use crate::model::ResourceContainer;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeeReserveError {
    OutOfCostUnit,
    Overflow,
    LimitExceeded,
    SystemLoanNotCleared,
}

// TODO: rename to `FeeReserve`
pub trait FeeReserve {
    fn consume<T: ToString>(&mut self, n: u32, reason: T) -> Result<(), FeeReserveError>;

    fn repay(
        &mut self,
        vault_id: VaultId,
        fee: ResourceContainer,
        contingent: bool,
    ) -> Result<ResourceContainer, FeeReserveError>;

    fn finalize(self) -> FeeSummary;

    fn limit(&self) -> u32;

    fn consumed(&self) -> u32;

    fn balance(&self) -> u32;

    fn owed(&self) -> u32;
}

pub struct SystemLoanFeeReserve {
    /// The price of cost unit
    cost_unit_price: Decimal,
    /// The tip percentage
    tip_percentage: u32,
    /// Payments made during the execution of a transaction.
    payments: Vec<(VaultId, ResourceContainer, bool)>,
    /// The balance cost units
    balance: u32,
    /// The number of cost units owed to the system
    owed: u32,
    /// The total cost units consumed so far
    consumed: u32,
    /// The max number of cost units that can be consumed
    limit: u32,
    /// At which point the system loan repayment is checked
    check_point: u32,
    /// Cost breakdown
    cost_breakdown: HashMap<String, u32>,
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
            balance: system_loan,
            owed: system_loan,
            consumed: 0,
            limit: cost_unit_limit,
            check_point: system_loan,
            cost_breakdown: HashMap::new(),
        }
    }
}

impl FeeReserve for SystemLoanFeeReserve {
    fn consume<T: ToString>(&mut self, n: u32, reason: T) -> Result<(), FeeReserveError> {
        self.balance = self
            .balance
            .checked_sub(n)
            .ok_or(FeeReserveError::OutOfCostUnit)?;
        self.consumed = self
            .consumed
            .checked_add(n)
            .ok_or(FeeReserveError::Overflow)?;

        self.cost_breakdown
            .entry(reason.to_string())
            .or_default()
            .add_assign(n);

        if self.consumed > self.limit {
            return Err(FeeReserveError::LimitExceeded);
        }
        if self.consumed >= self.check_point && self.owed > 0 {
            return Err(FeeReserveError::SystemLoanNotCleared);
        }
        Ok(())
    }

    fn repay(
        &mut self,
        vault_id: VaultId,
        mut fee: ResourceContainer,
        contingent: bool,
    ) -> Result<ResourceContainer, FeeReserveError> {
        let effective_cost_unit_price =
            self.cost_unit_price + self.cost_unit_price * self.tip_percentage / 100;

        // TODO: Add `TryInto` implementation once the new decimal types are in place
        let n = u32::from_str(
            (fee.liquid_amount() / effective_cost_unit_price)
                .round(0, RoundingMode::TowardsZero)
                .to_string()
                .as_str(),
        )
        .map_err(|_| FeeReserveError::Overflow)?;

        if !contingent {
            if n >= self.owed {
                self.balance = self
                    .balance
                    .checked_add(n - self.owed)
                    .ok_or(FeeReserveError::Overflow)?;
                self.owed = 0;
            } else {
                self.owed -= n;
            }
        }

        let actual_amount = effective_cost_unit_price * n;
        self.payments.push((
            vault_id,
            fee.take_by_amount(actual_amount)
                .expect("Failed to take from fee resource"),
            contingent,
        ));

        Ok(fee)
    }

    fn finalize(mut self) -> FeeSummary {
        if self.owed > 0 && self.balance != 0 {
            let n = u32::min(self.owed, self.balance);
            self.owed -= n;
            self.balance -= n;
        }

        FeeSummary {
            loan_fully_repaid: self.owed == 0,
            cost_unit_limit: self.limit,
            cost_unit_consumed: self.consumed,
            cost_unit_price: self.cost_unit_price,
            tip_percentage: self.tip_percentage,
            burned: self.cost_unit_price * self.consumed,
            tipped: self.cost_unit_price * self.tip_percentage / 100 * self.consumed,
            payments: self.payments,
            cost_breakdown: self.cost_breakdown,
        }
    }

    fn limit(&self) -> u32 {
        self.limit
    }

    fn consumed(&self) -> u32 {
        self.consumed
    }

    fn balance(&self) -> u32 {
        self.balance
    }

    fn owed(&self) -> u32 {
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

pub struct UnlimitedLoanFeeReserve {
    /// The price of cost unit
    cost_unit_price: Decimal,
    /// The tip percentage
    tip_percentage: u32,
    /// The total cost units consumed so far
    consumed: u32,
    /// The max number of cost units that can be consumed
    limit: u32,
    /// The cost breakdown
    cost_breakdown: HashMap<String, u32>,
}

impl UnlimitedLoanFeeReserve {
    pub fn new(limit: u32, tip_percentage: u32, cost_unit_price: Decimal) -> Self {
        Self {
            cost_unit_price,
            tip_percentage,
            consumed: 0,
            limit: limit,
            cost_breakdown: HashMap::new(),
        }
    }
}

impl FeeReserve for UnlimitedLoanFeeReserve {
    fn consume<T: ToString>(&mut self, n: u32, reason: T) -> Result<(), FeeReserveError> {
        self.consumed = self
            .consumed
            .checked_add(n)
            .ok_or(FeeReserveError::Overflow)?;

        self.cost_breakdown
            .entry(reason.to_string())
            .or_default()
            .add_assign(n);

        Ok(())
    }

    fn repay(
        &mut self,
        _vault_id: VaultId,
        fee: ResourceContainer,
        _contingent: bool,
    ) -> Result<ResourceContainer, FeeReserveError> {
        Ok(fee) // No-op
    }

    fn finalize(self) -> FeeSummary {
        FeeSummary {
            loan_fully_repaid: true,
            cost_unit_limit: self.limit,
            cost_unit_consumed: self.consumed,
            cost_unit_price: self.cost_unit_price,
            tip_percentage: self.tip_percentage,
            burned: self.cost_unit_price * self.consumed,
            tipped: self.cost_unit_price * self.tip_percentage / 100 * self.consumed,
            payments: Vec::new(),
            cost_breakdown: self.cost_breakdown,
        }
    }

    fn limit(&self) -> u32 {
        self.limit
    }

    fn consumed(&self) -> u32 {
        self.consumed
    }

    fn balance(&self) -> u32 {
        u32::MAX
    }

    fn owed(&self) -> u32 {
        0
    }
}

impl Default for UnlimitedLoanFeeReserve {
    fn default() -> UnlimitedLoanFeeReserve {
        UnlimitedLoanFeeReserve::new(
            DEFAULT_COST_UNIT_LIMIT,
            0,
            DEFAULT_COST_UNIT_PRICE
                .parse()
                .expect("Invalid DEFAULT_COST_UNIT_PRICE"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrypto::{crypto::Hash, prelude::RADIX_TOKEN};

    const TEST_VAULT_ID: VaultId = (Hash([0u8; 32]), 1);

    fn xrd<T: Into<Decimal>>(amount: T) -> ResourceContainer {
        ResourceContainer::new_fungible(RADIX_TOKEN, 18, amount.into())
    }

    #[test]
    fn test_consume_and_repay() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 0, 1.into(), 5);
        fee_reserve.consume(2, "test").unwrap();
        fee_reserve.repay(TEST_VAULT_ID, xrd(3), false).unwrap();
        assert_eq!(3, fee_reserve.balance());
        assert_eq!(2, fee_reserve.consumed());
        assert_eq!(2, fee_reserve.owed());
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 0, 1.into(), 5);
        assert_eq!(
            Err(FeeReserveError::OutOfCostUnit),
            fee_reserve.consume(6, "test")
        );
    }

    #[test]
    fn test_overflow() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 0, 1.into(), 0);
        assert_eq!(
            Ok(xrd(0)),
            fee_reserve.repay(TEST_VAULT_ID, xrd(u32::max_value()), false)
        );
        assert_eq!(
            Err(FeeReserveError::Overflow),
            fee_reserve.repay(TEST_VAULT_ID, xrd(1), false)
        );
    }

    #[test]
    fn test_repay() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 0, 1.into(), 500);
        fee_reserve.repay(TEST_VAULT_ID, xrd(100), false).unwrap();
        assert_eq!(500, fee_reserve.balance());
        assert_eq!(400, fee_reserve.owed());
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut fee_reserve = SystemLoanFeeReserve::new(100, 0, 5.into(), 500);
        fee_reserve.repay(TEST_VAULT_ID, xrd(100), false).unwrap();
        assert_eq!(500, fee_reserve.balance());
        assert_eq!(500 - 100 / 5, fee_reserve.owed());
        assert_eq!(
            vec![(TEST_VAULT_ID, xrd(100), false)],
            fee_reserve.finalize().payments
        )
    }
}
