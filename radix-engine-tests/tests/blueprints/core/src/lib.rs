use scrypto::api::key_value_store_api::KeyValueStoreGenericArgs;
use scrypto::api::object_api::ModuleId;
use scrypto::api::LockFlags;
use scrypto::prelude::wasm_api::*;
use scrypto::prelude::*;

#[blueprint]
mod globalize_test {
    struct GlobalizeTest {
        own: Option<Own>,
    }

    impl GlobalizeTest {
        pub fn globalize(x: Own) {
            let modules = btreemap!(
                ModuleId::Metadata => Metadata::new().0.as_node_id().clone(),
                ModuleId::Royalty => Royalty::new(ComponentRoyaltyConfig::default()).0.as_node_id().clone(),
            );

            ScryptoVmV1Api::object_globalize(x.0, modules, None);
        }

        pub fn globalize_in_package(package_address: PackageAddress) {
            let x = GlobalizeTest { own: None }.instantiate();

            ScryptoVmV1Api::blueprint_call(
                package_address,
                "GlobalizeTest",
                "globalize",
                scrypto_args!(x),
            );
        }

        pub fn globalize_bucket() {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(100)
                .into();
            Self::globalize(bucket.0);
        }

        pub fn globalize_proof() {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(100)
                .into();
            let proof = bucket.create_proof_of_all();
            Self::globalize(proof.0);
        }

        pub fn globalize_vault() {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(100)
                .into();
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
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(100)
                .into();
            Self::store(bucket.0);
        }

        pub fn store_proof() {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(100)
                .into();
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

            ScryptoVmV1Api::blueprint_call(package_address, "DropTest", "drop", scrypto_args!(x));
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
                .into()
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
mod recursive_test {
    struct RecursiveTest {
        own: Own,
    }

    impl RecursiveTest {
        pub fn create_own_at_depth(depth: u32) {
            // Can be further optimized by pre-computation
            let schema = scrypto_encode(&KeyValueStoreGenericArgs::new::<u32, Own>(true)).unwrap();
            let key_payload = scrypto_encode(&0u32).unwrap();
            let mut value_payload = scrypto_encode(&Own(NodeId([0u8; NodeId::LENGTH]))).unwrap();

            fn create_kv_store(schema: &[u8]) -> NodeId {
                let bytes =
                    copy_buffer(unsafe { kv_store::kv_store_new(schema.as_ptr(), schema.len()) });
                NodeId(bytes[bytes.len() - NodeId::LENGTH..].try_into().unwrap())
            }

            fn move_kv_store(
                store: NodeId,
                to: NodeId,
                key_payload: &[u8],
                value_payload: &mut [u8],
            ) {
                unsafe {
                    let handle = kv_store::kv_store_open_entry(
                        to.as_ref().as_ptr(),
                        to.as_ref().len(),
                        key_payload.as_ptr(),
                        key_payload.len(),
                        LockFlags::MUTABLE.bits(),
                    );

                    let len = value_payload.len();
                    value_payload[len - NodeId::LENGTH..].copy_from_slice(store.as_bytes());

                    kv_entry::kv_entry_write(handle, value_payload.as_ptr(), value_payload.len());
                    kv_entry::kv_entry_close(handle);
                }
            }

            let mut root = create_kv_store(&schema);
            for _ in 0..depth {
                let store = create_kv_store(&schema);
                move_kv_store(root, store, &key_payload, &mut value_payload);
                root = store;
            }

            Self { own: Own(root) }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }
    }
}
