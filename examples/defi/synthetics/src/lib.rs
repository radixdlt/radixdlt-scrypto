use sbor::*;
use scrypto::prelude::*;

import! {
r#"
    {
        "package_id": "014eb598fe6ed7df56a5f02950df2d7b08530d9d1081f05a6398f9",
        "blueprint_name": "PriceOracle",
        "functions": [
            {
                "name": "new",
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
                            "name": "Bucket",
                            "generics": []
                        },
                        {
                            "type": "Custom",
                            "name": "ComponentId",
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
                        "name": "ResourceDefId",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "ResourceDefId",
                        "generics": []
                    }
                ],
                "output": {
                    "type": "Option",
                    "value": {
                        "type": "Custom",
                        "name": "Decimal",
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
                        "name": "ResourceDefId",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "ResourceDefId",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "Decimal",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "Proof",
                        "generics": []
                    }
                ],
                "output": {
                    "type": "Unit"
                }
            },
            {
                "name": "admin_badge",
                "mutability": "Immutable",
                "inputs": [],
                "output": {
                    "type": "Custom",
                    "name": "ResourceDefId",
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
        snx_token: ResourceDefId,
        /// USD resource definition
        usd_token: ResourceDefId,

        /// Users
        users: LazyMap<ResourceDefId, User>,
        /// Synthetics
        synthetics: HashMap<String, SyntheticToken>,
        /// Mint badge
        synthetics_mint_badge: Vault,
        /// Global debt
        synthetics_global_debt_share: ResourceDefId,
    }

    impl SyntheticPool {
        pub fn instantiate_pool(
            oracle_component_id: ComponentId,
            snx_token: ResourceDefId,
            usd_token: ResourceDefId,
            collateralization_threshold: Decimal,
        ) -> ComponentId {
            let oracle: PriceOracle = oracle_component_id.into();
            let synthetics_mint_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Synthetics Mint Badge")
                .initial_supply_fungible(1);
            let synthetics_global_debt_share = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Synthetics Global Debt")
                .flags(MINTABLE | BURNABLE)
                .badge(synthetics_mint_badge.resource_def_id(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            Self {
                oracle,
                collateralization_threshold,
                snx_token,
                usd_token,
                users: LazyMap::new(),
                synthetics: HashMap::new(),
                synthetics_mint_badge: Vault::with_bucket(synthetics_mint_badge),
                synthetics_global_debt_share,
            }
            .instantiate()
        }

        /// Add new a new synthetic token to the protocol
        pub fn add_synthetic_token(
            &mut self,
            asset_symbol: String,
            asset_resource_def_id: ResourceDefId,
        ) -> ResourceDefId {
            assert!(
                !self.synthetics.contains_key(&asset_symbol),
                "Asset already exist",
            );

            let token = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", format!("Synthetic {}", asset_symbol.clone()))
                .metadata("symbol", format!("s{}", asset_symbol.clone()))
                .flags(MINTABLE | BURNABLE)
                .badge(
                    self.synthetics_mint_badge.resource_def_id(),
                    MAY_MINT | MAY_BURN,
                )
                .no_initial_supply();
            self.synthetics.insert(
                asset_symbol.clone(),
                SyntheticToken::new(asset_symbol, asset_resource_def_id, token),
            );

            token
        }

        /// Deposits SNX into my staking account
        pub fn stake(&mut self, user_auth: Proof, stake_in_snx: Bucket) {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, true);
            user.snx.put(stake_in_snx);
        }

        /// Withdraws SNX from my staking account.
        pub fn unstake(&mut self, user_auth: Proof, amount: Decimal) -> Bucket {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, false);

            let tokens = user.snx.take(amount);
            user.check_collateralization_ratio(
                self.get_snx_price(),
                self.get_total_global_debt(),
                self.synthetics_global_debt_share.clone(),
                self.collateralization_threshold,
            );
            tokens
        }

        /// Mints synthetics tokens
        pub fn mint(&mut self, user_auth: Proof, amount: Decimal, symbol: String) -> Bucket {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, false);

            let mut synth = self.synthetics.get(&symbol).unwrap().clone();
            let global_debt = self.get_total_global_debt();
            let new_debt = self.get_asset_price(synth.asset_resource_def_id) * amount;

            user.global_debt_share
                .put(self.synthetics_mint_badge.authorize(|auth| {
                    self.synthetics_global_debt_share.mint(
                        if global_debt.is_zero() {
                            Decimal::from(100)
                        } else {
                            new_debt
                                / (global_debt / self.synthetics_global_debt_share.total_supply())
                        },
                        auth,
                    )
                }));
            let tokens = self
                .synthetics_mint_badge
                .authorize(|auth| synth.token.mint(amount, auth));
            user.check_collateralization_ratio(
                self.get_snx_price(),
                self.get_total_global_debt(),
                self.synthetics_global_debt_share.clone(),
                self.collateralization_threshold,
            );
            tokens
        }

        /// Burns synthetic tokens
        pub fn burn(&mut self, user_auth: Proof, bucket: Bucket) {
            let user_id = Self::get_user_id(user_auth);
            let mut user = self.get_user(user_id, false);

            let synth = self
                .synthetics
                .iter()
                .find(|(_, v)| v.token == bucket.resource_def_id())
                .unwrap()
                .1;
            let global_debt = self.get_total_global_debt();
            let debt_to_remove =
                self.get_asset_price(synth.asset_resource_def_id) * bucket.amount();
            let shares_to_burn = user.global_debt_share.take(
                self.synthetics_global_debt_share.total_supply() * debt_to_remove / global_debt,
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
                    self.get_asset_price(synth.asset_resource_def_id) * synth.token.total_supply();
            }
            total
        }

        /// Retrieves the price of pair SNX/USD
        pub fn get_snx_price(&self) -> Decimal {
            self.get_asset_price(self.snx_token)
        }

        /// Retrieves the prices of pair XYZ/USD
        pub fn get_asset_price(&self, asset_resource_def_id: ResourceDefId) -> Decimal {
            let usd_token_ref = self.usd_token;
            if let Some(oracle_price) = self.oracle.get_price(asset_resource_def_id, usd_token_ref)
            {
                oracle_price
            } else {
                panic!(
                    "Failed to obtain price of {}/{}",
                    asset_resource_def_id, usd_token_ref
                );
            }
        }

        /// Retrieves user summary.
        pub fn get_user_summary(&mut self, user_id: ResourceDefId) -> String {
            let user = self.get_user(user_id, false);
            format!(
                "SNX balance: {}, SNX price: {}, Debt: {} * {} / {}",
                user.snx.amount(),
                self.get_snx_price(),
                self.get_total_global_debt(),
                user.global_debt_share.amount(),
                self.synthetics_global_debt_share.total_supply()
            )
        }

        /// Registers a new user
        pub fn new_user(&self) -> Bucket {
            ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Synthetic Pool User Badge")
                .initial_supply_fungible(1)
        }

        /// Parse user id from a proof.
        fn get_user_id(user_auth: Proof) -> ResourceDefId {
            assert!(user_auth.amount() > 0.into(), "Invalid user proof");
            user_auth.resource_def_id()
        }

        /// Retrieves user state.
        fn get_user(&mut self, user_id: ResourceDefId, create_if_missing: bool) -> User {
            if let Some(user) = self.users.get(&user_id) {
                user
            } else if create_if_missing {
                self.users.insert(
                    user_id,
                    User::new(self.snx_token, self.synthetics_global_debt_share),
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
    /// The resource definition ID of the asset
    asset_resource_def_id: ResourceDefId,
    /// The synth (sXYZ) resource definition
    token: ResourceDefId,
}

impl SyntheticToken {
    pub fn new(
        asset_symbol: String,
        asset_resource_def_id: ResourceDefId,
        token: ResourceDefId,
    ) -> Self {
        Self {
            asset_symbol,
            asset_resource_def_id,
            token,
        }
    }
}

#[derive(Debug, TypeId, Encode, Decode, Describe)]
pub struct User {
    snx: Vault,
    global_debt_share: Vault,
}

impl User {
    pub fn new(
        snx_token_ref: ResourceDefId,
        global_debt_share_resource_def_id: ResourceDefId,
    ) -> Self {
        Self {
            snx: Vault::new(snx_token_ref),
            global_debt_share: Vault::new(global_debt_share_resource_def_id),
        }
    }

    // Checks the collateralization ratio of this user
    pub fn check_collateralization_ratio(
        &self,
        snx_price: Decimal,
        global_debt: Decimal,
        resource_def_id: ResourceDefId,
        threshold: Decimal,
    ) {
        if !resource_def_id.total_supply().is_zero() {
            assert!(
                self.snx.amount() * snx_price
                    / (global_debt / resource_def_id.total_supply()
                        * self.global_debt_share.amount())
                    >= threshold,
                "Under collateralized!",
            );
        }
    }
}
