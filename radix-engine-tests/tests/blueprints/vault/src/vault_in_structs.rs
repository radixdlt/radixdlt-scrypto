use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct Data {}

#[blueprint]
mod vault_test {
    struct VaultTest {
        vault: Vault,
        vaults: KeyValueStore<u128, Vault>,
        vault_vector: Vec<Vault>,
    }

    impl VaultTest {
        fn new_fungible() -> Bucket {
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into()
        }

        pub fn new_vault_into_map() -> Global<VaultTest> {
            let bucket = Self::new_fungible();
            let vault = Vault::with_bucket(bucket);
            let bucket = Self::new_fungible();
            let vaults = KeyValueStore::new();
            vaults.insert(0, Vault::with_bucket(bucket));
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn invalid_double_ownership_of_vault() -> Global<VaultTest> {
            let bucket = Self::new_fungible();
            let vault = Vault::new(bucket.resource_address());
            let vault_fake_copy = Vault(vault.0.clone());

            VaultTest {
                vault,
                vaults: KeyValueStore::new(),
                vault_vector: vec![vault_fake_copy],
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn new_vault_into_map_then_get() -> Global<VaultTest> {
            let bucket = Self::new_fungible();
            let vault = Vault::new(bucket.resource_address());
            let mut vaults = KeyValueStore::new();
            vaults.insert(0, vault);
            {
                let mut vault = vaults.get_mut(&0).unwrap();
                vault.put(bucket);
            }

            let vault_vector = Vec::new();
            VaultTest {
                vault: Vault::with_bucket(Self::new_fungible()),
                vaults,
                vault_vector,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn overwrite_vault_in_map(&mut self) -> () {
            let bucket = Self::new_fungible();
            self.vaults.insert(0, Vault::with_bucket(bucket))
        }

        pub fn new_vault_into_vector() -> Global<VaultTest> {
            let bucket = Self::new_fungible();
            let vault = Vault::with_bucket(bucket);
            let bucket = Self::new_fungible();
            let vaults = KeyValueStore::new();
            let mut vault_vector = Vec::new();
            vault_vector.push(Vault::with_bucket(bucket));
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn clear_vector(&mut self) -> () {
            self.vault_vector.clear()
        }

        pub fn push_vault_into_vector(&mut self) -> () {
            let bucket = Self::new_fungible();
            self.vault_vector.push(Vault::with_bucket(bucket))
        }

        pub fn new_fungible_vault_with_take() -> Global<VaultTest> {
            let bucket = Self::new_fungible();
            let mut vault = Vault::with_bucket(bucket);
            let bucket = vault.take(1);
            vault.put(bucket);
            let vaults = KeyValueStore::new();
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }
    }
}
