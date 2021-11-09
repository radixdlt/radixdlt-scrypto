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
    "package": "01e61370219353678d8bfb37bce521e257d0ec29ae9a2e95d194ea",
    "name": "PriceOracle",
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
    "package": "01d25d4eab30b60d9951f3433b35cff52a48f8cf163b66c0a16677",
    "name": "SyntheticPool",
    "functions": [
        {
            "name": "new",
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
    "package": "01f91f613875b4326060eac0bcc0c98c0eaad15eb1c9c51ace0401",
    "name": "Radiswap",
    "functions": [
        {
            "name": "new",
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
        pub fn new(
            price_oracle_address: Address,
            xrd_snx_radiswap_address: Address,
            synthetic_pool_address: Address,
            asset_symbol: String,
            asset_address: Address,
            initial_shares: Decimal,
            initial_xrd: Bucket,
            snx_address: Address,
            usd_address: Address,
        ) -> (Bucket, Component) {
            debug!("Create an identity badge for accessing other components");
            let identity_badge = ResourceBuilder::new().new_badge_fixed(1);
            let identity_badge_address = identity_badge.resource_def().address();

            debug!("Fetch price info from oracle");
            let price_oracle: PriceOracle = price_oracle_address.into();
            let xrd_usd_price = price_oracle
                .get_price(initial_xrd.resource_def().address(), usd_address)
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
            synthetic_pool.stake(identity_badge.borrow(), snx);
            let quantity = snx_amount * snx_usd_price / 10 / tesla_usd_price;
            let synth =
                synthetic_pool.mint(identity_badge.borrow(), quantity, asset_symbol.clone());
            let synth_address = synth.resource_def().address();

            debug!("Set up sTESLA/XRD swap pool");
            let (radiswap_comp, lp_tokens) = Radiswap::new(
                synth,
                initial_xrd,
                1000000.into(),
                "LP".to_owned(),
                "LP Token".to_owned(),
                "https://example.com/".to_owned(),
                "0.003".parse().unwrap(),
            );

            debug!("Mint initial shares");
            let mutual_farm_share_resource_def = ResourceBuilder::new()
                .metadata("name", "MutualFarm share")
                .new_token_mutable(identity_badge_address);
            let shares =
                mutual_farm_share_resource_def.mint(initial_shares, identity_badge.borrow());

            debug!("Instantiate MutualFund component");
            let component = Self {
                identity_badge: Vault::with_bucket(identity_badge),
                price_oracle,
                xrd_snx_radiswap,
                synthetic_pool,
                asset_symbol,
                asset_address,
                synth_address,
                radiswap: radiswap_comp.into(),
                radiswap_lp_tokens: Vault::with_bucket(lp_tokens),
                mutual_farm_share_resource_def,
                total_contribution_in_usd: xrd_amount * xrd_usd_price,
            }
            .instantiate();

            (shares, component)
        }

        pub fn deposit(&mut self, _xrd_bucket: Bucket) -> Bucket {
            todo!()
        }

        pub fn withdraw(&mut self) -> (Bucket, Bucket) {
            todo!()
        }
    }
}
