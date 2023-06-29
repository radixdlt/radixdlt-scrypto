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
    pub user_account1: VirtualAccount,
    pub user_account2: VirtualAccount,
    pub user_account3: VirtualAccount,

    /* Resources & Pools */
    pub pools: Option<
        [(
            // Pool Component Address
            Option<ComponentAddress>,
            // Pool Resources
            (ResourceAddress, ResourceAddress),
            // Pool Unit
            Option<ResourceAddress>,
        ); 2],
    >,
}

impl RadiswapScenarioConfig {
    fn pools_or_panic(
        &self,
    ) -> [(
        ComponentAddress,
        (ResourceAddress, ResourceAddress),
        ResourceAddress,
    ); 2] {
        self.pools.as_ref().unwrap().map(
            |(component_address, (resource1, resource2), pool_unit)| {
                (
                    component_address.unwrap(),
                    (resource1, resource2),
                    pool_unit.unwrap(),
                )
            },
        )
    }
}

impl Default for RadiswapScenarioConfig {
    fn default() -> Self {
        Self {
            radiswap_owner: secp256k1_account_1(),
            storing_account: secp256k1_account_2(),
            user_account1: secp256k1_account_3(),
            user_account2: ed25519_account_1(),
            user_account3: ed25519_account_2(),
            pools: None,
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
        let user_account1 = &self.config.user_account1;
        let user_account2 = &self.config.user_account2;
        let user_account3 = &self.config.user_account3;
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
                            btreeset!(Burn),
                            roles_init! {
                                BURNER_ROLE => rule!(allow_all), locked;
                            },
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
                            Some(100_000_000_000u64.into()),
                        )
                        .create_fungible_resource(
                            OwnerRole::None,
                            true,
                            18,
                            btreeset!(Burn),
                            roles_init! {
                                BURNER_ROLE => rule!(allow_all), locked;
                            },
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
                            Some(100_000_000_000u64.into()),
                        )
                        .create_fungible_resource(
                            OwnerRole::None,
                            true,
                            18,
                            btreeset!(Burn),
                            roles_init! {
                                BURNER_ROLE => rule!(allow_all), locked;
                            },
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
                            Some(100_000_000_000u64.into()),
                        )
                        .try_deposit_batch_or_abort(storing_account.address)
                    },
                    vec![],
                )
            }
            2 => {
                let commit_success = core.check_commit_success(&previous)?;

                let resource1 = RADIX_TOKEN;
                let resource2 = commit_success
                    .new_resource_addresses()
                    .get(0)
                    .unwrap()
                    .clone();
                let resource3 = commit_success
                    .new_resource_addresses()
                    .get(1)
                    .unwrap()
                    .clone();
                let resource4 = commit_success
                    .new_resource_addresses()
                    .get(2)
                    .unwrap()
                    .clone();

                self.config.pools = Some([
                    (None, (resource1, resource2), None),
                    (None, (resource3, resource4), None),
                ]);

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
                                        manifest_args!(resource1, resource2)
                                    )
                                    .call_function(
                                        DynamicPackageAddress::Named(named_address),
                                        "Radiswap", 
                                        "new", 
                                        manifest_args!(resource3, resource4)
                                    )
                                    .try_deposit_batch_or_abort(storing_account.address)
                            },
                        )
                    },
                    vec![],
                )
            }
            3 => {
                let commit_success = core.check_commit_success(&previous)?;

                {
                    let pool1 = commit_success
                        .new_component_addresses()
                        .get(0)
                        .unwrap()
                        .clone();
                    let pool2 = commit_success
                        .new_component_addresses()
                        .get(2)
                        .unwrap()
                        .clone();

                    let pool_unit1 = commit_success
                        .new_resource_addresses()
                        .get(0)
                        .unwrap()
                        .clone();
                    let pool_unit2 = commit_success
                        .new_resource_addresses()
                        .get(1)
                        .unwrap()
                        .clone();

                    let pools = self.config.pools.as_mut().unwrap();
                    pools[0].0 = Some(pool1);
                    pools[1].0 = Some(pool2);

                    pools[0].2 = Some(pool_unit1);
                    pools[1].2 = Some(pool_unit2);
                }
                let [(pool1, (resource1, resource2), _), (pool2, (resource3, resource4), _)] =
                    self.config.pools_or_panic();

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-add-liquidity",
                    |builder| {
                        builder
                            .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                            .withdraw_from_account(storing_account.address, resource2, 7000.into())
                            .withdraw_from_account(storing_account.address, resource3, 5000.into())
                            .withdraw_from_account(storing_account.address, resource4, 8000.into())
                            .take_all_from_worktop(resource1, |builder, bucket1| {
                                builder.take_all_from_worktop(resource2, |builder, bucket2| {
                                    builder.call_method(
                                        pool1,
                                        "add_liquidity",
                                        manifest_args!(bucket1, bucket2),
                                    )
                                })
                            })
                            .take_all_from_worktop(resource3, |builder, bucket1| {
                                builder.take_all_from_worktop(resource4, |builder, bucket2| {
                                    builder.call_method(
                                        pool2,
                                        "add_liquidity",
                                        manifest_args!(bucket1, bucket2),
                                    )
                                })
                            })
                            .try_deposit_batch_or_abort(storing_account.address)
                    },
                    vec![&storing_account.key],
                )
            }
            4 => {
                core.check_commit_success(&previous)?;

                let [(_, (resource1, resource2), pool_unit1), (_, (resource3, resource4), pool_unit2)] =
                    self.config.pools_or_panic();

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-distribute-tokens",
                    |builder| {
                        builder.call_method(FAUCET, "free", manifest_args!());
                        for destination_account in [user_account1, user_account2, user_account3] {
                            for resource_address in [
                                resource1, resource2, resource3, resource4, pool_unit1, pool_unit2,
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
                let [(pool1, (resource1, _), _), _] = self.config.pools_or_panic();

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-swap-tokens",
                    |builder| {
                        builder
                            .withdraw_from_account(user_account1.address, resource1, 100.into())
                            .take_all_from_worktop(resource1, |builder, bucket| {
                                builder.call_method(pool1, "swap", manifest_args!(bucket))
                            })
                            .try_deposit_batch_or_abort(user_account1.address)
                    },
                    vec![&user_account1.key],
                )
            }
            6 => {
                core.check_commit_success(&previous)?;
                let [(pool1, (_, _), pool_unit1), _] = self.config.pools_or_panic();

                core.next_transaction_with_faucet_lock_fee(
                    "radiswap-swap-tokens",
                    |builder| {
                        builder
                            .withdraw_from_account(user_account1.address, pool_unit1, 100.into())
                            .take_all_from_worktop(pool_unit1, |builder, bucket| {
                                builder.call_method(
                                    pool1,
                                    "remove_liquidity",
                                    manifest_args!(bucket),
                                )
                            })
                            .try_deposit_batch_or_abort(user_account1.address)
                    },
                    vec![&user_account1.key],
                )
            }
            _ => {
                core.check_commit_success(&previous)?;
                return Ok(core.finish_scenario());
            }
        };
        Ok(NextAction::Transaction(up_next))
    }
}
