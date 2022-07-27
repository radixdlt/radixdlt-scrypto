use core::ops::AddAssign;

use crate::constants::{DEFAULT_COST_UNIT_LIMIT, DEFAULT_SYSTEM_LOAN};
use sbor::rust::collections::HashMap;
use scrypto::{engine::types::VaultId, math::Decimal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostUnitCounterError {
    OutOfCostUnit,
    CounterOverflow,
    LimitExceeded,
    SystemLoanNotCleared,
}

pub trait CostUnitCounter {
    fn consume(&mut self, n: u32, reason: &'static str) -> Result<(), CostUnitCounterError>;

    fn repay(&mut self, vault_id: VaultId, n: u32) -> Result<(), CostUnitCounterError>;

    fn limit(&self) -> u32;

    fn consumed(&self) -> u32;

    fn balance(&self) -> u32;

    fn owed(&self) -> u32;

    fn payments(&self) -> Vec<(VaultId, Decimal)>;

    fn analysis(&self) -> &HashMap<&'static str, u32>;
}

pub struct SystemLoanCostUnitCounter {
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
    pub fn new(limit: u32, loan: u32) -> Self {
        Self {
            balance: loan,
            owed: loan,
            consumed: 0,
            limit,
            check_point: loan,
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

    fn repay(&mut self, _vault_id: VaultId, n: u32) -> Result<(), CostUnitCounterError> {
        if n >= self.owed {
            self.balance = self
                .balance
                .checked_add(n - self.owed)
                .ok_or(CostUnitCounterError::CounterOverflow)?;
            self.owed = 0;
        } else {
            self.owed -= n;
        }
        Ok(())
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

    fn payments(&self) -> Vec<(VaultId, Decimal)> {
        Vec::new()
    }
}

impl Default for SystemLoanCostUnitCounter {
    fn default() -> SystemLoanCostUnitCounter {
        SystemLoanCostUnitCounter::new(DEFAULT_COST_UNIT_LIMIT, DEFAULT_SYSTEM_LOAN)
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

    fn repay(&mut self, _vault_id: VaultId, _n: u32) -> Result<(), CostUnitCounterError> {
        Ok(()) // No-op
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

    fn payments(&self) -> Vec<(VaultId, Decimal)> {
        Vec::new()
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

    #[test]
    fn test_consume_and_repay() {
        let mut counter = SystemLoanCostUnitCounter::new(100, 5);
        counter.consume(2, "test").unwrap();
        counter.repay((Hash([0u8; 32]), 1), 3).unwrap();
        assert_eq!(3, counter.balance());
        assert_eq!(2, counter.consumed());
        assert_eq!(2, counter.owed());
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut counter = SystemLoanCostUnitCounter::new(100, 5);
        assert_eq!(
            Err(CostUnitCounterError::OutOfCostUnit),
            counter.consume(6, "test")
        );
    }

    #[test]
    fn test_overflow() {
        let mut counter = SystemLoanCostUnitCounter::new(100, 0);
        assert_eq!(
            Ok(()),
            counter.repay((Hash([0u8; 32]), 1), u32::max_value())
        );
        assert_eq!(
            Err(CostUnitCounterError::CounterOverflow),
            counter.repay((Hash([0u8; 32]), 1), 1)
        );
    }

    #[test]
    fn test_repay() {
        let mut counter = SystemLoanCostUnitCounter::new(100, 500);
        counter.repay((Hash([0u8; 32]), 1), 100).unwrap();
        assert_eq!(500, counter.balance());
        assert_eq!(400, counter.owed());
    }
}
