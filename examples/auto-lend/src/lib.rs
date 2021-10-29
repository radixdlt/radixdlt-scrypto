use scrypto::prelude::*;
mod token;

blueprint! {
    struct AutoLend {
        a_b_resource_auth: Vault,
        a_b_resource_def: ResourceDef,
        a_b_pool: Vault,
        b_pool: Vault,
        c_pool: Vault,
        collateral_ratio: u32,
    }

    impl AutoLend {
        pub fn new(b_addr: Address, c_addr: Address) -> Component {
            let  a_b_resource_auth = ResourceBuilder::new()
            .metadata("name", "LP Token Mint Auth")
            .create_fixed(1);
            let a_b_resource_def = ResourceBuilder::new()
                .metadata("symbol", "aB")
                .metadata("name", "aB")
                .create_mutable(a_b_resource_auth.resource_def());
            let a_b_addr = a_b_resource_def.address();

            Self {
                a_b_resource_auth: Vault::with_bucket(a_b_resource_auth),
                a_b_resource_def,
                a_b_pool: Vault::new(a_b_addr),
                b_pool: Vault::new(b_addr),
                c_pool: Vault::new(c_addr), 
                collateral_ratio: 2,
            }
            .instantiate()
        }

        // XXX: HOW TO KNOW WHENTHE COLLATERAL DROPS BELOW RATIO
        //      ^^ add a map with all the credits? (per currecy pair)

        // deposit B and get aB
        pub fn deposit(&mut self, b_tokens: Bucket) -> Bucket {
            let lp_amount_to_be_minted = if self.b_pool.amount() > 0.into() {
                b_tokens.amount() / self.b_pool.amount() * self.a_b_resource_def.supply()
            } else {
                1.into()
            };
            self.b_pool.put(b_tokens);
            let a_b_tokens = self.a_b_resource_auth.authorize(|badge| {
                self.a_b_resource_def
                    .mint(lp_amount_to_be_minted, badge)
            });
            return a_b_tokens
        }

        // get back the deposit
        // XXX: HOW TO PAY INTEREST?!!
        //      1. We need internal map. Tracking external account doesn't work as the asset is liquid
        //      2. Interest needs to be based on liquidity as well
        pub fn redeem(&mut self, a_b_tokens: Bucket) -> Bucket {
            let b_amount_needed = a_b_tokens.amount();
            scrypto_assert!(
                self.b_pool.amount() < b_amount_needed,
                "Not enough liquidity"
            );
            a_b_tokens.burn();
            return self.b_pool.take(b_amount_needed);
        }

        // only one currency (B) available for borrow,
        // so 1 arg for now
        pub fn borrow(&mut self, b_requested: u32, c_tokens: Bucket) -> Bucket {
            
            // TODO: go via oracle to establish B<->C exachange
            //       bellow I assume
            scrypto_assert!(
                c_tokens.amount().as_u32() < b_requested * self.collateral_ratio,
                "Not enough collateral"
            );
            scrypto_assert!(
                self.b_pool.amount().as_u32() < b_requested,
                "Not enough liquidity"
            );
            self.c_pool.put(c_tokens);

            // TODO: take fee % and add a pool for it

            return self.b_pool.take(b_requested);
        }

        // give back the Bs
        // XXX: HOW TO KNOW WHICH COLLATERAL WE SHOULD RETURN?!!
        // XXX: HOW TO AUTHORIZE? DO WE NEED IT? ITs x2 collateral!!!
        // XXX: WE NEED TO PASS A PAIR (like B<->C) or HAVE CONTRACT PER PAIR
        pub fn repay(&mut self, b_repaid: Bucket) -> Bucket {
            let repaid_b = b_repaid.amount();
            let needed_c = repaid_b * self.collateral_ratio;
            scrypto_assert!(
                self.c_pool.amount() < needed_c,
                "Not enough liquidity"
            );
            self.b_pool.put(b_repaid);
            return self.c_pool.take(needed_c);
        }

        pub fn get_collateral_ratio(&self) -> u32 {
            return self.collateral_ratio;
        }

        // XXX: again idea with user classes? L... and B... s

        pub fn a_b_tokens_supply(&mut self) -> Amount {
            return self.a_b_resource_def.supply();
        }

        pub fn b_tokens_liquidity(&mut self) -> Amount {
            return self.b_pool.amount()
        }
    }
}
