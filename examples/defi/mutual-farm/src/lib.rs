use scrypto::prelude::*;

import! {
r#"
{
    "package": "01024420d4c8749579abc13133bf07b0a4fc307aa0172f595a0245",
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
    "package": "01205eedae4ac21cbdc07728bf934d6c0b253cdec0439f867e6bee",
    "name": "AutoLend",
    "functions": [
        {
            "name": "new",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
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
            "name": "new_user",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
            }
        },
        {
            "name": "deposit",
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
            "name": "redeem",
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
            "name": "borrow",
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
            "name": "repay",
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
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
            }
        },
        {
            "name": "liquidate",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                },
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
        },
        {
            "name": "get_user",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                }
            ],
            "output": {
                "type": "Struct",
                "name": "User",
                "fields": {
                    "type": "Named",
                    "named": [
                        [
                            "deposit_balance",
                            {
                                "type": "Custom",
                                "name": "scrypto::types::Decimal",
                                "generics": []
                            }
                        ],
                        [
                            "deposit_interest_rate",
                            {
                                "type": "Custom",
                                "name": "scrypto::types::Decimal",
                                "generics": []
                            }
                        ],
                        [
                            "deposit_last_update",
                            {
                                "type": "U64"
                            }
                        ],
                        [
                            "borrow_balance",
                            {
                                "type": "Custom",
                                "name": "scrypto::types::Decimal",
                                "generics": []
                            }
                        ],
                        [
                            "borrow_interest_rate",
                            {
                                "type": "Custom",
                                "name": "scrypto::types::Decimal",
                                "generics": []
                            }
                        ],
                        [
                            "borrow_last_update",
                            {
                                "type": "U64"
                            }
                        ]
                    ]
                }
            }
        },
        {
            "name": "set_deposit_interest_rate",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Unit"
            }
        },
        {
            "name": "set_borrow_interest_rate",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Unit"
            }
        }
    ]
}
"#
}

import! {
r#"
{
    "package": "01a78cfec3dac583cc2394d14452099892a5af4a5201d771d918a2",
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
    "package": "01d25d4eab30b60d9951f3433b35cff52a48f8cf163b66c0a16677",
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
        /// Price Oracle
        price_oracle: PriceOracle,
        /// AutoLend component for borrowing SNX with XRD as collateral
        auto_lend: AutoLend,
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
    }

    impl MutualFarm {
        pub fn new(
            price_oracle_address: Address,
            auto_lend_address: Address,
            synthetic_pool_address: Address,
            asset_symbol: String,
            asset_address: Address,
            initial_xrd: Bucket,
            snx_address: Address,
            num_of_tesla: Decimal,
        ) -> Component {
            debug!("Create an identity badge for accessing other components");
            let identity_badge = ResourceBuilder::new().new_badge_fixed(1);
            let identity_badge_address = identity_badge.resource_def().address();

            debug!("Calculate how much the initial XRD worths in SNX");
            let price_oracle: PriceOracle = price_oracle_address.into();
            let xrd_snx_price = price_oracle
                .get_price(initial_xrd.resource_def().address(), snx_address)
                .unwrap();
            let xrd_amount = initial_xrd.amount();
            let xrd_in_snx = &xrd_amount * &xrd_snx_price;

            debug!(
                "Deposit half of the XRD into AutoLend and borrow SNX with 200% collateral ratio"
            );
            let auto_lend: AutoLend = auto_lend_address.into();
            auto_lend.deposit(identity_badge.borrow(), initial_xrd.take(xrd_amount / 2));
            let snx = auto_lend.borrow(identity_badge.borrow(), xrd_in_snx / 4); // Not working, as AutoLend only provides loans in XRD

            debug!("Deposit SNX into synthetic pool and mint sTESLA.");
            let synthetic_pool: SyntheticPool = synthetic_pool_address.into();
            synthetic_pool.add_synthetic_token(asset_symbol.clone(), asset_address);
            synthetic_pool.stake(identity_badge.borrow(), snx);
            let synth =
                synthetic_pool.mint(identity_badge.borrow(), num_of_tesla, asset_symbol.clone());
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

            Self {
                identity_badge: Vault::with_bucket(identity_badge),
                price_oracle,
                auto_lend,
                synthetic_pool,
                asset_symbol,
                asset_address,
                synth_address,
                radiswap: radiswap_comp.into(),
                radiswap_lp_tokens: Vault::with_bucket(lp_tokens),
                mutual_farm_share_resource_def: ResourceBuilder::new()
                    .metadata("name", "MutualFarm share")
                    .new_token_mutable(identity_badge_address),
            }
            .instantiate()
        }

        pub fn deposit(&mut self, _xrd_bucket: Bucket) -> Bucket {
            todo!()
        }

        pub fn withdraw(&mut self) -> (Bucket, Bucket) {
            todo!()
        }
    }
}
