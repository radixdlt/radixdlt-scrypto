use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::RoleDefinition;
use radix_engine_interface::api::node_modules::auth::ToRoleEntry;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::*;

use crate::internal_prelude::*;

pub struct RadiswapScenarioConfig {
    pub radiswap_dapp_definition_account: VirtualAccount,
    pub storing_account: VirtualAccount,
    pub user_account_1: VirtualAccount,
    pub user_account_2: VirtualAccount,
    pub user_account_3: VirtualAccount,
}

impl Default for RadiswapScenarioConfig {
    fn default() -> Self {
        Self {
            radiswap_dapp_definition_account: ed25519_account_for_private_key(891231),
            storing_account: secp256k1_account_2(),
            user_account_1: secp256k1_account_3(),
            user_account_2: ed25519_account_1(),
            user_account_3: ed25519_account_2(),
        }
    }
}

#[derive(Default)]
pub struct RadiswapScenarioState {
    owner_badge: State<NonFungibleGlobalId>,
    radiswap_package: State<PackageAddress>,
    pool_1: PoolData,
    pool_2: PoolData,
}

#[derive(Default)]
pub struct PoolData {
    radiswap: State<ComponentAddress>,
    pool: State<ComponentAddress>,
    resource_1: State<ResourceAddress>,
    resource_2: State<ResourceAddress>,
    pool_unit: State<ResourceAddress>,
}

pub struct RadiswapScenarioCreator;

impl ScenarioCreator for RadiswapScenarioCreator {
    type Config = RadiswapScenarioConfig;
    type State = RadiswapScenarioState;

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "radiswap",
        };

        #[allow(unused_variables)]
        ScenarioBuilder::new(core, metadata, config, start_state)
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-create-new-resources",
                        |builder| {
                            builder.create_fungible_resource(
                                OwnerRole::None,
                                false,
                                18,
                                FungibleResourceRoles {
                                    burn_roles: burn_roles! {
                                    burner => rule!(allow_all);
                                    burner_updater => rule!(deny_all);
                                },
                                    ..Default::default()
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
                                FungibleResourceRoles {
                                    burn_roles: burn_roles! {
                                    burner => rule!(allow_all);
                                    burner_updater => rule!(deny_all);
                                },
                                    ..Default::default()
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
                                FungibleResourceRoles {
                                    burn_roles: burn_roles! {
                                    burner => rule!(allow_all);
                                    burner_updater => rule!(deny_all);
                                },
                                    ..Default::default()
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
                            .try_deposit_batch_or_abort(config.storing_account.address)
                            .done()
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    let new_resources = result.new_resource_addresses();
                    state.pool_1.resource_1.set(XRD);
                    state.pool_1.resource_2.set(new_resources[0]);
                    state.pool_2.resource_1.set(new_resources[1]);
                    state.pool_2.resource_2.set(new_resources[2]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "radiswap-create-owner-badge-and-dapp-definition-account",
                        |builder| {
                            let definition_account = config.radiswap_dapp_definition_account.address;
                            builder
                                .create_non_fungible_resource(
                                    // TODO: Once we can use address reservation with resource creation,
                                    // we can set the owner badge to be its own owner
                                    OwnerRole::None,
                                    NonFungibleIdType::Integer,
                                    true,
                                    NonFungibleResourceRoles::default(),
                                    metadata! {
                                        init {
                                            "name" => "Radiswap dApp Owner Badge", updatable;
                                            "description" => "The owner badge for the Radiswap dApp", updatable;
                                            "tags" => vec!["badge", "dex", "pool", "radiswap"], updatable;
                                            "info_url" => Url::of("https://www.radixdlt.com/"), updatable;
                                        }
                                    },
                                    Some([
                                        (NonFungibleLocalId::integer(1), ())
                                    ]),
                                )
                                .try_deposit_batch_or_abort(definition_account)
                                .set_metadata(
                                    definition_account,
                                    "account_type",
                                    "dapp definition"
                                )
                                .set_metadata(
                                    definition_account,
                                    "name",
                                    "Radiswap dApp Definition"
                                )
                                .set_metadata(
                                    definition_account,
                                    "claimed_websites",
                                    vec![Origin::of("https://radiswap.radixdlt.com/")]
                                )
                        },
                        vec![]
                    )
                },
                |core, config, state, result| {
                    let new_resources = result.new_resource_addresses();
                    state.owner_badge.set(NonFungibleGlobalId::new(
                        new_resources[0],
                        NonFungibleLocalId::integer(1)
                    ));

                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    let code = include_bytes!("../../../assets/radiswap.wasm");
                    let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                        "../../../assets/radiswap.rpd"
                    ))
                    .unwrap();
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-publish-and-create-pools",
                        |builder| {
                            let lookup = builder.name_lookup();
                            builder.allocate_global_address(
                                PACKAGE_PACKAGE,
                                PACKAGE_BLUEPRINT,
                                "radiswap_package_reservation",
                                "radiswap_package"
                            )
                            .get_free_xrd_from_faucet()
                            .publish_package_advanced(
                                Some("radiswap_package_reservation".to_string()),
                                code.to_vec(),
                                schema,
                                metadata_init! {
                                    "name" => "Radiswap Package".to_owned(), locked;
                                    "description" => "A package of the logic of a Uniswap v2 style DEX.".to_owned(), locked;
                                    "tags" => vec!["dex".to_owned(), "pool".to_owned(), "radiswap".to_owned()], locked;
                                },
                                radix_engine::types::OwnerRole::Fixed(rule!(require(
                                    state.owner_badge.get()?
                                ))),
                            ).call_function(
                                lookup.named_address("radiswap_package"),
                                "Radiswap", 
                                "new", 
                                manifest_args!(state.pool_1.resource_1.unwrap(), state.pool_1.resource_2.unwrap())
                            )
                            .call_function(
                                lookup.named_address("radiswap_package"),
                                "Radiswap", 
                                "new", 
                                manifest_args!(state.pool_2.resource_1.unwrap(), state.pool_2.resource_2.unwrap())
                            )
                            .try_deposit_batch_or_abort(config.storing_account.address)
                            .done()
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    let new_packages = result.new_package_addresses();
                    state.radiswap_package.set(new_packages[0]);

                    let new_components = result.new_component_addresses();
                    state.pool_1.radiswap.set(new_components[0]);
                    state.pool_1.pool.set(new_components[1]);
                    state.pool_2.radiswap.set(new_components[2]);
                    state.pool_2.pool.set(new_components[3]);

                    let new_resources = result.new_resource_addresses();
                    state.pool_1.pool_unit.set(new_resources[0]);
                    state.pool_2.pool_unit.set(new_resources[1]);

                    Ok(())
                },
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-add-liquidity",
                        |builder| {
                            builder
                                .get_free_xrd_from_faucet()
                                .withdraw_from_account(
                                    config.storing_account.address,
                                    state.pool_1.resource_2.get()?,
                                    7000,
                                )
                                .withdraw_from_account(
                                    config.storing_account.address,
                                    state.pool_2.resource_1.get()?,
                                    5000,
                                )
                                .withdraw_from_account(
                                    config.storing_account.address,
                                    state.pool_2.resource_2.get()?,
                                    8000,
                                )
                                .take_all_from_worktop(
                                    state.pool_1.resource_1.get()?,
                                    "pool_1_resource_1"
                                )
                                .take_all_from_worktop(
                                    state.pool_1.resource_2.get()?,
                                    "pool_1_resource_2"
                                )
                                .call_method_with_name_lookup(
                                    state.pool_1.radiswap.get()?,
                                    "add_liquidity",
                                    |lookup| (
                                        lookup.bucket("pool_1_resource_1"),
                                        lookup.bucket("pool_1_resource_2"),
                                    ),
                                )
                                .take_all_from_worktop(
                                    state.pool_2.resource_1.get()?,
                                    "pool_2_resource_1",
                                )
                                .take_all_from_worktop(
                                    state.pool_2.resource_2.get()?,
                                    "pool_2_resource_2",
                                )
                                .call_method_with_name_lookup(
                                    state.pool_2.radiswap.get()?,
                                    "add_liquidity",
                                    |lookup| (
                                        lookup.bucket("pool_2_resource_1"),
                                        lookup.bucket("pool_2_resource_2"),
                                    ),
                                )
                                .try_deposit_batch_or_abort(config.storing_account.address)
                                .done()
                        },
                        vec![&config.storing_account.key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-distribute-tokens",
                        |mut builder| {
                            builder = builder.get_free_xrd_from_faucet();
                            for destination_account in [&config.user_account_1, &config.user_account_2, &config.user_account_3]
                            {
                                for resource_address in [
                                    state.pool_1.resource_1.get()?,
                                    state.pool_1.resource_2.get()?,
                                    state.pool_2.resource_1.get()?,
                                    state.pool_2.resource_2.get()?,
                                    state.pool_1.pool_unit.get()?,
                                    state.pool_2.pool_unit.get()?,
                                ] {
                                    builder = builder.withdraw_from_account(
                                        config.storing_account.address,
                                        resource_address,
                                        333,
                                    );
                                }
                                builder = builder.try_deposit_batch_or_abort(destination_account.address);
                            }
                            builder.done()
                        },
                        vec![&config.storing_account.key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-swap-tokens",
                        |builder| {
                            builder
                                .withdraw_from_account(
                                    config.user_account_1.address,
                                    state.pool_1.resource_1.get()?,
                                    100,
                                )
                                .take_all_from_worktop(
                                    state.pool_1.resource_1.get()?,
                                    "input",
                                ).call_method_with_name_lookup(
                                    state.pool_1.radiswap.unwrap(),
                                    "swap",
                                    |lookup| (
                                        lookup.bucket("input"),
                                    )
                                )
                                .try_deposit_batch_or_abort(config.user_account_1.address)
                                .done()
                        },
                        vec![&config.user_account_1.key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-remove-tokens",
                        |builder| {
                            builder
                                .withdraw_from_account(
                                    config.user_account_1.address,
                                    state.pool_1.pool_unit.get()?,
                                    100,
                                )
                                .take_all_from_worktop(
                                    state.pool_1.pool_unit.get()?,
                                    "pool_units",
                                )
                                .then(|builder| {
                                    let bucket = builder.bucket("pool_units");
                                    builder.call_method(
                                        state.pool_1.radiswap.unwrap(),
                                        "remove_liquidity",
                                        manifest_args!(bucket),
                                    )
                                })
                                .try_deposit_batch_or_abort(config.user_account_1.address)
                                .done()
                        },
                        vec![&config.user_account_1.key],
                    )
                }
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("radiswap_owner", &config.radiswap_dapp_definition_account)
                        .add("storing_account", &config.storing_account)
                        .add("user_account_1", &config.user_account_1)
                        .add("user_account_2", &config.user_account_2)
                        .add("user_account_3", &config.user_account_3)
                        .add("radiswap_package", state.radiswap_package.get()?)
                        .add("pool_1_radiswap", state.pool_1.radiswap.get()?)
                        .add("pool_1_pool", state.pool_1.pool.get()?)
                        .add("pool_1_resource_1", state.pool_1.resource_1.get()?)
                        .add("pool_1_resource_2", state.pool_1.resource_2.get()?)
                        .add("pool_1_pool_unit", state.pool_1.pool_unit.get()?)
                        .add("pool_2_radiswap", state.pool_2.radiswap.get()?)
                        .add("pool_2_pool", state.pool_2.pool.get()?)
                        .add("pool_2_resource_1", state.pool_2.resource_1.get()?)
                        .add("pool_2_resource_2", state.pool_2.resource_2.get()?)
                        .add("pool_2_pool_unit", state.pool_2.pool_unit.get()?),
                })
            })
    }
}
