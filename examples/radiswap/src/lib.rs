use scrypto::prelude::*;

blueprint! {
    struct Radiswap {
        /// The resource definition of the LP token.
        lp_token_def: ResourceDef,
        /// The reserve for token A.
        a_pool: Vault,
        /// The reserve for token B.
        b_pool: Vault,
        /// The fee to apply for every swap, like `3` for a 0.3% fee.
        fee_in_thousandths: u32,
        /// The scale to use when adding/removing liquidity. When this value is
        /// set to 5, the min liquidity to add is `0.001%` of the pools.
        scale: usize,
    }

    impl Radiswap {
        /// Creates a Radiswap component for token pair A and B and returns its address
        /// and the initial LP tokens.
        pub fn new(
            a_tokens: Bucket,
            b_tokens: Bucket,
            lp_initial_supply: Amount,
            lp_symbol: String,
            lp_name: String,
            lp_url: String,
            fee_in_thousandths: u32,
            scale: usize,
        ) -> (Address, Bucket) {
            // Check arguments
            assert!(
                !a_tokens.is_empty() && !b_tokens.is_empty(),
                "You must pass in an initial supply of each token"
            );
            assert!(fee_in_thousandths <= 1000, "Invalid fee in thousandths");
            assert!(scale >= 1 && scale <= 9, "Invalid scale");

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
                fee_in_thousandths,
                scale,
            }
            .instantiate();

            // Return the new Radiswap component, as well as the initial supply of LP tokens
            (radiswap, lp_tokens)
        }

        /// Returns the current total supply of the LP token.
        pub fn lp_token_supply(&self) -> Amount {
            self.lp_token_def.supply()
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
        pub fn add_liquidity(&self, a_tokens: Bucket, b_tokens: Bucket) -> (Bucket, Bucket) {
            let scale = Amount::exp10(self.scale);
            let a_share = scale * a_tokens.amount() / self.a_pool.amount();
            let b_share = scale * b_tokens.amount() / self.b_pool.amount();

            let (actual_share, remainder) = if a_share <= b_share {
                // We will claim all input token A's, and only the correct amount of token B
                self.a_pool.put(a_tokens);
                self.b_pool.put(b_tokens.take(self.b_pool.amount() * a_share / scale));
                (a_share, b_tokens)
            } else {
                // We will claim all input token B's, and only the correct amount of token A
                self.b_pool.put(b_tokens);
                self.a_pool.put(a_tokens.take(self.a_pool.amount() * b_share / scale));
                (b_share, a_tokens)
            };

            // Mint LP tokens according to the share the provider is contributing
            let lp_tokens = self.lp_token_def.mint(self.lp_token_supply() * actual_share / scale);

            // Return the LP tokens along with any remainder
            (lp_tokens, remainder)
        }

        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            assert!(
                self.lp_token_def.address() == lp_tokens.resource(),
                "Wrong token type passed in"
            );

            // Calculate the share based on the input LP tokens.
            let scale = Amount::exp10(self.scale);
            let share = scale * lp_tokens.amount() / self.lp_token_supply();

            // Withdraw the correct amounts of tokens A and B from reserves
            let a_withdrawn = self.a_pool.take(self.a_pool.amount() * share / scale);
            let b_withdrawn = self.b_pool.take(self.b_pool.amount() * share / scale);

            // Burn the LP tokens received
            lp_tokens.burn();

            // Return the withdrawn tokens
            (a_withdrawn, b_withdrawn)
        }

        /// Swaps token A for B, or vice versa.
        pub fn swap(&self, input_tokens: Bucket) -> Bucket {
            // Calculate the swap fee
            let fee_amount = input_tokens.amount() * self.fee_in_thousandths / 1000;

            if input_tokens.resource() == self.a_pool.resource() {
                // Calculate how much of token B we will return
                let b_amount = self.b_pool.amount()
                    - self.a_pool.amount() * self.b_pool.amount()
                        / (input_tokens.amount() - fee_amount + self.a_pool.amount());

                // Put the input tokens into our pool
                self.a_pool.put(input_tokens);

                // Return the tokens owed
                self.b_pool.take(b_amount)
            } else {
                // Calculate how much of token A we will return
                let a_amount = self.a_pool.amount()
                    - self.a_pool.amount() * self.b_pool.amount()
                        / (input_tokens.amount() - fee_amount + self.b_pool.amount());

                // Put the input tokens into our pool
                self.b_pool.put(input_tokens);

                // Return the tokens owed
                self.a_pool.take(a_amount)
            }
        }
    }
}
