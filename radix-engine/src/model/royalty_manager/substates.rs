use crate::model::*;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoyaltyManagerSubstate {
    pub balance: Resource,
}

impl RoyaltyManagerSubstate {
    pub fn take(&mut self, amount: Decimal) -> Result<Resource, ResourceOperationError> {
        self.balance.take_by_amount(amount)
    }

    pub fn put(&mut self, resource: Resource) -> Result<(), ResourceOperationError> {
        self.balance.put(resource)
    }
}
