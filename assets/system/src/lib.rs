use scrypto::prelude::*;
use scrypto::engine::{api::*, call_engine};

blueprint! {
    // nobody can instantiate a system component except the bootstrap process
    struct System {
        xrd: Vault,
    }

    impl System {
        /// Creates a resource.
        pub fn new_resource(
            resource_type: ResourceType,
            metadata: HashMap<String, String>,
            access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
            initial_supply: Option<MintParams>,
        ) -> (ResourceAddress, Option<Bucket>) {
            resource_system().new_resource(resource_type, metadata, access_rules, initial_supply)
        }

        /// Mints fungible resource. TODO: Remove
        pub fn mint(amount: Decimal, resource_address: ResourceAddress) -> Bucket {
            borrow_resource_manager!(resource_address).mint(amount)
        }

        /// Burns bucket. TODO: Remove
        pub fn burn(bucket: Bucket) {
            bucket.burn()
        }

        /// Gives away XRD tokens for testing. TODO: Remove
        pub fn free_xrd(&mut self) -> Bucket {
            self.xrd.take(1_000_000)
        }

        // TODO: Remove
        pub fn set_epoch(epoch: u64) {
            let input = RadixEngineInput::InvokeSNode(
                SNodeRef::SystemRef,
                "set_epoch".to_string(),
                scrypto_encode(&SystemSetEpochInput { epoch }),
            );
            call_engine(input)
        }
    }
}
