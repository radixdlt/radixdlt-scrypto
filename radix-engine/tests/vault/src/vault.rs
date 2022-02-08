use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Data {
}

blueprint! {
    struct VaultTest {
        vault: Vault
    }

    impl VaultTest {
        pub fn dangling_vault() -> () {
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply_fungible(1);
            let _vault = Vault::with_bucket(bucket);
        }

        pub fn new_vault_with_take() -> Component {
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply_fungible(1);
            let mut vault = Vault::with_bucket(bucket);
            let bucket = vault.take(1);
            vault.put(bucket);
            VaultTest { vault }.instantiate()
        }

        fn create_non_fungible_vault() -> Vault {
            let bucket = ResourceBuilder::new_non_fungible()
                .metadata("name", "TestToken")
                .initial_supply_non_fungible([
                    (NonFungibleKey::from(1u128), Data {})
                ]);
            Vault::with_bucket(bucket)
        }

        pub fn new_vault_with_take_non_fungible() -> Component {
            let mut vault = Self::create_non_fungible_vault();
            let bucket = vault.take_non_fungible(&NonFungibleKey::from(1u128));
            vault.put(bucket);
            VaultTest { vault }.instantiate()
        }

        pub fn new_vault_with_get_non_fungible_keys() -> Component {
            let vault = Self::create_non_fungible_vault();
            let _keys = vault.get_non_fungible_keys();
            VaultTest { vault }.instantiate()
        }
    }
}
