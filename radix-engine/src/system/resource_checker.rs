use crate::blueprints::resource::{
    FungibleResourceManagerTotalSupplySubstate, FungibleVaultBalanceSubstate,
    NonFungibleResourceManagerTotalSupplySubstate, NonFungibleVaultBalanceSubstate,
};
use crate::system::system_db_checker::ApplicationChecker;
use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::{scrypto_decode, RESOURCE_PACKAGE};
use radix_engine_common::types::{NodeId, ResourceAddress};
use radix_engine_interface::api::FieldIndex;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_VAULT_BLUEPRINT,
    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, NON_FUNGIBLE_VAULT_BLUEPRINT,
};
use radix_engine_interface::prelude::BlueprintInfo;
use radix_engine_interface::prelude::SafeAdd;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct ResourceTracker {
    total_supply: Option<Decimal>,
    tracking_supply: Decimal,
}

#[derive(Debug, Default)]
pub struct ResourceChecker {
    resources: BTreeMap<ResourceAddress, ResourceTracker>,
}

#[derive(Debug, Default)]
pub struct ResourceCheckerResults {
    pub num_resources: usize,
}

impl ApplicationChecker for ResourceChecker {
    type ApplicationCheckerResults = ResourceCheckerResults;

    fn on_field(
        &mut self,
        info: BlueprintInfo,
        node_id: NodeId,
        field_index: FieldIndex,
        value: &Vec<u8>,
    ) {
        if !info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE) {
            return;
        }

        if info
            .blueprint_id
            .blueprint_name
            .eq(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)
            && field_index.eq(&1u8)
        {
            let total_supply: FungibleResourceManagerTotalSupplySubstate =
                scrypto_decode(value).unwrap();
            let address = ResourceAddress::new_or_panic(node_id.0);
            let tracker = self.resources.entry(address).or_default();
            tracker.total_supply = Some(total_supply);
        }

        if info
            .blueprint_id
            .blueprint_name
            .eq(FUNGIBLE_VAULT_BLUEPRINT)
            && field_index.eq(&0u8)
        {
            let vault_balance: FungibleVaultBalanceSubstate = scrypto_decode(value).unwrap();
            let address =
                ResourceAddress::new_or_panic(info.outer_obj_info.expect().into_node_id().0);
            let tracker = self.resources.entry(address).or_default();
            tracker.tracking_supply = tracker
                .tracking_supply
                .safe_add(vault_balance.amount())
                .unwrap();
        }

        if info
            .blueprint_id
            .blueprint_name
            .eq(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)
            && field_index.eq(&2u8)
        {
            let total_supply: NonFungibleResourceManagerTotalSupplySubstate =
                scrypto_decode(value).unwrap();
            let address = ResourceAddress::new_or_panic(node_id.0);
            let tracker = self.resources.entry(address).or_default();
            tracker.total_supply = Some(total_supply);
        }

        if info
            .blueprint_id
            .blueprint_name
            .eq(NON_FUNGIBLE_VAULT_BLUEPRINT)
            && field_index.eq(&0u8)
        {
            let vault_balance: NonFungibleVaultBalanceSubstate = scrypto_decode(value).unwrap();
            let address =
                ResourceAddress::new_or_panic(info.outer_obj_info.expect().into_node_id().0);
            let tracker = self.resources.entry(address).or_default();
            tracker.tracking_supply = tracker
                .tracking_supply
                .safe_add(vault_balance.amount)
                .unwrap();
        }
    }

    fn on_finish(&self) -> Self::ApplicationCheckerResults {
        for (address, tracker) in &self.resources {
            if let Some(total_supply) = tracker.total_supply {
                if !total_supply.eq(&tracker.tracking_supply) {
                    panic!(
                        "Total Supply mismatch: {:?} total_supply: {:?} tracked_supply: {:?}",
                        address, total_supply, tracker.tracking_supply,
                    );
                }
            }
        }

        ResourceCheckerResults {
            num_resources: self.resources.len(),
        }
    }
}
