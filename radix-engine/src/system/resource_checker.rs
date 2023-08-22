use std::collections::BTreeMap;
use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::{RESOURCE_PACKAGE, scrypto_decode};
use radix_engine_common::types::{NodeId, ResourceAddress};
use radix_engine_interface::api::FieldIndex;
use radix_engine_interface::blueprints::resource::{FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT};
use radix_engine_interface::prelude::BlueprintInfo;
use radix_engine_interface::types::BlueprintId;
use crate::blueprints::resource::{FungibleResourceManagerTotalSupplySubstate, NonFungibleResourceManagerTotalSupplySubstate};
use crate::system::system_db_checker::ApplicationChecker;

#[derive(Debug, Default)]
pub struct ResourceTracker {
    total_supply: Decimal,
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

    fn on_field(&mut self, info: BlueprintInfo, node_id: NodeId, field_index: FieldIndex, value: &Vec<u8>) {
        if info.blueprint_id.eq(&BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT))
            && field_index.eq(&1u8) {
            let total_supply: FungibleResourceManagerTotalSupplySubstate = scrypto_decode(value).unwrap();
            self.resources.insert(
                ResourceAddress::new_or_panic(node_id.0),
                ResourceTracker {
                    total_supply
                },
            );
        }

        if info.blueprint_id.eq(&BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT))
            && field_index.eq(&2u8) {

            let total_supply: NonFungibleResourceManagerTotalSupplySubstate = scrypto_decode(value).unwrap();

            self.resources.insert(
                ResourceAddress::new_or_panic(node_id.0),
                ResourceTracker {
                    total_supply
                },
            );
        }
    }

    fn on_finish(&self) -> Self::ApplicationCheckerResults {
        ResourceCheckerResults {
            num_resources: self.resources.len()
        }
    }
}
