use std::collections::{BTreeMap, BTreeSet};
use radix_engine_common::prelude::RESOURCE_PACKAGE;
use radix_engine_common::types::{NodeId, ResourceAddress};
use radix_engine_interface::api::FieldIndex;
use radix_engine_interface::blueprints::resource::{FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT};
use radix_engine_interface::prelude::BlueprintInfo;
use radix_engine_interface::types::BlueprintId;
use crate::system::system_db_checker::ApplicationChecker;

pub struct ResourceChecker {
    resources: BTreeSet<ResourceAddress>,
}

impl ApplicationChecker for ResourceChecker {
    fn on_field(&mut self, info: BlueprintInfo, node_id: NodeId, field_index: FieldIndex, value: &Vec<u8>) {
        if info.blueprint_id.pac.eq(&BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)) {
            self.resources.insert(ResourceAddress::new_or_panic(node_id.0))
        }
        if info.blueprint_id.pac.eq(&BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)) {
            self.resources.insert(ResourceAddress::new_or_panic(node_id.0))
        }
    }
}
