use scrypto::api::object_api::ObjectModuleId;
use scrypto::api::ClientBlueprintApi;
use scrypto::api::ClientObjectApi;
use scrypto::prelude::*;

#[blueprint]
mod globalize_test {
    struct GlobalizeTest {
        own: Option<Own>,
    }

    impl GlobalizeTest {
        pub fn globalize(x: Own) {
            let modules = btreemap!(
                ObjectModuleId::Main => x.0,
                ObjectModuleId::Metadata => Metadata::new().0.as_node_id().clone(),
                ObjectModuleId::Royalty => Royalty::new(ComponentRoyaltyConfig::default()).0.as_node_id().clone(),
            );

            let _ = ScryptoEnv.globalize(modules, None).unwrap();
        }

        pub fn globalize_in_package(package_address: PackageAddress) {
            let x = GlobalizeTest { own: None }.instantiate();

            ScryptoEnv
                .call_function(
                    package_address,
                    "GlobalizeTest",
                    "globalize",
                    scrypto_args!(x),
                )
                .unwrap();
        }

        pub fn globalize_bucket() {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);
            Self::globalize(bucket.0);
        }

        pub fn globalize_proof() {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);
            let proof = bucket.create_proof_of_all();
            Self::globalize(proof.0);
        }

        pub fn globalize_vault() {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);
            let vault = Vault::with_bucket(bucket);
            Self::globalize(vault.0);
        }

        pub fn globalize_metadata() {
            let metadata = Metadata::new().0.as_node_id().clone();
            Self::globalize(Own(metadata));
        }

        pub fn globalize_royalty() {
            let royalty = Royalty::new(ComponentRoyaltyConfig::default())
                .0
                .as_node_id()
                .clone();
            Self::globalize(Own(royalty));
        }

        pub fn globalize_role_assignment() {
            let ra = RoleAssignment::new(OwnerRole::None, btreemap!())
                .0
                .as_node_id()
                .clone();
            Self::globalize(Own(ra));
        }

        pub fn store(x: Own) {
            Self { own: Some(x) }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        pub fn store_bucket() {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);
            Self::store(bucket.0);
        }

        pub fn store_proof() {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);
            let proof = bucket.create_proof_of_all();
            Self::store(proof.0);
        }

        pub fn store_metadata() {
            let metadata = Metadata::new().0.as_node_id().clone();
            Self::store(Own(metadata));
        }

        pub fn store_royalty() {
            let royalty = Royalty::new(ComponentRoyaltyConfig::default())
                .0
                .as_node_id()
                .clone();
            Self::store(Own(royalty));
        }

        pub fn store_role_assignment() {
            let ra = RoleAssignment::new(OwnerRole::None, btreemap!())
                .0
                .as_node_id()
                .clone();
            Self::store(Own(ra));
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
