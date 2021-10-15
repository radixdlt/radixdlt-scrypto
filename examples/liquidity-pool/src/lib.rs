use scrypto::prelude::*;

blueprint! {
    struct Token {
        vault: Vault
    }

    impl Token {
        pub fn new(name: String, symbol: String) -> Address {
            Self {
                vault: Vault::wrap(
                    ResourceBuilder::new()
                        .metadata("name", &name)
                        .metadata("symbol", &symbol)
                        .create_fixed(1000)
                )
            }
            .instantiate()
        }

        pub fn get_vault_amount(&mut self) -> Amount {
            return self.vault.amount();
        }
    }
}

/*
blueprint! {
    struct TokenB {
        vault: Vault
    }

    impl TokenB {
        pub fn new() -> Address {
            Self {
                vault: Vault::wrap(
                    ResourceBuilder::new()
                        .metadata("name", "Token B")
                        .metadata("symbol", "tokenB")
                        .create_fixed(1000)
                )
            }
            .instantiate()
        }
    }
}
*/

blueprint! {
    struct LiquidityPool {
        token_a: Vault,
        token_b: Vault
    }

    impl LiquidityPool {
        pub fn new(token_a: Address, token_b: Address) -> Address {
            Self {
                token_a: Vault::new(token_a),
                token_b: Vault::new(token_b)
            }
            .instantiate()
        }

        pub fn get_pair(&mut self) -> (Address, Address) {
            (self.token_a.resource(), self.token_b.resource())
        }

        pub fn swap(&mut self, amount_out: Amount, amount_in: Bucket) -> (Bucket, Bucket) {
            assert!(amount_out > 0.into() , "Amount out must be greater than zero");
 
            let price = 3;

            let amount_in_to_take = amount_out * price.into();

            assert!(amount_in.amount() >= amount_in_to_take , "You don' t have enough funds");
            
            if amount_in.resource() == self.token_a.resource() {
                assert!(self.token_b.amount() >= amount_out, "Token B does not have enough liquidity");
                self.token_a.put(amount_in.take(amount_in_to_take));
                return (amount_in, self.token_b.take(amount_out));
            } else {
                assert!(self.token_a.amount() >= amount_out, "Token  does not have enough liquidity");
                self.token_b.put(amount_in.take(amount_in_to_take));
                return (amount_in, self.token_a.take(amount_out));
            }
        }
    }
}
