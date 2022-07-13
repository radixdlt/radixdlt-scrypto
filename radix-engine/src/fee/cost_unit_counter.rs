use core::ops::AddAssign;

use sbor::rust::collections::HashMap;

pub struct CostUnitCounter {
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
    pub analysis: HashMap<&'static str, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostUnitCounterError {
    OutOfCostUnit,
    CounterOverflow,
    LimitExceeded,
    SystemLoanNotCleared,
}

impl CostUnitCounter {
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

    pub fn consume(&mut self, n: u32, reason: &'static str) -> Result<(), CostUnitCounterError> {
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

    pub fn repay(&mut self, n: u32) -> Result<(), CostUnitCounterError> {
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

    pub fn limit(&self) -> u32 {
        self.limit
    }

    pub fn consumed(&self) -> u32 {
        self.consumed
    }

    pub fn balance(&self) -> u32 {
        self.balance
    }

    pub fn owed(&self) -> u32 {
        self.owed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_and_repay() {
        let mut counter = CostUnitCounter::new(100, 5);
        counter.consume(2, "test").unwrap();
        counter.repay(3).unwrap();
        assert_eq!(3, counter.balance());
        assert_eq!(2, counter.consumed());
        assert_eq!(2, counter.owed());
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut counter = CostUnitCounter::new(100, 5);
        assert_eq!(
            Err(CostUnitCounterError::OutOfCostUnit),
            counter.consume(6, "test")
        );
    }

    #[test]
    fn test_overflow() {
        let mut counter = CostUnitCounter::new(100, 0);
        assert_eq!(Ok(()), counter.repay(u32::max_value()));
        assert_eq!(Err(CostUnitCounterError::CounterOverflow), counter.repay(1));
    }

    #[test]
    fn test_repay() {
        let mut counter = CostUnitCounter::new(100, 500);
        counter.repay(100).unwrap();
        assert_eq!(500, counter.balance());
        assert_eq!(400, counter.owed());
    }
}
