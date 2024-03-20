use crate::system::checkers::{ResourceDatabaseCheckerResults, ResourceEventCheckerResults};
use radix_common::math::Decimal;
use radix_common::prelude::{NodeId, ResourceAddress};
use sbor::rust::collections::BTreeMap;

#[derive(Debug)]
pub enum ResourceReconciliationError {
    TotalSuppliesDontMatch(
        BTreeMap<ResourceAddress, Decimal>,
        BTreeMap<ResourceAddress, Decimal>,
    ),
    VaultAmountsDontMatch(BTreeMap<NodeId, Decimal>, BTreeMap<NodeId, Decimal>),
}

pub struct ResourceReconciler;

impl ResourceReconciler {
    pub fn reconcile(
        db_results: &ResourceDatabaseCheckerResults,
        event_results: &ResourceEventCheckerResults,
    ) -> Result<(), ResourceReconciliationError> {
        let mut db_total_supplies = db_results.total_supply.clone();
        db_total_supplies.retain(|_, total_supply| total_supply.is_positive());

        let mut event_total_supplies = event_results.total_supply.clone();
        event_total_supplies.retain(|_, total_supply| total_supply.is_positive());

        if db_total_supplies.ne(&event_total_supplies) {
            return Err(ResourceReconciliationError::TotalSuppliesDontMatch(
                db_total_supplies,
                event_total_supplies,
            ));
        }

        let mut db_vault_amounts = db_results.vaults.clone();
        db_vault_amounts.retain(|_, amount| amount.is_positive());

        let mut event_vault_amounts = event_results.vault_amounts.clone();
        event_vault_amounts.retain(|_, amount| amount.is_positive());

        if db_vault_amounts.ne(&event_vault_amounts) {
            return Err(ResourceReconciliationError::VaultAmountsDontMatch(
                db_vault_amounts,
                event_vault_amounts,
            ));
        }

        Ok(())
    }
}
