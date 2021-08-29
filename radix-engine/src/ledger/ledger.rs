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

    // For now, we always read/write everything in a resource bucket.

    fn get_bucket(&self, bid: BID) -> Option<PersistedBucket>;

    fn put_bucket(&mut self, bid: BID, bucket: PersistedBucket);
}
