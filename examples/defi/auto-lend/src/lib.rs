use sbor::*;
use scrypto::prelude::*;

// This is a barebone implementation of Lending protocol.
//
// The following core features are missing:
// * Fees
// * Multi-collateral with price oracle
// * Authorization
// * Interest dynamic adjustment strategy
// * Upgradability

#[derive(Debug, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct User {
    /// The user's deposit balance
    pub deposit_balance: Decimal,
    /// The interest rate of deposits
    pub deposit_interest_rate: Decimal,
    /// Last update timestamp
    pub deposit_last_update: u64,

    /// The user's borrow balance
    pub borrow_balance: Decimal,
    /// The (stable) interest rate of loans
    pub borrow_interest_rate: Decimal,
    /// Last update timestamp
    pub borrow_last_update: u64,
}

impl User {
    pub fn get_collateral_ratio(&self) -> Option<Decimal> {
        if self.borrow_balance.is_zero() {
            None
        } else {
            let collateral = &self.deposit_balance
                + &self.deposit_balance
                    * &self.deposit_interest_rate
                    * self.deposit_time_elapsed();

            let loan = &self.borrow_balance
                + &self.borrow_balance * &self.borrow_interest_rate * self.borrow_time_elapsed();

            Some(collateral / loan)
        }
    }

    pub fn check_collateral_ratio(&self, min_collateral_ratio: Decimal) {
        let collateral_ratio = self.get_collateral_ratio();
        if let Some(ratio) = collateral_ratio {
            scrypto_assert!(
                ratio >= min_collateral_ratio,
                "Min collateral ratio does not meet"
            );
        }
    }

    pub fn on_deposit(&mut self, amount: Decimal, interest_rate: Decimal) {
        // Increase principle balance by interests accrued
        let interest =
            &self.deposit_balance * &self.deposit_interest_rate * self.deposit_time_elapsed();
        self.deposit_balance += interest;

        // Calculate the aggregated interest of previous deposits & the new deposit
        self.deposit_interest_rate = (&self.deposit_balance * &self.deposit_interest_rate
            + &amount * &interest_rate)
            / (&self.deposit_balance + &amount);

        // Increase principle balance by the amount.
        self.deposit_balance += amount;

        // Update timestamp
        self.deposit_last_update = Context::current_epoch();
    }

    pub fn on_redeem(&mut self, amount: Decimal) -> Decimal {
        // Deduct withdrawn amount from principle
        self.deposit_balance -= &amount;

        // Calculate the amount to return
        &amount + &amount * &self.deposit_interest_rate * self.deposit_time_elapsed()
    }

    pub fn on_borrow(&mut self, amount: Decimal, interest_rate: Decimal) {
        // Increase borrow balance by interests accrued
        let interest =
            &self.borrow_balance * &self.borrow_interest_rate * self.borrow_time_elapsed();
        self.borrow_balance += interest;

        // Calculate the aggregated interest of previous borrows & the new borrow
        self.borrow_interest_rate = (&self.borrow_balance * &self.borrow_interest_rate
            + &amount * &interest_rate)
            / (&self.borrow_balance + &amount);

        // Increase principle balance by the amount.
        self.borrow_balance += amount;

        // Update timestamp
        self.borrow_last_update = Context::current_epoch();
    }

    pub fn on_repay(&mut self, amount: Decimal) -> Decimal {
        // Increase borrow balance by interests accrued
        let interest =
            &self.borrow_balance * &self.borrow_interest_rate * self.borrow_time_elapsed();
        self.borrow_balance += interest;

        // Repay the loan
        if self.borrow_balance < amount {
            let to_return = amount - &self.borrow_balance;
            self.borrow_balance = Decimal::zero();
            self.borrow_interest_rate = Decimal::zero();
            to_return
        } else {
            self.borrow_balance -= amount;
            Decimal::zero()
        }
    }

    fn deposit_time_elapsed(&self) -> u64 {
        // +1 is for demo purpose only
        Context::current_epoch() - self.deposit_last_update + 1
    }

    fn borrow_time_elapsed(&self) -> u64 {
        // +1 is for demo purpose only
        Context::current_epoch() - self.borrow_last_update + 1
    }
}

blueprint! {
    struct AutoLend {
        /// The liquidity pool
        liquidity_pool: Vault,
        /// The total amount of all borrows
        total_borrows: Decimal,
        /// The min collateral ratio that a user has to maintain
        min_collateral_ratio: Decimal,
        /// The max percentage of liquidity pool one can borrow
        max_borrow_percentage: Decimal,
        /// The max percentage of debt one can liquidate
        max_liquidation_percentage: Decimal,
        /// Liquidation bonus
        liquidation_bonus: Decimal,
        /// AToken resource definition
        a_token_def: ResourceDef,
        /// AToken minter badge
        a_token_minter_badge: Vault,
        /// User state
        users: LazyMap<Address, User>,
        /// The interest rate of deposits, per epoch
        deposit_interest_rate: Decimal,
        /// The (stable) interest rate of loans, per epoch
        borrow_interest_rate: Decimal,
    }

    impl AutoLend {
        /// Creates a lending pool, with single collateral.
        pub fn new(reserve_address: Address, reserve_symbol: String) -> Component {
            let a_token_minter_badge = ResourceBuilder::new()
                .metadata("name", "AToken Minter Badge")
                .new_badge_fixed(1);
            let a_token_def = ResourceBuilder::new()
                .metadata("name", format!("a{}", reserve_symbol))
                .metadata("symbol", format!("a{}", reserve_symbol))
                .new_token_mutable(a_token_minter_badge.resource_def());

            Self {
                liquidity_pool: Vault::new(reserve_address),
                total_borrows: Decimal::zero(),
                min_collateral_ratio: "1.2".parse().unwrap(),
                max_borrow_percentage: "0.3".parse().unwrap(),
                max_liquidation_percentage: "0.5".parse().unwrap(),
                liquidation_bonus: "0.05".parse().unwrap(),
                a_token_def,
                a_token_minter_badge: Vault::with_bucket(a_token_minter_badge),
                users: LazyMap::new(),
                deposit_interest_rate: "0.01".parse().unwrap(),
                borrow_interest_rate: "0.02".parse().unwrap(),
            }
            .instantiate()
        }

        /// Registers a new user
        pub fn new_user(&self) -> Bucket {
            ResourceBuilder::new()
                .metadata("name", "AutoLend User Badge")
                .new_badge_fixed(1)
        }

        /// Deposits into the liquidity pool and start earning.
        pub fn deposit(&mut self, user_auth: BucketRef, reserve_tokens: Bucket) -> Bucket {
            let user_id = Self::get_user_id(user_auth);
            let amount = reserve_tokens.amount();

            // Mint aA token 1:1
            let a_tokens = self
                .a_token_minter_badge
                .authorize(|badge| self.a_token_def.mint(amount.clone(), badge));

            // Update user state
            let deposit_interest_rate = self.deposit_interest_rate.clone();
            let user = match self.users.get(&user_id) {
                Some(mut user) => {
                    user.on_deposit(amount, deposit_interest_rate);
                    user
                }
                None => User {
                    deposit_balance: amount,
                    borrow_balance: Decimal::zero(),
                    deposit_interest_rate,
                    borrow_interest_rate: Decimal::zero(),
                    deposit_last_update: Context::current_epoch(),
                    borrow_last_update: Context::current_epoch(),
                },
            };

            // Commit state changes
            self.users.insert(user_id, user);
            self.liquidity_pool.put(reserve_tokens);
            a_tokens
        }

        /// Redeems the underlying assets, partially or in full.
        pub fn redeem(&mut self, user_auth: BucketRef, a_tokens: Bucket) -> Bucket {
            let user_id = Self::get_user_id(user_auth);

            // Update user state
            let mut user = self.get_user(user_id);
            let to_return_amount = user.on_redeem(a_tokens.amount());
            user.check_collateral_ratio(self.min_collateral_ratio.clone());

            // Burn the aToken used for redeeming
            self.a_token_minter_badge.authorize(|badge| {
                a_tokens.burn(badge);
            });

            debug!(
                "LP balance: {}, redeemded: {}",
                self.liquidity_pool.amount(),
                to_return_amount
            );

            // Commit state changes
            self.users.insert(user_id, user);
            self.liquidity_pool.take(to_return_amount)
        }

        /// Borrows the specified amount from lending pool
        pub fn borrow(&mut self, user_auth: BucketRef, requested: Decimal) -> Bucket {
            let user_id = Self::get_user_id(user_auth);

            scrypto_assert!(
                requested <= self.liquidity_pool.amount() * &self.max_borrow_percentage,
                "Max borrow percent exceeded"
            );

            // Update user state
            let borrow_interest_rate = self.borrow_interest_rate.clone();
            let mut user = self.get_user(user_id);
            user.on_borrow(requested.clone(), borrow_interest_rate);
            user.check_collateral_ratio(self.min_collateral_ratio.clone());

            // Commit state changes
            self.users.insert(user_id, user);
            self.liquidity_pool.take(requested)
        }

        /// Repays a loan, partially or in full.
        pub fn repay(&mut self, user_auth: BucketRef, repaid: Bucket) -> Bucket {
            let user_id = Self::get_user_id(user_auth);

            // Update user state
            let mut user = self.get_user(user_id);
            let to_return_amount = user.on_repay(repaid.amount());
            let to_return = repaid.take(to_return_amount);

            // Commit state changes
            self.users.insert(user_id, user);
            self.liquidity_pool.put(repaid);
            to_return
        }

        /// Liquidates one user's position, if it's under collateralized.
        pub fn liquidate(&mut self, user_id: Address, repaid: Bucket) -> Bucket {
            let user = self.get_user(user_id);

            // Check if the user is under collateralized
            let collateral_ratio = user.get_collateral_ratio();
            if let Some(ratio) = collateral_ratio {
                scrypto_assert!(
                    ratio <= self.min_collateral_ratio,
                    "Liquidation not allowed."
                );
            } else {
                scrypto_abort("No borrow from the user");
            }

            // Check liquidation size
            scrypto_assert!(
                repaid.amount() <= user.borrow_balance * &self.max_liquidation_percentage,
                "Liquidation not allowed."
            );

            todo!()
        }

        /// Returns the current state of a user.
        pub fn get_user(&self, user_id: Address) -> User {
            match self.users.get(&user_id) {
                Some(user) => user,
                _ => scrypto_abort("User not found"),
            }
        }

        /// Returns the earn interest rate per epoch
        pub fn set_deposit_interest_rate(&mut self, rate: Decimal) {
            self.deposit_interest_rate = rate;
        }

        /// Returns the borrow interest rate per epoch
        pub fn set_borrow_interest_rate(&mut self, rate: Decimal) {
            self.borrow_interest_rate = rate;
        }

        /// Parse user id from a bucket ref.
        fn get_user_id(user_auth: BucketRef) -> Address {
            scrypto_assert!(user_auth.amount() > 0.into(), "Invalid user proof");
            let user_id = user_auth.resource_def().address();
            user_auth.drop();
            user_id
        }
    }
}
