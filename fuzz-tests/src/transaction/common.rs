use radix_engine::types::*;
use radix_engine_store_interface::db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper};
use radix_engine_store_interface::interface::ListableSubstateDatabase;
use radix_engine_store_interface::interface::SubstateDatabase;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;

pub fn get_ledger_entries(
    substate_db: &InMemorySubstateDatabase,
) -> (
    Vec<PackageAddress>,
    Vec<ComponentAddress>,
    Vec<ResourceAddress>,
    Vec<ResourceAddress>,
) {
    let mut packages: Vec<PackageAddress> = vec![];
    let mut components: Vec<ComponentAddress> = vec![];
    let mut resources_fungible: Vec<ResourceAddress> = vec![];
    let mut resources_non_fungible: Vec<ResourceAddress> = vec![];

    for key in substate_db.list_partition_keys() {
        let _entries = substate_db.list_entries(&key);
        let (node_id, _) = SpreadPrefixKeyMapper::from_db_partition_key(&key);
        if let Ok(address) = PackageAddress::try_from(node_id.as_ref()) {
            if !packages.contains(&address) {
                packages.push(address);
            }
        } else if let Ok(address) = ComponentAddress::try_from(node_id.as_ref()) {
            if !components.contains(&address) {
                components.push(address);
            }
        } else if let Ok(address) = ResourceAddress::try_from(node_id.as_ref()) {
            let resources = {
                if address.as_node_id().is_global_fungible_resource_manager() {
                    &mut resources_fungible
                } else {
                    &mut resources_non_fungible
                }
            };

            if !resources.contains(&address) {
                resources.push(address);
            }
        }
    }

    (
        packages,
        components,
        resources_fungible,
        resources_non_fungible,
    )
}
