use scrypto::prelude::*;

blueprint! {
    struct Token {
        vault: Vault
    }
    impl Token {
        pub fn new(name: String, symbol: String) -> Component {
            Self {
                vault: Vault::with_bucket(
                    ResourceBuilder::new()
                        .metadata("name", &name)
                        .metadata("symbol", &symbol)
                        .create_fixed(1000)
                )
            }
            .instantiate()
        }
        pub fn get_vault_amount(&mut self) -> Amount {
            return self.vault.amount()
        }
    }
}