use scrypto::api::object_api::ObjectModuleId;
use scrypto::api::ClientBlueprintApi;
use scrypto::api::ClientObjectApi;
use scrypto::prelude::*;

#[blueprint]
mod globalize_test {
    struct GlobalizeTest;

    impl GlobalizeTest {
        pub fn globalize_in_package(package_address: PackageAddress) {
            let x = GlobalizeTest {}.instantiate();

            ScryptoEnv
                .call_function(
                    package_address,
                    "GlobalizeTest",
                    "globalize",
                    scrypto_args!(x),
                )
                .unwrap();
        }

        pub fn globalize(x: Own) {
            let modules = btreemap!(
                ObjectModuleId::Main => x.0,
                ObjectModuleId::AccessRules => AccessRules::new(OwnerRole::None, btreemap!()).0.as_node_id().clone(),
                ObjectModuleId::Metadata => Metadata::new().0.as_node_id().clone(),
                ObjectModuleId::Royalty => Royalty::new(ComponentRoyaltyConfig::default()).0.as_node_id().clone(),
            );

            let _ = ScryptoEnv.globalize(modules, None).unwrap();
        }
    }
}

#[blueprint]
mod drop_test {
    struct DropTest;

    impl DropTest {
        pub fn drop_in_package(package_address: PackageAddress) {
            let x = DropTest {}.instantiate();

            ScryptoEnv
                .call_function(package_address, "DropTest", "drop", scrypto_args!(x))
                .unwrap();
        }

        pub fn drop(x: Own) {
            let _ = ScryptoEnv.drop_object(&x.0);
        }
    }
}

#[blueprint]
mod move_test {
    struct MoveTest {
        vaults: Vec<Vault>,
    }

    impl MoveTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(amount)
        }

        pub fn receive_bucket(&mut self, t: Bucket) {
            self.vaults.push(Vault::with_bucket(t));
        }

        pub fn receive_proof(&self, t: Proof) {
            t.drop();
        }

        pub fn move_bucket() {
            let bucket = Self::create_test_token(1000);
            let component = MoveTest { vaults: Vec::new() }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            component.receive_bucket(bucket);
        }

        pub fn move_proof() -> Bucket {
            let bucket = Self::create_test_token(1000);
            let component = MoveTest { vaults: Vec::new() }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            component.receive_proof(bucket.create_proof_of_all());

            bucket
        }
    }
}

#[blueprint]
mod core_test {
    struct CoreTest;

    impl CoreTest {
        pub fn query() -> (PackageAddress, Hash, Epoch) {
            (
                Runtime::package_address(),
                Runtime::transaction_hash(),
                Runtime::current_epoch(),
            )
        }
    }
}
