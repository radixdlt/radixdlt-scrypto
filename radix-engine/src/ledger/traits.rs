use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
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

    fn get_vault(&self, vid: VID) -> Option<Vault>;

    fn put_vault(&mut self, vid: VID, bucket: Vault);

    fn bootstrap(&mut self) {
        if self.get_package(SYSTEM_PACKAGE).is_none() {
            // System package
            self.put_package(
                SYSTEM_PACKAGE,
                Package::new(include_bytes!("../../../assets/system.wasm").to_vec()),
            );

            // Account package
            self.put_package(
                ACCOUNT_PACKAGE,
                Package::new(include_bytes!("../../../assets/account.wasm").to_vec()),
            );

            // XRD resource
            let mut metadata = HashMap::new();
            metadata.insert("symbol".to_owned(), "xrd".to_owned());
            metadata.insert("name".to_owned(), "Radix".to_owned());
            metadata.insert("description".to_owned(), "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.".to_owned());
            metadata.insert("url".to_owned(), "https://tokens.radixdlt.com".to_owned());
            self.put_resource_def(
                RADIX_TOKEN,
                ResourceDef {
                    metadata,
                    minter: Some(SYSTEM_PACKAGE),
                    auth: Some(SYSTEM_PACKAGE),
                    supply: 0.into(),
                },
            );
        }
    }
}
