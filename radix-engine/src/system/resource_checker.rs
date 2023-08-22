use radix_engine_common::prelude::RESOURCE_PACKAGE;
use radix_engine_interface::api::FieldIndex;
use radix_engine_interface::blueprints::resource::FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT;
use radix_engine_interface::prelude::BlueprintInfo;
use radix_engine_interface::types::BlueprintId;
use crate::system::system_db_checker::ApplicationChecker;

struct ResourceChecker;

impl ApplicationChecker for ResourceChecker {
    fn on_field(&mut self, info: BlueprintInfo, field_index: FieldIndex, value: &Vec<u8>) {
        if info.blueprint_id.eq(&BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)) {
            
        }
    }
}

impl ResourceChecker {

}