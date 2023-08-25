use crate::system::checkers::{ResourceDatabaseCheckerResults, ResourceEventCheckerResults};

#[derive(Debug)]
pub enum ResourceReconciliationError {
    TotalSuppliesDontMatch
}

pub struct ResourceReconciliation;

impl ResourceReconciliation {
    pub fn reconcile(db_results: &ResourceDatabaseCheckerResults, event_results: &ResourceEventCheckerResults) -> Result<(), ResourceReconciliationError> {
        let mut db_total_supplies = db_results.total_supply.clone();
        db_total_supplies.retain(|_, total_supply| total_supply.is_positive());

        let mut event_total_supplies = event_results.total_supply.clone();
        event_total_supplies.retain(|_, total_supply| total_supply.is_positive());

        if db_total_supplies.ne(&event_total_supplies) {
            return Err(ResourceReconciliationError::TotalSuppliesDontMatch);
        }

        Ok(())
    }
}