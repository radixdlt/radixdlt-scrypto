use scrypto::prelude::*;

blueprint! {
    struct AutoLend {
        aB_resource_def: ResourceDef,
        aB_pool: Vault,
        B_pool: Vault,
        C_pool: Vault,
        collateral_ratio: u32,
    }

    impl AutoLend {
        pub fn new(B_addr: Address, C_addr: Address) -> Component {
             
            let aB_resource_def = ResourceBuilder::new()
                .metadata("symbol", "aB")
                .metadata("name", "aB")
                .create_mutable(Context::package_address());
            let aB_addr = aB_resource_def.address();

            Self {
                aB_resource_def,
                aB_pool: Vault::new(aB_addr),
                B_pool: Vault::new(B_addr),
                C_pool: Vault::new(C_addr), 
                collateral_ratio: 2,
            }
            .instantiate()
        }

        // XXX: HOW TO KNOW WHENTHE COLLATERAL DROPS BELOW RATIO
        //      ^^ add a map with all the credits? (per currecy pair)

        // deposit B and get aB
        pub fn deposit(&mut self, B_tokens: Bucket) -> Bucket {
            let aB_amount_needed = B_tokens.amount();
            self.B_pool.put(B_tokens);
            let aB_tokens = self.aB_resource_def.mint(aB_amount_needed);
            return aB_tokens
        }

        // get back the deposit
        // XXX: HOW TO PAY INTEREST?!!
        pub fn redeem(&mut self, aB_tokens: Bucket) -> Bucket {
            let B_amount_needed = aB_tokens.amount();
            scrypto_assert!(
                self.B_pool.amount() < B_amount_needed,
                "Not enough liquidity"
            );
            aB_tokens.burn();
            return self.B_pool.take(B_amount_needed);
        }

        // only one currency (B) available for borrow,
        // so 1 arg for now
        pub fn borrow(&mut self, B_requested: u32, C_tokens: Bucket) -> Bucket {
            
            // TODO: go via oracle to establish B<->C exachange
            //       bellow I assume
            scrypto_assert!(
                C_tokens.amount().as_u32() < B_requested * self.collateral_ratio,
                "Not enough collateral"
            );
            scrypto_assert!(
                self.B_pool.amount().as_u32() < B_requested,
                "Not enough liquidity"
            );
            self.C_pool.put(C_tokens);

            // TODO: take fee % and add a pool for it

            return self.B_pool.take(B_requested);
        }

        // give back the Bs
        // XXX: HOW TO KNOW WHICH COLLATERAL WE SHOULD RETURN?!!
        // XXX: HOW TO AUTHORIZE? DO WE NEED IT? ITs x2 collateral!!!
        // XXX: WE NEED TO PASS A PAIR (like B<->C) or HAVE CONTRACT PER PAIR
        pub fn repay(&mut self, B_repaid: Bucket) -> Bucket {
            let repaid_B = B_repaid.amount();
            let needed_C = repaid_B * self.collateral_ratio;
            scrypto_assert!(
                self.C_pool.amount() < needed_C,
                "Not enough liquidity"
            );
            self.B_pool.put(B_repaid);
            return self.C_pool.take(needed_C);
        }

        pub fn get_collateral_ratio(&self) -> u32 {
            return self.collateral_ratio;
        }

        // XXX: again idea with user classes? L... and B... s

    }
}
