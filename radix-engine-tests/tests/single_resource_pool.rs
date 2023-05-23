use radix_engine::blueprints::pool::single_resource_pool::SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT;
use radix_engine_interface::blueprints::pool::*;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::*;

#[test]
pub fn single_resource_pool_can_be_instantiated() {
    SingleResourcePoolTestRunner::new(18);
}

//===================================
// Test Runner and Utility Functions
//===================================

struct SingleResourcePoolTestRunner {
    test_runner: TestRunner,
    pool_component_address: ComponentAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl SingleResourcePoolTestRunner {
    pub fn new(divisibility: u8) -> Self {
        let mut test_runner = TestRunner::builder().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            None,
            divisibility,
            account,
        );

        let pool_component = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    SINGLE_RESOURCE_POOL_INSTANTIATE_IDENT,
                    to_manifest_value(&SingleResourcePoolInstantiateManifestInput {
                        resource_address,
                        pool_manager_rule: rule!(require(virtual_signature_badge)),
                    }),
                )
                .build();
            let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
            receipt
                .expect_commit_success()
                .new_component_addresses()
                .get(0)
                .unwrap()
                .clone()
        };

        Self {
            test_runner,
            pool_component_address: pool_component,
            resource_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }
}
