use radix_engine::types::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::*;

use crate::internal_prelude::*;

pub struct RadiswapScenario {
    core: ScenarioCore,
    config: RadiswapScenarioConfig,
}

pub struct RadiswapScenarioConfig {
    /* Accounts */
    pub radiswap_owner: VirtualAccount,
    pub storing_account: VirtualAccount,
    pub user_account_1: VirtualAccount,
    pub user_account_2: VirtualAccount,
    pub user_account_3: VirtualAccount,

    /* Resources & Pools - These get created during the scenario */
    pub pool_1: PoolData,
    pub pool_2: PoolData,
}

#[derive(Default)]
pub struct PoolData {
    radiswap: Option<ComponentAddress>,
    pool: Option<ComponentAddress>,
    resource_1: Option<ResourceAddress>,
    resource_2: Option<ResourceAddress>,
    pool_unit: Option<ResourceAddress>,
}

impl PoolData {
    fn radiswap(&self) -> ComponentAddress {
        self.radiswap.unwrap()
    }

    fn pool(&self) -> ComponentAddress {
        self.pool.unwrap()
    }

    fn resource_1(&self) -> ResourceAddress {
        self.resource_1.unwrap()
    }

    fn resource_2(&self) -> ResourceAddress {
        self.resource_2.unwrap()
    }

    fn pool_unit(&self) -> ResourceAddress {
        self.pool_unit.unwrap()
    }
}

impl Default for RadiswapScenarioConfig {
    fn default() -> Self {
        Self {
            radiswap_owner: secp256k1_account_1(),
            storing_account: secp256k1_account_2(),
            user_account_1: secp256k1_account_3(),
            user_account_2: ed25519_account_1(),
            user_account_3: ed25519_account_2(),
            pool_1: Default::default(),
            pool_2: Default::default(),
        }
    }
}

impl ScenarioDefinition for RadiswapScenario {
    type Config = RadiswapScenarioConfig;

    fn new_with_config(core: ScenarioCore, config: Self::Config) -> Self {
        Self { core, config }
    }
}

impl ScenarioInstance for RadiswapScenario {
    fn metadata(&self) -> ScenarioMetadata {
        ScenarioMetadata {
            logical_name: "radiswap",
        }
    }

    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError> {
        let radiswap_owner = &self.config.radiswap_owner;
        let storing_account = &self.config.storing_account;
        let user_account_1 = &self.config.user_account_1;
        let user_account_2 = &self.config.user_account_2;
        let user_account_3 = &self.config.user_account_3;
        let pool_1 = &mut self.config.pool_1;
        let pool_2 = &mut self.config.pool_2;
        let core = &mut self.core;

        let up_next = match core.next_stage() {
            1 => {
                core.check_start(&previous)?;
                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-create-new-resources",
                    |builder| {
                        builder.create_fungible_resource(
                            OwnerRole::None,
                            false,
                            18,
                            metadata! {
                                init {
                                    "name" => "Bitcoin".to_owned(), locked;
                                    "symbol" => "BTC".to_owned(), locked;
                                    "description" => "A peer to peer decentralized proof of work network.".to_owned(), locked;
                                    "tags" => vec!["p2p".to_owned(), "blockchain".to_owned()], locked;
                                    "icon_url" => "https://www.example.com/".to_owned(), locked;
                                    "info_url" => "https://www.example.com/".to_owned(), locked;
                                }
                            },
                            btreemap! {
                                Mint => (rule!(deny_all), rule!(deny_all)),
                                Burn => (rule!(allow_all), rule!(deny_all))
                            },
                            Some(100_000_000_000u64.into()),
                        )
                        .create_fungible_resource(
                            OwnerRole::None,
                            true,
                            18,
                            metadata! {
                                init {
                                    "name" => "Ethereum".to_owned(), locked;
                                    "symbol" => "ETH".to_owned(), locked;
                                    "description" => "The native token of the Ethereum blockchain".to_owned(), locked;
                                    "tags" => vec!["p2p".to_owned(), "blockchain".to_owned(), "gas".to_owned()], locked;
                                    "icon_url" => "https://www.example.com/".to_owned(), locked;
                                    "info_url" => "https://www.example.com/".to_owned(), locked;
                                }
                            },
                            btreemap! {
                                Mint => (rule!(deny_all), rule!(deny_all)),
                                Burn => (rule!(allow_all), rule!(deny_all))
                            },
                            Some(100_000_000_000u64.into()),
                        )
                        .create_fungible_resource(
                            OwnerRole::None,
                            true,
                            18,
                            metadata! {
                                init {
                                    "name" => "Ethereum".to_owned(), locked;
                                    "symbol" => "ETC".to_owned(), locked;
                                    "description" => "The native token of the Ethereum Classic blockchain".to_owned(), locked;
                                    "tags" => vec!["p2p".to_owned(), "blockchain".to_owned(), "gas".to_owned()], locked;
                                    "icon_url" => "https://www.example.com/".to_owned(), locked;
                                    "info_url" => "https://www.example.com/".to_owned(), locked;
                                }
                            },
                            btreemap! {
                                Mint => (rule!(deny_all), rule!(deny_all)),
                                Burn => (rule!(allow_all), rule!(deny_all))
                            },
                            Some(100_000_000_000u64.into()),
                        )
                        .try_deposit_batch_or_abort(storing_account.address)
                    },
                    vec![],
                )
            }
            2 => {
                let commit_success = core.check_commit_success(&previous)?;
                let new_resources = commit_success.new_resource_addresses();

                pool_1.resource_1 = Some(RADIX_TOKEN);
                pool_1.resource_2 = Some(new_resources[0]);
                pool_2.resource_1 = Some(new_resources[1]);
                pool_2.resource_2 = Some(new_resources[2]);

                let code = include_bytes!("../../../assets/radiswap.wasm");
                let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                    "../../../assets/radiswap.schema"
                ))
                .unwrap();

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-publish-and-create-pools",
                    |builder| {
                        builder.allocate_global_address(
                            BlueprintId {
                                package_address: PACKAGE_PACKAGE,
                                blueprint_name: PACKAGE_BLUEPRINT.to_owned(),
                            },
                            |builder, reservation, named_address| {
                                builder
                                    .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                                    .publish_package_advanced(
                                        Some(reservation),
                                        code.to_vec(),
                                        schema,
                                        metadata_init! {
                                            "name" => "Radiswap Package".to_owned(), locked;
                                            "description" => "A package of the logic of a Uniswap v2 style DEX.".to_owned(), locked;
                                            "tags" => vec!["dex".to_owned(), "pool".to_owned(), "radiswap".to_owned()], locked;
                                        },
                                        radix_engine::types::OwnerRole::Fixed(rule!(require(
                                            NonFungibleGlobalId::from_public_key(
                                                &radiswap_owner.public_key
                                            )
                                        ))),
                                    ).call_function(
                                        DynamicPackageAddress::Named(named_address),
                                        "Radiswap", 
                                        "new", 
                                        manifest_args!(pool_1.resource_1(), pool_1.resource_2())
                                    )
                                    .call_function(
                                        DynamicPackageAddress::Named(named_address),
                                        "Radiswap", 
                                        "new", 
                                        manifest_args!(pool_2.resource_1(), pool_2.resource_2())
                                    )
                                    .try_deposit_batch_or_abort(storing_account.address)
                            },
                        )
                    },
                    vec![],
                )
            }
            3 => {
                {
                    let commit_success = core.check_commit_success(&previous)?;
                    let new_components = commit_success.new_component_addresses();
                    let new_resources = commit_success.new_resource_addresses();
                    pool_1.radiswap = Some(new_components[0]);
                    pool_1.pool = Some(new_components[1]);
                    pool_2.radiswap = Some(new_components[2]);
                    pool_2.pool = Some(new_components[3]);

                    pool_1.pool_unit = Some(new_resources[0]);
                    pool_2.pool_unit = Some(new_resources[1]);
                }

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-add-liquidity",
                    |builder| {
                        builder
                            .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                            .withdraw_from_account(
                                storing_account.address,
                                pool_1.resource_2(),
                                7000.into(),
                            )
                            .withdraw_from_account(
                                storing_account.address,
                                pool_2.resource_1(),
                                5000.into(),
                            )
                            .withdraw_from_account(
                                storing_account.address,
                                pool_2.resource_2(),
                                8000.into(),
                            )
                            .take_all_from_worktop(pool_1.resource_1(), |builder, bucket1| {
                                builder.take_all_from_worktop(
                                    pool_1.resource_2(),
                                    |builder, bucket2| {
                                        builder.call_method(
                                            pool_1.radiswap(),
                                            "add_liquidity",
                                            manifest_args!(bucket1, bucket2),
                                        )
                                    },
                                )
                            })
                            .take_all_from_worktop(pool_2.resource_1(), |builder, bucket1| {
                                builder.take_all_from_worktop(
                                    pool_2.resource_2(),
                                    |builder, bucket2| {
                                        builder.call_method(
                                            pool_2.radiswap(),
                                            "add_liquidity",
                                            manifest_args!(bucket1, bucket2),
                                        )
                                    },
                                )
                            })
                            .try_deposit_batch_or_abort(storing_account.address)
                    },
                    vec![&storing_account.key],
                )
            }
            4 => {
                core.check_commit_success(&previous)?;

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-distribute-tokens",
                    |builder| {
                        builder.call_method(FAUCET, "free", manifest_args!());
                        for destination_account in [user_account_1, user_account_2, user_account_3]
                        {
                            for resource_address in [
                                pool_1.resource_1(),
                                pool_1.resource_2(),
                                pool_2.resource_1(),
                                pool_2.resource_2(),
                                pool_1.pool_unit(),
                                pool_2.pool_unit(),
                            ] {
                                builder.withdraw_from_account(
                                    storing_account.address,
                                    resource_address,
                                    333.into(),
                                );
                            }
                            builder.try_deposit_batch_or_abort(destination_account.address);
                        }
                        builder
                    },
                    vec![&storing_account.key],
                )
            }
            5 => {
                core.check_commit_success(&previous)?;

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-swap-tokens",
                    |builder| {
                        builder
                            .withdraw_from_account(
                                user_account_1.address,
                                pool_1.resource_1(),
                                100.into(),
                            )
                            .take_all_from_worktop(pool_1.resource_1(), |builder, bucket| {
                                builder.call_method(
                                    pool_1.radiswap(),
                                    "swap",
                                    manifest_args!(bucket),
                                )
                            })
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            6 => {
                core.check_commit_success(&previous)?;

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-remove-tokens",
                    |builder| {
                        builder
                            .withdraw_from_account(
                                user_account_1.address,
                                pool_1.pool_unit(),
                                100.into(),
                            )
                            .take_all_from_worktop(pool_1.pool_unit(), |builder, bucket| {
                                builder.call_method(
                                    pool_1.radiswap(),
                                    "remove_liquidity",
                                    manifest_args!(bucket),
                                )
                            })
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            _ => {
                core.check_commit_success(&previous)?;
                // Re-deconstruct the config in order to ensure at compile time we capture all the addresses
                let RadiswapScenarioConfig {
                    radiswap_owner,
                    storing_account,
                    user_account_1,
                    user_account_2,
                    user_account_3,
                    pool_1,
                    pool_2,
                } = &self.config;
                let addresses = DescribedAddresses::new()
                    .add("radiswap_owner", radiswap_owner)
                    .add("storing_account", storing_account)
                    .add("user_account_1", user_account_1)
                    .add("user_account_2", user_account_2)
                    .add("user_account_3", user_account_3)
                    .add("pool_1_radiswap", pool_1.radiswap())
                    .add("pool_1_pool", pool_1.pool())
                    .add("pool_1_resource_1", pool_1.resource_1())
                    .add("pool_1_resource_2", pool_1.resource_2())
                    .add("pool_1_pool_unit", pool_1.pool_unit())
                    .add("pool_2_radiswap", pool_2.radiswap())
                    .add("pool_2_pool", pool_2.pool())
                    .add("pool_2_resource_1", pool_2.resource_1())
                    .add("pool_2_resource_2", pool_2.resource_2())
                    .add("pool_2_pool_unit", pool_2.pool_unit());
                return Ok(core.finish_scenario(addresses));
            }
        };
        Ok(NextAction::Transaction(up_next))
    }
}
