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
    "package_ref": "014eb598fe6ed7df56a5f02950df2d7b08530d9d1081f05a6398f9",
    "blueprint_name": "PriceOracle",
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
                        "name": "Bucket",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "ComponentRef",
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
                    "name": "ResourceDefRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "ResourceDefRef",
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
                    "name": "ResourceDefRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "ResourceDefRef",
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
                "name": "ResourceDefRef",
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
    "package_ref": "01e0983e33158b489e70313b77767abe80eb449e6acd46f9476328",
    "blueprint_name": "SyntheticPool",
    "functions": [
        {
            "name": "instantiate_pool",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "ComponentRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "ResourceDefRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "ResourceDefRef",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "ComponentRef",
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
                    "name": "ResourceDefRef",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "ResourceDefRef",
                "generics": []
            }
        },
        {
            "name": "stake",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "Proof",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Bucket",
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
                    "name": "Proof",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "Bucket",
                "generics": []
            }
        },
        {
            "name": "mint",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "Proof",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Decimal",
                    "generics": []
                },
                {
                    "type": "String"
                }
            ],
            "output": {
                "type": "Custom",
                "name": "Bucket",
                "generics": []
            }
        },
        {
            "name": "burn",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "Proof",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Bucket",
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
                "name": "Decimal",
                "generics": []
            }
        },
        {
            "name": "get_snx_price",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "Decimal",
                "generics": []
            }
        },
        {
            "name": "get_asset_price",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "ResourceDefRef",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "Decimal",
                "generics": []
            }
        },
        {
            "name": "get_user_summary",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "ResourceDefRef",
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
                "name": "Bucket",
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
    "package_ref": "01899b2991b37ee1bc51a84182a3752c1dc48ec3df01969d3516d3",
    "blueprint_name": "Radiswap",
    "functions": [
        {
            "name": "instantiate_pool",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "Bucket",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Bucket",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Decimal",
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
                    "name": "Decimal",
                    "generics": []
                }
            ],
            "output": {
                "type": "Tuple",
                "elements": [
                    {
                        "type": "Custom",
                        "name": "ComponentRef",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "Bucket",
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
                    "name": "Bucket",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "Bucket",
                    "generics": []
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
                        "name": "Bucket",
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
                    "name": "Bucket",
                    "generics": []
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
                        "name": "Bucket",
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
                    "name": "Bucket",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "Bucket",
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
        /// Asset resource definition
        asset_resource_def_ref: ResourceDefRef,
        /// Synthetic resource definition
        synth_resource_def_ref: ResourceDefRef,
        /// SNX resource definition
        snx_resource_def_ref: ResourceDefRef,
        /// USD resource definition
        usd_resource_def_ref: ResourceDefRef,

        /// Radiswap for sTESLA/XRD
        radiswap: Radiswap,
        /// Radiswap LP token vault
        radiswap_lp_tokens: Vault,

        /// Mutual farm share resource definition
        mutual_farm_share: ResourceDefRef,
        /// Total contribution
        total_contribution_in_usd: Decimal,
    }

    impl MutualFarm {
        pub fn instantiate_farm(
            price_oracle: ComponentRef,
            xrd_snx_radiswap: ComponentRef,
            synthetic_pool: ComponentRef,
            asset_symbol: String,
            asset_resource_def_ref: ResourceDefRef,
            initial_shares: Decimal,
            mut initial_xrd: Bucket,
            snx_resource_def_ref: ResourceDefRef,
            usd_resource_def_ref: ResourceDefRef,
        ) -> (Bucket, ComponentRef) {
            debug!("Create an identity badge for accessing other components");
            let identity_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "ID")
                .initial_supply_fungible(1);

            debug!("Fetch price info from oracle");
            let price_oracle: PriceOracle = price_oracle.into();
            let xrd_usd_price = price_oracle
                .get_price(initial_xrd.resource_def_ref(), usd_resource_def_ref)
                .unwrap();
            let snx_usd_price = price_oracle
                .get_price(snx_resource_def_ref, usd_resource_def_ref)
                .unwrap();
            let tesla_usd_price = price_oracle
                .get_price(asset_resource_def_ref, usd_resource_def_ref)
                .unwrap();

            debug!("Swap 3/4 of XRD for SNX");
            let xrd_snx_radiswap: Radiswap = xrd_snx_radiswap.into();
            let xrd_amount = initial_xrd.amount();
            let snx = xrd_snx_radiswap.swap(initial_xrd.take(initial_xrd.amount() * 3 / 4));
            let snx_amount = snx.amount();

            debug!("Deposit SNX into synthetic pool and mint sTESLA (1/10 of our SNX).");
            let price_oracle: PriceOracle = price_oracle.into();
            let synthetic_pool: SyntheticPool = synthetic_pool.into();
            synthetic_pool.add_synthetic_token(asset_symbol.clone(), asset_resource_def_ref);
            synthetic_pool.stake(identity_badge.present(), snx);
            let quantity = snx_amount * snx_usd_price / 10 / tesla_usd_price;
            let synth =
                synthetic_pool.mint(identity_badge.present(), quantity, asset_symbol.clone());
            let synth_resource_def_ref = synth.resource_def_ref();

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
            let mut mutual_farm_share = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "MutualFarm share")
                .flags(MINTABLE | BURNABLE)
                .badge(identity_badge.resource_def_ref(), MAY_MINT | MAY_BURN)
                .no_initial_supply();
            let shares = mutual_farm_share.mint(initial_shares, identity_badge.present());

            debug!("Instantiate MutualFund component");
            let component = Self {
                identity_badge: Vault::with_bucket(identity_badge),
                price_oracle,
                xrd_snx_radiswap,
                synthetic_pool,
                asset_symbol,
                asset_resource_def_ref,
                synth_resource_def_ref,
                snx_resource_def_ref,
                usd_resource_def_ref,
                radiswap: radiswap_comp.into(),
                radiswap_lp_tokens: Vault::with_bucket(lp_tokens),
                mutual_farm_share,
                total_contribution_in_usd: xrd_amount * xrd_usd_price,
            }
            .instantiate();

            (shares, component)
        }

        pub fn deposit(&mut self, mut xrd: Bucket) -> (Bucket, Bucket) {
            debug!("Fetch price info from oracle");
            let xrd_usd_price = self
                .price_oracle
                .get_price(xrd.resource_def_ref(), self.usd_resource_def_ref)
                .unwrap();
            let snx_usd_price = self
                .price_oracle
                .get_price(self.snx_resource_def_ref, self.usd_resource_def_ref)
                .unwrap();
            let tesla_usd_price = self
                .price_oracle
                .get_price(self.asset_resource_def_ref, self.usd_resource_def_ref)
                .unwrap();

            debug!("Swap 3/4 of XRD for SNX");
            let xrd_resource_def_ref = xrd.resource_def_ref();
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
            if remainder.resource_def_ref() == self.synth_resource_def_ref {
                self.identity_badge.authorize(|auth| {
                    self.synthetic_pool.burn(auth, remainder);
                });
                remainder = Bucket::new(xrd_resource_def_ref);
            }
            self.radiswap_lp_tokens.put(lp_tokens);

            debug!("Mint initial shares");
            let contribution = xrd_usd_price * (xrd_amount - remainder.amount());
            let num_shares_to_issue = contribution
                / (self.total_contribution_in_usd / self.mutual_farm_share.total_supply());
            self.total_contribution_in_usd += contribution;
            let shares = self
                .identity_badge
                .authorize(|auth| self.mutual_farm_share.mint(num_shares_to_issue, auth));
            (shares, remainder)
        }

        pub fn withdraw(&mut self) -> (Bucket, Bucket) {
            todo!()
        }
    }
}
