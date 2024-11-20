use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

use crate::internal_prelude::*;

#[allow(deprecated)]
pub struct RadiswapScenarioConfig {
    pub radiswap_dapp_definition_account: PreallocatedAccount,
    pub storing_account: PreallocatedAccount,
    pub user_account_1: PreallocatedAccount,
    pub user_account_2: PreallocatedAccount,
    pub user_account_3: PreallocatedAccount,
}

impl Default for RadiswapScenarioConfig {
    fn default() -> Self {
        Self {
            radiswap_dapp_definition_account: ed25519_account_from_u64(891231),
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
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "radiswap",
        protocol_min_requirement: ProtocolVersion::Babylon,
        protocol_max_requirement: ProtocolVersion::LATEST,
        testnet_run_at: Some(ProtocolVersion::Babylon),
        safe_to_run_on_used_ledger: false,
    };

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance {
        #[allow(unused_variables, deprecated)]
        ScenarioBuilder::new(core, Self::METADATA, config, start_state)
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
                            .try_deposit_entire_worktop_or_abort(config.storing_account.address, None)
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
                                            "name" => "Radiswap - dApp Owner Badge", updatable;
                                            "description" => "[EXAMPLE] The owner badge for the Radiswap dApp and associated entities", updatable;
                                            "tags" => ["badge", "dex", "pool", "radiswap"], updatable;
                                            "info_url" => UncheckedUrl::of("https://radiswap.radixdlt.com/"), updatable;
                                        }
                                    },
                                    Some([
                                        (NonFungibleLocalId::integer(1), ())
                                    ]),
                                )
                                .try_deposit_entire_worktop_or_abort(definition_account, None)
                                .set_metadata(definition_account, "account_type", "dapp definition")
                                .set_metadata(definition_account, "name", "Radiswap dApp Definition")
                                .set_metadata(definition_account, "description", "[EXAMPLE] The Radiswap dApp definition account")
                                .set_metadata(definition_account, "tags", ["dex", "pool", "radiswap"])
                                .set_metadata(definition_account, "info_url", UncheckedUrl::of("https://radiswap.radixdlt.com/"))
                                .set_metadata(
                                    definition_account,
                                    "claimed_websites",
                                    [UncheckedOrigin::of("https://radiswap.radixdlt.com")]
                                )
                        },
                        vec![&config.radiswap_dapp_definition_account.key]
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
                    let code = include_bytes!("../../assets/radiswap.wasm");
                    let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                        "../../assets/radiswap.rpd"
                    ))
                    .unwrap();
                    let owner_role = radix_engine_interface::prelude::OwnerRole::Fixed(rule!(require(
                        state.owner_badge.get()?
                    )));
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
                                "radiswap_package_reservation",
                                code.to_vec(),
                                schema,
                                metadata_init! {
                                    "name" => "Radiswap Package", locked;
                                    "description" => "[EXAMPLE] A package of the logic of a Uniswap v2 style DEX.".to_owned(), locked;
                                    "tags" => ["dex", "pool", "radiswap"], locked;
                                },
                                owner_role.clone(),
                            ).call_function(
                                lookup.named_address("radiswap_package"),
                                "Radiswap",
                                "new",
                                manifest_args!(
                                    owner_role.clone(),
                                    state.pool_1.resource_1.get()?,
                                    state.pool_1.resource_2.get()?,
                                )
                            )
                            .call_function(
                                lookup.named_address("radiswap_package"),
                                "Radiswap",
                                "new",
                                manifest_args!(
                                    owner_role.clone(),
                                    state.pool_2.resource_1.get()?,
                                    state.pool_2.resource_2.get()?,
                                )
                            )
                            .try_deposit_entire_worktop_or_abort(config.radiswap_dapp_definition_account.address, None)
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
                                .try_deposit_entire_worktop_or_abort(config.storing_account.address, None)
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
                            builder = builder.get_free_xrd_from_faucet()
                                .try_deposit_entire_worktop_or_abort(config.storing_account.address, None);
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
                                builder = builder.try_deposit_entire_worktop_or_abort(destination_account.address, None);
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
                                .try_deposit_entire_worktop_or_abort(config.user_account_1.address, None)
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
                                .try_deposit_entire_worktop_or_abort(config.user_account_1.address, None)
                                .done()
                        },
                        vec![&config.user_account_1.key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    let definition = GlobalAddress::from(config.radiswap_dapp_definition_account.address);
                    let radiswap_1 = GlobalAddress::from(state.pool_1.radiswap.get()?);
                    let pool_1 = GlobalAddress::from(state.pool_1.pool.get()?);
                    let pool_unit_1 = GlobalAddress::from(state.pool_1.pool_unit.get()?);
                    let radiswap_2 = GlobalAddress::from(state.pool_2.radiswap.get()?);
                    let pool_2 = GlobalAddress::from(state.pool_2.pool.get()?);
                    let pool_unit_2 = GlobalAddress::from(state.pool_2.pool_unit.get()?);
                    fn add_metadata(
                        builder: ManifestBuilder,
                        address: GlobalAddress,
                        name: &'static str,
                        description: &'static str,
                    ) -> ManifestBuilder {
                        builder
                            .set_metadata(address, "name", name)
                            .set_metadata(address, "description", description)
                            .set_metadata(address, "tags", ["badge", "dex", "pool", "radiswap"])
                            .set_metadata(address, "info_url", UncheckedUrl::of("https://radiswap.radixdlt.com/"))
                    }
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-set-two-way-linking",
                        |builder| {
                            builder
                                .create_proof_from_account_of_non_fungible(
                                    config.radiswap_dapp_definition_account.address,
                                    state.owner_badge.get()?
                                )
                                // Set up two-way-linking
                                .set_metadata(
                                    definition,
                                    "claimed_entities",
                                    vec![
                                        radiswap_1,
                                        pool_1,
                                        pool_unit_1,
                                        radiswap_2,
                                        pool_2,
                                        pool_unit_2,
                                    ]
                                )
                                // Note - Components use "dapp_definition" but Resources use "dapp_definitions"
                                .set_metadata(radiswap_1, "dapp_definition", definition)
                                .set_metadata(radiswap_2, "dapp_definition", definition)
                                .set_metadata(pool_1, "dapp_definition", definition)
                                .set_metadata(pool_2, "dapp_definition", definition)
                                .set_metadata(pool_unit_1, "dapp_definitions", [definition])
                                .set_metadata(pool_unit_2, "dapp_definitions", [definition])
                                // Set up other metadata which has been missed
                                .then(|builder| add_metadata(
                                    builder,
                                    radiswap_1,
                                    "Radiswap 1 - XRD/BTC: Component",
                                    "[EXAMPLE] A Radiswap component between test tokens \"XRD\" and \"BTC\"",
                                ))
                                .then(|builder| add_metadata(
                                    builder,
                                    pool_1,
                                    "Radiswap 1 - XRD/BTC: Pool",
                                    "[EXAMPLE] The underyling pool between test tokens \"XRD\" and \"BTC\"",
                                ))
                                .then(|builder| add_metadata(
                                    builder,
                                    pool_unit_1,
                                    "Radiswap 1 - XRD/BTC: Pool Units",
                                    "[EXAMPLE] The pool units resource for the underlying pool between test tokens \"XRD\" and \"BTC\"",
                                ))
                                .then(|builder| add_metadata(
                                    builder,
                                    radiswap_2,
                                    "Radiswap 2 - ETH/ETC: Component",
                                    "[EXAMPLE] A Radiswap dApp between test tokens \"ETH\" and \"ETC\"",
                                ))
                                .then(|builder| add_metadata(
                                    builder,
                                    pool_2,
                                    "Radiswap 2 - ETH/ETC: Pool",
                                    "[EXAMPLE] The underyling pool between test tokens \"ETH\" and \"ETC\"",
                                ))
                                .then(|builder| add_metadata(
                                    builder,
                                    pool_unit_2,
                                    "Radiswap 2 - ETH/ETC: Pool Units",
                                    "[EXAMPLE] The pool units resource for the underlying pool between test tokens \"ETH\" and \"ETC\"",
                                ))
                                .done()
                        },
                        vec![&config.radiswap_dapp_definition_account.key],
                    )
                }
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("radiswap_dapp_definition_account", &config.radiswap_dapp_definition_account)
                        .add("radiswap_dapp_owner_badge", state.owner_badge.get()?)
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
