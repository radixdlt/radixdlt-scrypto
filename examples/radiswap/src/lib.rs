use scrypto::prelude::*;

blueprint! {
    struct Radiswap {
        /// The resource definition of the LP token.
        lp_token_def: ResourceDef,
        /// The reserve of token A.
        a_pool: Vault,
        /// The reserve of token B.
        b_pool: Vault,
        /// The commission to apply for every swap.
        fee_in_thousandth: Amount,
        /// The precision used when calculating shares.
        precision: Amount,
    }

    impl Radiswap {
        /// Creates a swap pool for token A and B and returns the Radiswap component address
        /// and the initial LP tokens.
        pub fn new(
            a_tokens: Bucket,
            b_tokens: Bucket,
            lp_initial_supply: Amount,
            lp_symbol: String,
            lp_name: String,
            lp_url: String,
            fee_in_thousandth: Amount,
            precision: Amount,
        ) -> (Address, Bucket) {
            // Make sure we were provided with an initial supply of each token
            assert!(
                a_tokens.amount() > 0.into() && b_tokens.amount() > 0.into(),
                "You must pass in an initial supply of each token"
            );

            // Instantiate our LP token and mint an initial supply of them
            let lp_token_def = ResourceBuilder::new()
                .metadata("symbol", lp_symbol)
                .metadata("name", lp_name)
                .metadata("url", lp_url)
                .create_mutable(Context::package_address());
            let lp_tokens = lp_token_def.mint(lp_initial_supply);

            // Instantiate our Radiswap component
            let radiswap = Self {
                lp_token_def,
                a_pool: Vault::with_bucket(a_tokens),
                b_pool: Vault::with_bucket(b_tokens),
                fee_in_thousandth,
                precision,
            }
            .instantiate();

            // Return the new Uniswap component, as well as the initial supply of LP tokens
            (radiswap, lp_tokens)
        }

        /// Returns the current total supply of the LP token.
        pub fn lp_token_supply(&self) -> Amount {
            self.lp_token_def.supply()
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
        pub fn add_liquidity(&self, a_tokens: Bucket, b_tokens: Bucket) -> (Bucket, Bucket) {
            let a_share = self.precision * a_tokens.amount() / self.a_pool.amount();
            let b_share = self.precision * b_tokens.amount() / self.b_pool.amount();

            let actual_share;
            let remainder;
            if a_share <= b_share {
                // We will claim all input token A's, and only the correct amount of token B
                actual_share = a_share;
                self.a_pool.put(a_tokens);
                self.b_pool.put(b_tokens.take(actual_share * self.b_pool.amount() / self.precision));
                remainder = b_tokens;
            } else {
                // We will claim all input token B's, and only the correct amount of token A
                actual_share = b_share;
                self.b_pool.put(b_tokens);
                self.a_pool.put(a_tokens.take(actual_share * self.a_pool.amount() / self.precision));
                remainder = a_tokens;
            }

            // Mint LP tokens according to the share the provider is contributing
            let lp_tokens = self.lp_token_def.mint(actual_share * self.lp_token_supply() / self.precision);

            // Return the LP tokens along with any remainder
            (lp_tokens, remainder)
        }

        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            assert!(
                self.lp_token_def.address() == lp_tokens.resource(),
                "Wrong token type passed in"
            );

            // Withdraw the correct amounts of tokens A and B from reserves
            let share = self.precision * lp_tokens.amount() / self.lp_token_supply();
            let a_withdrawn = self.a_pool.take(share * self.a_pool.amount() / self.precision);
            let b_withdrawn = self.b_pool.take(share * self.b_pool.amount() / self.precision);

            // Burn the LP tokens received
            ResourceDef::burn(lp_tokens);

            // Return the withdrawn tokens
            (a_withdrawn, b_withdrawn)
        }

        /// Swaps token A for B, or vice versa.
        pub fn swap(&self, input_tokens: Bucket) -> Bucket {
            // Calculate the swap fee
            let fee_amount = self.fee_in_thousandth * input_tokens.amount() / 1000.into();

            // Calculate the constant product of the formula
            let product = self.a_pool.amount() * self.b_pool.amount();

            if input_tokens.resource() == self.a_pool.resource() {
                // Calculate how much of token B we will return
                let b_amount = self.b_pool.amount()
                    - product / (input_tokens.amount() - fee_amount + self.a_pool.amount());

                // Put the input tokens into our pool
                self.a_pool.put(input_tokens);

                // Return the tokens owed
                self.b_pool.take(b_amount)
            } else if input_tokens.resource() == self.b_pool.resource() {
                // Calculate how much of token A we will return
                let a_amount = self.a_pool.amount()
                    - product / (input_tokens.amount() - fee_amount + self.b_pool.amount());

                // Put the input tokens into our pool
                self.b_pool.put(input_tokens);

                // Return the tokens owed
                self.a_pool.take(a_amount)
            } else {
                panic!("Unexpected input tokens");
            }
        }
    }
}
