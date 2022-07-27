use core::ops::AddAssign;
use sbor::rust::collections::HashMap;
use sbor::rust::str::FromStr;
use sbor::rust::vec::Vec;
use scrypto::{
    engine::types::VaultId,
    math::{Decimal, RoundingMode},
};

use crate::constants::{DEFAULT_COST_UNIT_LIMIT, DEFAULT_COST_UNIT_PRICE, DEFAULT_SYSTEM_LOAN};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostUnitCounterError {
    OutOfCostUnit,
    CounterOverflow,
    LimitExceeded,
    SystemLoanNotCleared,
}

pub trait CostUnitCounter {
    fn consume(&mut self, n: u32, reason: &'static str) -> Result<(), CostUnitCounterError>;

    fn repay(
        &mut self,
        vault_id: VaultId,
        amount: Decimal,
    ) -> Result<Decimal, CostUnitCounterError>;

    fn limit(&self) -> u32;

    fn consumed(&self) -> u32;

    fn balance(&self) -> u32;

    fn owed(&self) -> u32;

    fn payments(&self) -> &[(VaultId, u32)];

    fn analysis(&self) -> &HashMap<&'static str, u32>;
}

pub struct SystemLoanCostUnitCounter {
    /// The price of cost unit
    cost_unit_price: Decimal,
    /// Payments made during the execution of a transaction.
    payments: Vec<(VaultId, u32)>,
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
    /// Costing analysis
    analysis: HashMap<&'static str, u32>,
}

impl SystemLoanCostUnitCounter {
    pub fn new(cost_unit_price: Decimal, cost_unit_limit: u32, system_loan: u32) -> Self {
        Self {
            cost_unit_price,
            payments: Vec::new(),
            balance: system_loan,
            owed: system_loan,
            consumed: 0,
            limit: cost_unit_limit,
            check_point: system_loan,
            analysis: HashMap::new(),
        }
    }
}

impl CostUnitCounter for SystemLoanCostUnitCounter {
    fn consume(&mut self, n: u32, reason: &'static str) -> Result<(), CostUnitCounterError> {
        self.balance = self
            .balance
            .checked_sub(n)
            .ok_or(CostUnitCounterError::OutOfCostUnit)?;
        self.consumed = self
            .consumed
            .checked_add(n)
            .ok_or(CostUnitCounterError::CounterOverflow)?;

        self.analysis.entry(reason).or_default().add_assign(n);

        if self.consumed > self.limit {
            return Err(CostUnitCounterError::LimitExceeded);
        }
        if self.consumed >= self.check_point && self.owed > 0 {
            return Err(CostUnitCounterError::SystemLoanNotCleared);
        }
        Ok(())
    }

    fn repay(
        &mut self,
        vault_id: VaultId,
        amount: Decimal,
    ) -> Result<Decimal, CostUnitCounterError> {
        // TODO: Add `TryInto` implementation once the new decimal types are in place
        let n = u32::from_str(
            (amount / self.cost_unit_price)
                .round(0, RoundingMode::TowardsZero)
                .to_string()
                .as_str(),
        )
        .map_err(|_| CostUnitCounterError::CounterOverflow)?;

        if n >= self.owed {
            self.balance = self
                .balance
                .checked_add(n - self.owed)
                .ok_or(CostUnitCounterError::CounterOverflow)?;
            self.owed = 0;
        } else {
            self.owed -= n;
        }

        self.payments.push((vault_id, n));

        Ok(self.cost_unit_price * n)
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

    fn analysis(&self) -> &HashMap<&'static str, u32> {
        &self.analysis
    }

    fn payments(&self) -> &[(VaultId, u32)] {
        &self.payments
    }
}

impl Default for SystemLoanCostUnitCounter {
    fn default() -> SystemLoanCostUnitCounter {
        SystemLoanCostUnitCounter::new(
            DEFAULT_COST_UNIT_PRICE.parse().unwrap(),
            DEFAULT_COST_UNIT_LIMIT,
            DEFAULT_SYSTEM_LOAN,
        )
    }
}

pub struct UnlimitedLoanCostUnitCounter {
    /// The total cost units consumed so far
    consumed: u32,
    /// The max number of cost units that can be consumed
    limit: u32,
    /// Costing analysis
    pub analysis: HashMap<&'static str, u32>,
}

impl UnlimitedLoanCostUnitCounter {
    pub fn new(limit: u32) -> Self {
        Self {
            consumed: 0,
            limit: limit,
            analysis: HashMap::new(),
        }
    }
}

impl CostUnitCounter for UnlimitedLoanCostUnitCounter {
    fn consume(&mut self, n: u32, reason: &'static str) -> Result<(), CostUnitCounterError> {
        self.consumed = self
            .consumed
            .checked_add(n)
            .ok_or(CostUnitCounterError::CounterOverflow)?;

        self.analysis.entry(reason).or_default().add_assign(n);

        Ok(())
    }

    fn repay(
        &mut self,
        _vault_id: VaultId,
        amount: Decimal,
    ) -> Result<Decimal, CostUnitCounterError> {
        Ok(amount) // No-op
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

    fn payments(&self) -> &[(VaultId, u32)] {
        &[]
    }

    fn analysis(&self) -> &HashMap<&'static str, u32> {
        &self.analysis
    }
}

impl Default for UnlimitedLoanCostUnitCounter {
    fn default() -> UnlimitedLoanCostUnitCounter {
        UnlimitedLoanCostUnitCounter::new(DEFAULT_COST_UNIT_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrypto::crypto::Hash;

    const TEST_VAULT_ID: VaultId = (Hash([0u8; 32]), 1);

    #[test]
    fn test_consume_and_repay() {
        let mut counter = SystemLoanCostUnitCounter::new(1.into(), 100, 5);
        counter.consume(2, "test").unwrap();
        counter.repay(TEST_VAULT_ID, 3.into()).unwrap();
        assert_eq!(3, counter.balance());
        assert_eq!(2, counter.consumed());
        assert_eq!(2, counter.owed());
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut counter = SystemLoanCostUnitCounter::new(1.into(), 100, 5);
        assert_eq!(
            Err(CostUnitCounterError::OutOfCostUnit),
            counter.consume(6, "test")
        );
    }

    #[test]
    fn test_overflow() {
        let mut counter = SystemLoanCostUnitCounter::new(1.into(), 100, 0);
        assert_eq!(
            Ok(u32::max_value().into()),
            counter.repay(TEST_VAULT_ID, u32::max_value().into())
        );
        assert_eq!(
            Err(CostUnitCounterError::CounterOverflow),
            counter.repay(TEST_VAULT_ID, 1.into())
        );
    }

    #[test]
    fn test_repay() {
        let mut counter = SystemLoanCostUnitCounter::new(1.into(), 100, 500);
        counter.repay(TEST_VAULT_ID, 100.into()).unwrap();
        assert_eq!(500, counter.balance());
        assert_eq!(400, counter.owed());
    }

    #[test]
    fn test_xrd_cost_unit_conversion() {
        let mut counter = SystemLoanCostUnitCounter::new(5.into(), 100, 500);
        counter.repay(TEST_VAULT_ID, 100.into()).unwrap();
        assert_eq!(500, counter.balance());
        assert_eq!(500 - 100 / 5, counter.owed());
        assert_eq!(vec![(TEST_VAULT_ID, 20)], counter.payments())
    }
}
