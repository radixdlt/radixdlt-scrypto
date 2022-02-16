use sbor::*;
use scrypto::prelude::*;

import! {
r#"
{
    "package": "01e61370219353678d8bfb37bce521e257d0ec29ae9a2e95d194ea",
    "name": "PriceOracle",
    "functions": [
        {
            "name": "instantiate_oracle",
            "inputs": [
                {
                    "type": "U32"
                }
            ],
            "output": {
                "type": "Tuple",
                "elements": [
                    {
                        "type": "Custom",
                        "name": "scrypto::resource::Bucket",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "scrypto::core::Component",
                        "generics": []
                    }
                ]
            }
        }
    ],
    "methods": [
        {
            "name": "get_price",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                }
            ],
            "output": {
                "type": "Option",
                "value": {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                }
            }
        },
        {
            "name": "update_price",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::resource::BucketRef",
                    "generics": []
                }
            ],
            "output": {
                "type": "Unit"
            }
        },
        {
            "name": "admin_badge_address",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::types::Address",
                "generics": []
            }
        }
    ]
}
"#
}

// Main missing features:
// - Liquidation
// - Authorization through badge

blueprint! {
    struct SyntheticPool {
        /// The price oracle
        oracle: PriceOracle,
        /// The collateralization ratio one has to maintain when minting synthetics
        collateralization_threshold: Decimal,
        /// SNX resource definition
        snx_resource_def: ResourceDef,
        /// USD resource definition
        usd_resource_def: ResourceDef,

        /// Users
        users: LazyMap<Address, User>,
        /// Synthetics
        synthetics: HashMap<String, SyntheticToken>,
        /// Mint badge
        synthetics_mint_badge: Vault,
        /// Global debt
        synthetics_global_debt_share_resource_def: ResourceDef,
    }

    impl SyntheticPool {
        pub fn instantiate_pool(
            oracle_address: Address,
            snx_token_address: Address,
            usd_token_address: Address,
            collateralization_threshold: Decimal,
        ) -> Component {
            let oracle: PriceOracle = oracle_address.into();
            let snx_resource_def: ResourceDef = snx_token_address.into();
            let usd_resource_def: ResourceDef = usd_token_address.into();
            let synthetics_mint_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Synthetics Mint Badge")
                .initial_supply_fungible(1);
            let synthetics_global_debt_share_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Synthetics Global Debt")
                .flags(MINTABLE | BURNABLE)
                .badge(synthetics_mint_badge.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            Self {
                oracle,
                collateralization_threshold,
                snx_resource_def,
                usd_resource_def,
                users: LazyMap::new(),
                synthetics: HashMap::new(),
                synthetics_mint_badge: Vault::with_bucket(synthetics_mint_badge),
                synthetics_global_debt_share_resource_def,
            }
            .instantiate()
        }

        /// Add new a new synthetic token to the protocol
        pub fn add_synthetic_token(
            &mut self,
            asset_symbol: String,
            asset_address: Address,
        ) -> Address {
            assert!(
                !self.synthetics.contains_key(&asset_symbol),
                "Asset already exist",
            );

            let token_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", format!("Synthetic {}", asset_symbol.clone()))
                .metadata("symbol", format!("s{}", asset_symbol.clone()))
                .flags(MINTABLE | BURNABLE)
                .badge(self.synthetics_mint_badge.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();
            let token_address = token_resource_def.address();
            self.synthetics.insert(
                asset_symbol.clone(),
                SyntheticToken::new(asset_symbol, asset_address, token_resource_def),
            );

            token_address
        }

        /// Deposits SNX into my staking account
        pub fn stake(&mut self, user_auth: BucketRef, stake_in_snx: Bucket) {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, true);
            user.snx.put(stake_in_snx);
        }

        /// Withdraws SNX from my staking account.
        pub fn unstake(&mut self, user_auth: BucketRef, amount: Decimal) -> Bucket {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, false);

            let tokens = user.snx.take(amount);
            user.check_collateralization_ratio(
                self.get_snx_price(),
                self.get_total_global_debt(),
                self.synthetics_global_debt_share_resource_def.clone(),
                self.collateralization_threshold,
            );
            tokens
        }

        /// Mints synthetics tokens
        pub fn mint(&mut self, user_auth: BucketRef, amount: Decimal, symbol: String) -> Bucket {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, false);

            let mut synth = self.synthetics.get(&symbol).unwrap().clone();
            let global_debt = self.get_total_global_debt();
            let new_debt = self.get_asset_price(synth.asset_address) * amount;

            user.global_debt_share
                .put(self.synthetics_mint_badge.authorize(|auth| {
                    self.synthetics_global_debt_share_resource_def.mint(
                        if global_debt.is_zero() {
                            Decimal::from(100)
                        } else {
                            new_debt
                                / (global_debt
                                    / self.synthetics_global_debt_share_resource_def.total_supply())
                        },
                        auth,
                    )
                }));
            let tokens = self
                .synthetics_mint_badge
                .authorize(|auth| synth.token_resource_def.mint(amount, auth));
            user.check_collateralization_ratio(
                self.get_snx_price(),
                self.get_total_global_debt(),
                self.synthetics_global_debt_share_resource_def.clone(),
                self.collateralization_threshold,
            );
            tokens
        }

        /// Burns synthetic tokens
        pub fn burn(&mut self, user_auth: BucketRef, bucket: Bucket) {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, false);

            let synth = self
                .synthetics
                .iter()
                .find(|(_, v)| v.token_resource_def == bucket.resource_def())
                .unwrap()
                .1;
            let global_debt = self.get_total_global_debt();
            let debt_to_remove = self.get_asset_price(synth.asset_address) * bucket.amount();
            let shares_to_burn = user.global_debt_share.take(
                self.synthetics_global_debt_share_resource_def.total_supply() * debt_to_remove
                    / global_debt
            );

            self.synthetics_mint_badge.authorize(|auth| {
                shares_to_burn.burn_with_auth(auth);
            });
            self.synthetics_mint_badge
                .authorize(|auth| bucket.burn_with_auth(auth));
        }

        /// Returns the total global debt.
        pub fn get_total_global_debt(&self) -> Decimal {
            let mut total = Decimal::zero();
            for (_, synth) in &self.synthetics {
                total +=
                    self.get_asset_price(synth.asset_address) * synth.token_resource_def.total_supply();
            }
            total
        }

        /// Retrieves the price of pair SNX/USD
        pub fn get_snx_price(&self) -> Decimal {
            self.get_asset_price(self.snx_resource_def.address())
        }

        /// Retrieves the prices of pair XYZ/USD
        pub fn get_asset_price(&self, asset_address: Address) -> Decimal {
            let usd_address = self.usd_resource_def.address();
            if let Some(oracle_price) = self.oracle.get_price(asset_address, usd_address) {
                oracle_price
            } else {
                panic!(
                    "Failed to obtain price of {}/{}",
                    asset_address, usd_address
                ) ;
            }
        }

        /// Retrieves user summary.
        pub fn get_user_summary(&mut self, user_id: Address) -> String {
            let user = self.get_user(user_id, false);
            format!(
                "SNX balance: {}, SNX price: {}, Debt: {} * {} / {}",
                user.snx.amount(),
                self.get_snx_price(),
                self.get_total_global_debt(),
                user.global_debt_share.amount(),
                self.synthetics_global_debt_share_resource_def.total_supply()
            )
        }

        /// Registers a new user
        pub fn new_user(&self) -> Bucket {
            ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Synthetic Pool User Badge")
                .initial_supply_fungible(1)
        }

        /// Parse user id from a bucket ref.
        fn get_user_id(user_auth: BucketRef) -> Address {
            assert!(user_auth.amount() > 0.into(), "Invalid user proof");
            user_auth.resource_address()
        }

        /// Retrieves user state.
        fn get_user(&mut self, user_id: Address, create_if_missing: bool) -> User {
            if let Some(user) = self.users.get(&user_id) {
                user
            } else if create_if_missing {
                self.users.insert(
                    user_id,
                    User::new(
                        self.snx_resource_def.address(),
                        self.synthetics_global_debt_share_resource_def.address(),
                    ),
                );
                self.users.get(&user_id).unwrap()
            } else {
                panic!("User not found");
            }
        }
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct SyntheticToken {
    /// The symbol of the asset
    asset_symbol: String,
    /// The resource definition address of the asset
    asset_address: Address,
    /// The synth (sXYZ) resource definition
    token_resource_def: ResourceDef,
}

impl SyntheticToken {
    pub fn new(
        asset_symbol: String,
        asset_address: Address,
        token_resource_def: ResourceDef,
    ) -> Self {
        Self {
            asset_symbol,
            asset_address,
            token_resource_def,
        }
    }
}

#[derive(Debug, TypeId, Encode, Decode, Describe)]
pub struct User {
    snx: Vault,
    global_debt_share: Vault,
}

impl User {
    pub fn new(snx_address: Address, global_debt_share_address: Address) -> Self {
        Self {
            snx: Vault::new(snx_address),
            global_debt_share: Vault::new(global_debt_share_address),
        }
    }

    // Checks the collateralization ratio of this user
    pub fn check_collateralization_ratio(
        &self,
        snx_price: Decimal,
        global_debt: Decimal,
        global_debt_resource_def: ResourceDef,
        threshold: Decimal,
    ) {
        if !global_debt_resource_def.total_supply().is_zero() && !self.global_debt_share.amount().is_zero() {
            assert!(
                self.snx.amount() * snx_price
                    / (global_debt / global_debt_resource_def.total_supply()
                        * self.global_debt_share.amount())
                    >= threshold,
                "Under collateralized!",
            );
        }
    }
}
