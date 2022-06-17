pub struct CostUnitCounter {
    limit: u32,
    loan: u32,
    consumed: u32,
    remaining: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostUnitCounterError {
    OutOfCostUnit,
    CounterOverflow,
    LimitExceeded,
}

impl CostUnitCounter {
    pub fn new(limit: u32, loan: u32) -> Self {
        Self {
            limit,
            loan,
            consumed: 0,
            remaining: loan,
        }
    }

    pub fn consume(&mut self, n: u32) -> Result<(), CostUnitCounterError> {
        self.remaining = self
            .remaining
            .checked_sub(n)
            .ok_or(CostUnitCounterError::OutOfCostUnit)?;
        self.consumed = self
            .consumed
            .checked_add(n)
            .ok_or(CostUnitCounterError::CounterOverflow)?;
        if self.consumed > self.limit {
            return Err(CostUnitCounterError::LimitExceeded);
        }
        Ok(())
    }

    pub fn refill(&mut self, n: u32) -> Result<(), CostUnitCounterError> {
        self.remaining = self
            .remaining
            .checked_add(n)
            .ok_or(CostUnitCounterError::CounterOverflow)?;
        Ok(())
    }

    pub fn repay(&mut self, n: u32) -> Result<(), CostUnitCounterError> {
        self.remaining = self
            .remaining
            .checked_sub(n)
            .ok_or(CostUnitCounterError::OutOfCostUnit)?;
        self.loan = self
            .loan
            .checked_sub(n)
            .ok_or(CostUnitCounterError::CounterOverflow)?;
        Ok(())
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }

    pub fn consumed(&self) -> u32 {
        self.consumed
    }

    pub fn remaining(&self) -> u32 {
        self.remaining
    }

    pub fn loan(&self) -> u32 {
        self.loan
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_and_refill() {
        let mut counter = CostUnitCounter::new(100, 5);
        counter.consume(2).unwrap();
        counter.refill(3).unwrap();
        assert_eq!(6, counter.remaining());
        assert_eq!(2, counter.consumed());
    }

    #[test]
    fn test_out_of_cost_unit() {
        let mut counter = CostUnitCounter::new(100, 5);
        assert_eq!(Err(CostUnitCounterError::OutOfCostUnit), counter.consume(6));
    }

    #[test]
    fn test_overflow() {
        let mut counter = CostUnitCounter::new(100, u32::max_value());
        counter.consume(1).unwrap();
        assert_eq!(
            Err(CostUnitCounterError::CounterOverflow),
            counter.refill(2)
        );
    }

    #[test]
    fn test_repay() {
        let mut counter = CostUnitCounter::new(100, 500);
        counter.repay(100).unwrap();
        assert_eq!(400, counter.remaining());
        assert_eq!(400, counter.loan());
    }
}
