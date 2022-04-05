use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Data {}

blueprint! {
    struct VaultTest {
        vault: Vault,
        vaults: LazyMap<u128, Vault>,
        vault_vector: Vec<Vault>,
    }

    impl VaultTest {
        pub fn dangling_vault() -> () {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(1);
            let _vault = Vault::with_bucket(bucket);
        }

        fn new_fungible() -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(1)
        }

        pub fn new_vault_into_map() -> ComponentAddress {
            let bucket = Self::new_fungible();
            let vault = Vault::with_bucket(bucket);
            let bucket = Self::new_fungible();
            let vaults = LazyMap::new();
            vaults.insert(0, Vault::with_bucket(bucket));
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        pub fn invalid_double_ownership_of_vault() -> ComponentAddress {
            let bucket = Self::new_fungible();
            let vault = Vault::new(bucket.resource_address());
            let vaults = LazyMap::new();
            vaults.insert(0, vault);
            let mut vault = vaults.get(&0).unwrap();
            vault.put(bucket);

            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_vault_into_map_then_get() -> ComponentAddress {
            let bucket = Self::new_fungible();
            let vault = Vault::new(bucket.resource_address());
            let vaults = LazyMap::new();
            vaults.insert(0, vault);
            let mut vault = vaults.get(&0).unwrap();
            vault.put(bucket);

            let vault_vector = Vec::new();
            VaultTest {
                vault: Vault::with_bucket(Self::new_fungible()),
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        pub fn overwrite_vault_in_map(&mut self) -> () {
            let bucket = Self::new_fungible();
            self.vaults.insert(0, Vault::with_bucket(bucket))
        }

        pub fn new_vault_into_vector() -> ComponentAddress {
            let bucket = Self::new_fungible();
            let vault = Vault::with_bucket(bucket);
            let bucket = Self::new_fungible();
            let vaults = LazyMap::new();
            let mut vault_vector = Vec::new();
            vault_vector.push(Vault::with_bucket(bucket));
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        pub fn clear_vector(&mut self) -> () {
            self.vault_vector.clear()
        }

        pub fn push_vault_into_vector(&mut self) -> () {
            let bucket = Self::new_fungible();
            self.vault_vector.push(Vault::with_bucket(bucket))
        }

        pub fn new_vault_with_take() -> ComponentAddress {
            let bucket = Self::new_fungible();
            let mut vault = Vault::with_bucket(bucket);
            let bucket = vault.take(1);
            vault.put(bucket);
            let vaults = LazyMap::new();
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        fn create_non_fungible_vault() -> Vault {
            let bucket = ResourceBuilder::new_non_fungible()
                .metadata("name", "TestToken")
                .initial_supply([(NonFungibleId::from(1u128), Data {})]);
            Vault::with_bucket(bucket)
        }

        pub fn new_vault_with_take_non_fungible() -> ComponentAddress {
            let mut vault = Self::create_non_fungible_vault();
            let bucket = vault.take_non_fungible(&NonFungibleId::from(1u128));
            vault.put(bucket);
            let vaults = LazyMap::new();
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_vault_with_get_non_fungible_ids() -> ComponentAddress {
            let vault = Self::create_non_fungible_vault();
            let _ids = vault.get_non_fungible_ids();
            let vaults = LazyMap::new();
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_vault_with_get_amount() -> ComponentAddress {
            let vault = Self::create_non_fungible_vault();
            let _amount = vault.amount();
            let vaults = LazyMap::new();
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }

        pub fn new_vault_with_get_resource_manager() -> ComponentAddress {
            let vault = Self::create_non_fungible_vault();
            let _resource_manager = vault.resource_address();
            let vaults = LazyMap::new();
            let vault_vector = Vec::new();
            VaultTest {
                vault,
                vaults,
                vault_vector,
            }
            .instantiate()
            .globalize()
        }
    }
}
