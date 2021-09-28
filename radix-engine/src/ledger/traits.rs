use scrypto::types::*;

use crate::model::*;

/// A ledger stores all transactions and substates.
pub trait Ledger {
    fn get_resource_def(&self, address: Address) -> Option<ResourceDef>;

    fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef);

    fn get_package(&self, address: Address) -> Option<Package>;

    fn put_package(&mut self, address: Address, package: Package);

    fn get_component(&self, address: Address) -> Option<Component>;

    fn put_component(&mut self, address: Address, component: Component);

    fn get_lazy_map(&self, mid: MID) -> Option<LazyMap>;

    fn put_lazy_map(&mut self, mid: MID, lazy_map: LazyMap);

    // For now, we always read/write everything in a vault.

    fn get_vault(&self, vid: VID) -> Option<Vault>;

    fn put_vault(&mut self, vid: VID, bucket: Vault);
}
