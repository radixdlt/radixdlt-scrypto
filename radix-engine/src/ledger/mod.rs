mod file;
mod memory;

pub use file::FileBasedLedger;
pub use memory::InMemoryLedger;

use scrypto::kernel::ResourceInfo;
use scrypto::types::*;

use crate::model::*;

pub trait Ledger {
    fn get_blueprint(&self, address: Address) -> Option<Vec<u8>>;

    fn put_blueprint(&mut self, address: Address, blueprint: Vec<u8>);

    fn get_resource(&self, address: Address) -> Option<ResourceInfo>;

    fn put_resource(&mut self, address: Address, info: ResourceInfo);

    fn get_component(&self, address: Address) -> Option<Component>;

    fn put_component(&mut self, address: Address, component: Component);

    fn get_account(&self, address: Address) -> Option<Account>;

    fn put_account(&mut self, address: Address, account: Account);
}
