use scrypto::prelude::*;

// Welcome to MutualFund!
//
// Start earning today by converting your XRD into liquidity.
//
// For every 1 XRD invested,
// 1. We immediately convert 0.75 XRD into SNX
// 2. All SNX will be staked into a Synthetic Pool
// 3. We mint Synthetic TESLA token a 1000% collateralization ratio
// 4. The minted sTELSA and 0.25 XRD will be added to a sTESLA/XRD swap pool owned by us (with change returned to you)
// 5. Based on your contribution (in dollar amount), we issue MutualFund share tokens which allow you to redeem underlying assets and claim dividends.

import! {
r#"
{
    "package": "014eb598fe6ed7df56a5f02950df2d7b08530d9d1081f05a6398f9",
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

import! {
r#"
{
    "package": "01e0983e33158b489e70313b77767abe80eb449e6acd46f9476328",
    "name": "SyntheticPool",
    "functions": [
        {
            "name": "instantiate_pool",
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
                    "name": "scrypto::types::Address",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::core::Component",
                "generics": []
            }
        }
    ],
    "methods": [
        {
            "name": "add_synthetic_token",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "String"
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::types::Address",
                "generics": []
            }
        },
        {
            "name": "stake",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::BucketRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                }
            ],
            "output": {
                "type": "Unit"
            }
        },
        {
            "name": "unstake",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::BucketRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
            }
        },
        {
            "name": "mint",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::BucketRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                },
                {
                    "type": "String"
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
            }
        },
        {
            "name": "burn",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::BucketRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                }
            ],
            "output": {
                "type": "Unit"
            }
        },
        {
            "name": "get_total_global_debt",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::types::Decimal",
                "generics": []
            }
        },
        {
            "name": "get_snx_price",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::types::Decimal",
                "generics": []
            }
        },
        {
            "name": "get_asset_price",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::types::Decimal",
                "generics": []
            }
        },
        {
            "name": "get_user_summary",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                }
            ],
            "output": {
                "type": "String"
            }
        },
        {
            "name": "new_user",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
            }
        }
    ]
}
"#
}

import! {
r#"
{
    "package": "01899b2991b37ee1bc51a84182a3752c1dc48ec3df01969d3516d3",
    "name": "Radiswap",
    "functions": [
        {
            "name": "instantiate_pool",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                },
                {
                    "type": "String"
                },
                {
                    "type": "String"
                },
                {
                    "type": "String"
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Tuple",
                "elements": [
                    {
                        "type": "Custom",
                        "name": "scrypto::core::Component",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "scrypto::resource::Bucket",
                        "generics": []
                    }
                ]
            }
        }
    ],
    "methods": [
        {
            "name": "add_liquidity",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
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
                        "name": "scrypto::resource::Bucket",
                        "generics": []
                    }
                ]
            }
        },
        {
            "name": "remove_liquidity",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
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
                        "name": "scrypto::resource::Bucket",
                        "generics": []
                    }
                ]
            }
        },
        {
            "name": "swap",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
            }
        }
    ]
}
"#
}

blueprint! {
    struct MutualFarm {
        /// Badge for interacting with other components.
        identity_badge: Vault,
        /// XRD/SNX Radiswap
        xrd_snx_radiswap: Radiswap,
        /// Price Oracle
        price_oracle: PriceOracle,
        /// Synthetic for minting synthetic tokens
        synthetic_pool: SyntheticPool,

        /// Asset symbol
        asset_symbol: String,
        /// Asset address
        asset_address: Address,
        /// Synthetic asset address
        synth_address: Address,
        /// SNX resource definition address
        snx_address: Address,
        /// USD resource definition address
        usd_address: Address,

        /// Radiswap for sTESLA/XRD
        radiswap: Radiswap,
        /// Radiswap LP token vault
        radiswap_lp_tokens: Vault,

        /// Mutual farm share resource definition
        mutual_farm_share_resource_def: ResourceDef,
        /// Total contribution
        total_contribution_in_usd: Decimal,
    }

    impl MutualFarm {
        pub fn instantiate_farm(
            price_oracle_address: Address,
            xrd_snx_radiswap_address: Address,
            synthetic_pool_address: Address,
            asset_symbol: String,
            asset_address: Address,
            initial_shares: Decimal,
            mut initial_xrd: Bucket,
            snx_address: Address,
            usd_address: Address,
        ) -> (Bucket, Component) {
            debug!("Create an identity badge for accessing other components");
            let identity_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "ID")
                .initial_supply_fungible(1);
            let identity_badge_address = identity_badge.resource_address();

            debug!("Fetch price info from oracle");
            let price_oracle: PriceOracle = price_oracle_address.into();
            let xrd_usd_price = price_oracle
                .get_price(initial_xrd.resource_address(), usd_address)
                .unwrap();
            let snx_usd_price = price_oracle.get_price(snx_address, usd_address).unwrap();
            let tesla_usd_price = price_oracle.get_price(asset_address, usd_address).unwrap();

            debug!("Swap 3/4 of XRD for SNX");
            let xrd_snx_radiswap: Radiswap = xrd_snx_radiswap_address.into();
            let xrd_amount = initial_xrd.amount();
            let snx = xrd_snx_radiswap.swap(initial_xrd.take(initial_xrd.amount() * 3 / 4));
            let snx_amount = snx.amount();

            debug!("Deposit SNX into synthetic pool and mint sTESLA (1/10 of our SNX).");
            let price_oracle: PriceOracle = price_oracle_address.into();
            let synthetic_pool: SyntheticPool = synthetic_pool_address.into();
            synthetic_pool.add_synthetic_token(asset_symbol.clone(), asset_address);
            synthetic_pool.stake(identity_badge.present(), snx);
            let quantity = snx_amount * snx_usd_price / 10 / tesla_usd_price;
            let synth =
                synthetic_pool.mint(identity_badge.present(), quantity, asset_symbol.clone());
            let synth_address = synth.resource_address();

            debug!("Set up sTESLA/XRD swap pool");
            let (radiswap_comp, lp_tokens) = Radiswap::instantiate_pool(
                synth,
                initial_xrd,
                1000000.into(),
                "LP".to_owned(),
                "LP Token".to_owned(),
                "https://example.com/".to_owned(),
                "0.003".parse().unwrap(),
            );

            debug!("Mint initial shares");
            let mut mutual_farm_share_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "MutualFarm share")
                .flags(MINTABLE | BURNABLE)
                .badge(identity_badge_address, MAY_MINT | MAY_BURN)
                .no_initial_supply();
            let shares =
                mutual_farm_share_resource_def.mint(initial_shares, identity_badge.present());

            debug!("Instantiate MutualFund component");
            let component = Self {
                identity_badge: Vault::with_bucket(identity_badge),
                price_oracle,
                xrd_snx_radiswap,
                synthetic_pool,
                asset_symbol,
                asset_address,
                synth_address,
                snx_address,
                usd_address,
                radiswap: radiswap_comp.into(),
                radiswap_lp_tokens: Vault::with_bucket(lp_tokens),
                mutual_farm_share_resource_def,
                total_contribution_in_usd: xrd_amount * xrd_usd_price,
            }
            .instantiate();

            (shares, component)
        }

        pub fn deposit(&mut self, mut xrd: Bucket) -> (Bucket, Bucket) {
            debug!("Fetch price info from oracle");
            let xrd_usd_price = self
                .price_oracle
                .get_price(xrd.resource_address(), self.usd_address)
                .unwrap();
            let snx_usd_price = self
                .price_oracle
                .get_price(self.snx_address, self.usd_address)
                .unwrap();
            let tesla_usd_price = self
                .price_oracle
                .get_price(self.asset_address, self.usd_address)
                .unwrap();

            debug!("Swap 3/4 of XRD for SNX");
            let xrd_address = xrd.resource_def();
            let xrd_amount = xrd.amount();
            let snx = self.xrd_snx_radiswap.swap(xrd.take(xrd.amount() * 3 / 4));
            let snx_amount = snx.amount();

            debug!("Deposit SNX into synthetic pool and mint sTESLA (1/10 of our SNX).");
            self.identity_badge.authorize(|auth| {
                self.synthetic_pool.stake(auth, snx);
            });
            let quantity = snx_amount * snx_usd_price / 10 / tesla_usd_price;
            let synth = self.identity_badge.authorize(|auth| {
                self.synthetic_pool
                    .mint(auth, quantity, self.asset_symbol.clone())
            });

            debug!("Add liquidity to sTESLA/XRD swap pool");
            let (lp_tokens, mut remainder) = self.radiswap.add_liquidity(synth, xrd);
            if remainder.resource_address() == self.synth_address {
                self.identity_badge.authorize(|auth| {
                    self.synthetic_pool.burn(auth, remainder);
                });
                remainder = Bucket::new(xrd_address);
            }
            self.radiswap_lp_tokens.put(lp_tokens);

            debug!("Mint initial shares");
            let contribution = xrd_usd_price * (xrd_amount - remainder.amount());
            let num_shares_to_issue = contribution
                / (self.total_contribution_in_usd / self.mutual_farm_share_resource_def.total_supply());
            self.total_contribution_in_usd += contribution;
            let shares = self.identity_badge.authorize(|auth| {
                self.mutual_farm_share_resource_def
                    .mint(num_shares_to_issue, auth)
            });
            (shares, remainder)
        }

        pub fn withdraw(&mut self) -> (Bucket, Bucket) {
            todo!()
        }
    }
}
