use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

use crate::internal_prelude::*;

pub struct MayaRouterScenarioConfig {
    pub owner_account: VirtualAccount,
    pub signer_account: VirtualAccount,
    pub swapper_account: VirtualAccount,
}

impl Default for MayaRouterScenarioConfig {
    fn default() -> Self {
        Self {
            owner_account: ed25519_account_for_private_key(891231),
            signer_account: secp256k1_account_2(),
            swapper_account: secp256k1_account_3(),
        }
    }
}

#[derive(Default)]
pub struct MayaRouterScenarioState {
    owner_badge: State<NonFungibleGlobalId>,
    signer_badge: State<NonFungibleGlobalId>,
    swapper_badge: State<NonFungibleGlobalId>,

    maya_router_package: State<PackageAddress>,
    maya_router_data: MayaRouterData,
}

#[derive(Default)]
pub struct MayaRouterData {
    maya_router_address: State<ComponentAddress>,
    resources: State<IndexSet<ResourceAddress>>,
}

pub struct MayaRouterScenarioCreator;

impl ScenarioCreator for MayaRouterScenarioCreator {
    type Config = MayaRouterScenarioConfig;
    type State = MayaRouterScenarioState;

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "maya_router",
        };

        #[allow(unused_variables)]
        ScenarioBuilder::new(core, metadata, config, start_state)
            .successful_transaction(|core, config, state| {
                core.next_transaction_free_xrd_from_faucet(config.swapper_account.address)
            })
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-initialize-data",
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
                                        "name" => "EARLY".to_owned(), locked;
                                        "symbol" => "EARLY".to_owned(), locked;
                                        "description" => "Today, you’re still early…but not for long.".to_owned(), locked;
                                        "tags" => vec!["memecoin".to_owned()], locked;
                                        "icon_url" => "https://arweave.net/uXCQ9YVGkEijn7PS2wdkXqwkU_YrdgpNtQPH2Y1-Qcs".to_owned(), locked;
                                        "info_url" => "https://twitter.com/earlyxrd".to_owned(), locked;
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
                                        "name" => "Hug".to_owned(), locked;
                                        "symbol" => "HUG".to_owned(), locked;
                                        "description" => "give hugs".to_owned(), locked;
                                        "tags" => vec!["memecoin".to_owned()], locked;
                                        "icon_url" => "https://i.imgur.com/TjciHNV.png".to_owned(), locked;
                                        "info_url" => "https://hug.meme".to_owned(), locked;
                                    }
                                },
                                Some(100_000_000_000u64.into()),
                            )
                            .try_deposit_entire_worktop_or_abort(config.swapper_account.address, None)
                            .done()
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.maya_router_data.resources.set(result.new_resource_addresses().clone());
                    state.owner_badge.set(NonFungibleGlobalId::from_public_key(&config.owner_account.public_key));
                    state.signer_badge.set(NonFungibleGlobalId::from_public_key(&config.signer_account.public_key));
                    state.swapper_badge.set(NonFungibleGlobalId::from_public_key(&config.swapper_account.public_key));
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    let code = include_bytes!("../../assets/maya_router.wasm");
                    let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                        "../../assets/maya_router.rpd"
                    ))
                    .unwrap();
                    let owner_badge = state.owner_badge.get()?;
                    let owner_role = OwnerRole::Fixed(rule!(require(owner_badge.clone())));

                    let signer_badge = state.signer_badge.get()?;

                    let signer_rule = rule!(require(signer_badge));

                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-publish-and-instantiate",
                        |builder| {
                            let lookup = builder.name_lookup();
                            builder.allocate_global_address(
                                PACKAGE_PACKAGE,
                                PACKAGE_BLUEPRINT,
                                "maya_router_package_reservation",
                                "maya_router_package"
                            )
                            .publish_package_advanced(
                                Some("maya_router_package_reservation".to_string()),
                                code.to_vec(),
                                schema,
                                metadata_init! {
                                    "name" => "MayaRouter Package", locked;
                                    "description" => "MayaRouter package stores assets swappable with assets from other networks".to_owned(), locked;
                                    "tags" => ["bridge", "cross-chain"], locked;
                                },
                                owner_role,
                            )
                            .call_function(
                                lookup.named_address("maya_router_package"),
                                "MayaRouter",
                                "instantiate",
                                manifest_args!(
                                    owner_badge,
                                    signer_rule
                                )
                            )
                            .done()
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    let new_packages = result.new_package_addresses();
                    state.maya_router_package.set(new_packages[0]);
                    let new_components = result.new_component_addresses();
                    state.maya_router_data.maya_router_address.set(new_components[0]);
                    Ok(())
                },
            )
            /*
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-add-liquidity",
                        |builder| {
                            builder
                                .get_free_xrd_from_faucet()
                                .withdraw_from_account(
                                    config.signer_account.address,
                                    state.maya_router_data.resource_2.get()?,
                                    7000,
                                )
                                .withdraw_from_account(
                                    config.signer_account.address,
                                    state.pool_2.resource_1.get()?,
                                    5000,
                                )
                                .withdraw_from_account(
                                    config.signer_account.address,
                                    state.pool_2.resource_2.get()?,
                                    8000,
                                )
                                .take_all_from_worktop(
                                    state.maya_router_data.resource_1.get()?,
                                    "pool_1_resource_1"
                                )
                                .take_all_from_worktop(
                                    state.maya_router_data.resource_2.get()?,
                                    "pool_1_resource_2"
                                )
                                .call_method_with_name_lookup(
                                    state.maya_router_data.maya_router_address.get()?,
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
                                    state.pool_2.maya_router_address.get()?,
                                    "add_liquidity",
                                    |lookup| (
                                        lookup.bucket("pool_2_resource_1"),
                                        lookup.bucket("pool_2_resource_2"),
                                    ),
                                )
                                .try_deposit_entire_worktop_or_abort(config.signer_account.address, None)
                                .done()
                        },
                        vec![&config.signer_account.key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "radiswap-distribute-tokens",
                        |mut builder| {
                            builder = builder.get_free_xrd_from_faucet()
                                .try_deposit_entire_worktop_or_abort(config.signer_account.address, None);
                            for destination_account in [&config.swapper_account, &config.user_account_2, &config.user_account_3]
                            {
                                for resource_address in [
                                    state.maya_router_data.resource_1.get()?,
                                    state.maya_router_data.resource_2.get()?,
                                    state.pool_2.resource_1.get()?,
                                    state.pool_2.resource_2.get()?,
                                    state.maya_router_data.pool_unit.get()?,
                                    state.pool_2.pool_unit.get()?,
                                ] {
                                    builder = builder.withdraw_from_account(
                                        config.signer_account.address,
                                        resource_address,
                                        333,
                                    );
                                }
                                builder = builder.try_deposit_entire_worktop_or_abort(destination_account.address, None);
                            }
                            builder.done()
                        },
                        vec![&config.signer_account.key],
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
                                    config.swapper_account.address,
                                    state.maya_router_data.resource_1.get()?,
                                    100,
                                )
                                .take_all_from_worktop(
                                    state.maya_router_data.resource_1.get()?,
                                    "input",
                                ).call_method_with_name_lookup(
                                    state.maya_router_data.maya_router_address.unwrap(),
                                    "swap",
                                    |lookup| (
                                        lookup.bucket("input"),
                                    )
                                )
                                .try_deposit_entire_worktop_or_abort(config.swapper_account.address, None)
                                .done()
                        },
                        vec![&config.swapper_account.key],
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
                                    config.swapper_account.address,
                                    state.maya_router_data.pool_unit.get()?,
                                    100,
                                )
                                .take_all_from_worktop(
                                    state.maya_router_data.pool_unit.get()?,
                                    "pool_units",
                                )
                                .then(|builder| {
                                    let bucket = builder.bucket("pool_units");
                                    builder.call_method(
                                        state.maya_router_data.maya_router_address.unwrap(),
                                        "remove_liquidity",
                                        manifest_args!(bucket),
                                    )
                                })
                                .try_deposit_entire_worktop_or_abort(config.swapper_account.address, None)
                                .done()
                        },
                        vec![&config.swapper_account.key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    let definition = GlobalAddress::from(config.owner_account.address);
                    let radiswap_1 = GlobalAddress::from(state.maya_router_data.maya_router_address.get()?);
                    let pool_1 = GlobalAddress::from(state.maya_router_data.pool.get()?);
                    let pool_unit_1 = GlobalAddress::from(state.maya_router_data.pool_unit.get()?);
                    let radiswap_2 = GlobalAddress::from(state.pool_2.maya_router_address.get()?);
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
                                    config.owner_account.address,
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
                        vec![&config.owner_account.key],
                    )
                }
            )

            */
            .finalize(|core, config, state| {
                let mut interesting_addresses = DescribedAddresses::new()
                        .add("owner_account", &config.owner_account)
                        .add("owner_badge", state.owner_badge.get()?)
                        .add("signer_account", &config.signer_account)
                        .add("signer_badge", state.signer_badge.get()?)
                        .add("swapper_account", &config.swapper_account)
                        .add("swapper_badge", state.swapper_badge.get()?)
                        .add("maya_router_package", state.maya_router_package.get()?)
                        .add("maya_router_data", state.maya_router_data.maya_router_address.get()?);

                for (idx, resource) in state.maya_router_data.resources.get()?.iter().enumerate() {
                    interesting_addresses = interesting_addresses.add(format!("resource_{:?}", idx), resource.clone());
                }

                Ok(ScenarioOutput {
                    interesting_addresses
                })
            })
    }
}
