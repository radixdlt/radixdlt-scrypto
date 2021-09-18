use scrypto::types::*;

use crate::model::*;

/// A ledger stores all the transactions and substates.
pub trait Ledger {
    fn get_package(&self, address: Address) -> Option<Package>;

    fn put_package(&mut self, address: Address, package: Package);

    fn get_resource(&self, address: Address) -> Option<Resource>;

    fn put_resource(&mut self, address: Address, info: Resource);

    fn get_component(&self, address: Address) -> Option<Component>;

    fn put_component(&mut self, address: Address, component: Component);

    fn get_storage(&self, sid: SID) -> Option<Storage>;

    fn put_storage(&mut self, sid: SID, storage: Storage);

    // For now, we always read/write everything in a vault.

    fn get_vault(&self, vid: VID) -> Option<Vault>;

    fn put_vault(&mut self, vid: VID, bucket: Vault);
}
