use scrypto::prelude::*;

blueprint! {
    struct Radiswap {
        /// The resource definition of LP token.
        lp_resource_def: ResourceDef,
        /// Mint authorization to LP tokens.
        lp_minter: Vault,
        /// The reserve for token A.
        a_pool: Vault,
        /// The reserve for token B.
        b_pool: Vault,
        /// The fee to apply for every swap, like `3` for a 0.3% fee.
        fee_in_thousandths: u32,
    }

    impl Radiswap {
        /// Creates a Radiswap component for token pair A/B and returns the component address
        /// along with the initial LP tokens.
        pub fn new(
            a_tokens: Bucket,
            b_tokens: Bucket,
            lp_initial_supply: Decimal,
            lp_symbol: String,
            lp_name: String,
            lp_url: String,
            fee_in_thousandths: u32,
        ) -> (Component, Bucket) {
            // Check arguments
            scrypto_assert!(
                !a_tokens.is_empty() && !b_tokens.is_empty(),
                "You must pass in an initial supply of each token"
            );
            scrypto_assert!(fee_in_thousandths <= 1000, "Invalid fee in thousandths");

            // Instantiate our LP token and mint an initial supply of them
            let lp_minter = ResourceBuilder::new()
                .metadata("name", "LP Token Mint Auth")
                .new_token_fixed(1);
            let lp_resource_def = ResourceBuilder::new()
                .metadata("symbol", lp_symbol)
                .metadata("name", lp_name)
                .metadata("url", lp_url)
                .new_token_mutable(lp_minter.resource_def());
            let lp_tokens = lp_resource_def.mint(lp_initial_supply, lp_minter.borrow());

            // Instantiate our Radiswap component
            let radiswap = Self {
                lp_resource_def,
                lp_minter: Vault::with_bucket(lp_minter),
                a_pool: Vault::with_bucket(a_tokens),
                b_pool: Vault::with_bucket(b_tokens),
                fee_in_thousandths,
            }
            .instantiate();

            // Return the new Radiswap component, as well as the initial supply of LP tokens
            (radiswap, lp_tokens)
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
        pub fn add_liquidity(&self, a_tokens: Bucket, b_tokens: Bucket) -> (Bucket, Bucket) {
            let a_share = a_tokens.amount() / self.a_pool.amount();
            let b_share = b_tokens.amount() / self.b_pool.amount();

            let (actual_share, remainder) = if a_share <= b_share {
                // We will claim all input token A's, and only the correct amount of token B
                self.a_pool.put(a_tokens);
                self.b_pool
                    .put(b_tokens.take(self.b_pool.amount() * a_share.clone()));
                (a_share, b_tokens)
            } else {
                // We will claim all input token B's, and only the correct amount of token A
                self.b_pool.put(b_tokens);
                self.a_pool
                    .put(a_tokens.take(self.a_pool.amount() * b_share.clone()));
                (b_share, a_tokens)
            };

            // Mint LP tokens according to the share the provider is contributing
            let lp_tokens = self.lp_minter.authorize(|badge| {
                self.lp_resource_def
                    .mint(self.lp_resource_def.supply() * actual_share, badge)
            });

            // Return the LP tokens along with any remainder
            (lp_tokens, remainder)
        }

        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            scrypto_assert!(
                self.lp_resource_def == lp_tokens.resource_def(),
                "Wrong token type passed in"
            );

            // Calculate the share based on the input LP tokens.
            let share = lp_tokens.amount() / self.lp_resource_def.supply();

            // Withdraw the correct amounts of tokens A and B from reserves
            let a_withdrawn = self.a_pool.take(self.a_pool.amount() * share.clone());
            let b_withdrawn = self.b_pool.take(self.b_pool.amount() * share.clone());

            // Burn the LP tokens received
            self.lp_minter.authorize(|badge| {
                lp_tokens.burn(badge);
            });

            // Return the withdrawn tokens
            (a_withdrawn, b_withdrawn)
        }

        /// Swaps token A for B, or vice versa.
        pub fn swap(&self, input_tokens: Bucket) -> Bucket {
            // Calculate the swap fee
            let fee_amount = input_tokens.amount() * self.fee_in_thousandths / 1000;

            if input_tokens.resource_def() == self.a_pool.resource_def() {
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
