use scrypto::prelude::*;

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

        pub fn new_vault_with_put() -> Component {
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply_fungible(1);
            let vault = Vault::with_bucket(bucket);
            VaultTest { vault }.instantiate()
        }

        pub fn new_vault_with_take() -> Component {
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply_fungible(1);
            let vault = Vault::with_bucket(bucket);
            bucket = vault.take(1);
            vault.put(bucket);
            VaultTest { vault }.instantiate()
        }
    }
}
