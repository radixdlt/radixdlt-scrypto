use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

use crate::internal_prelude::*;

pub struct MayaRouterScenarioConfig {
    pub owner_account: VirtualAccount,
    pub admin_1_account: VirtualAccount,
    pub admin_2_account: VirtualAccount,
    pub swapper_account: VirtualAccount,
}

impl Default for MayaRouterScenarioConfig {
    fn default() -> Self {
        Self {
            owner_account: secp256k1_account_2(),
            admin_1_account: secp256k1_account_1(),
            admin_2_account: ed25519_account_1(),
            swapper_account: ed25519_account_2(),
        }
    }
}

#[derive(Default)]
pub struct MayaRouterScenarioState {
    owner_badge: State<NonFungibleGlobalId>,
    admin_1_badge: State<NonFungibleGlobalId>,
    admin_2_badge: State<NonFungibleGlobalId>,
    swapper_badge: State<NonFungibleGlobalId>,

    maya_router_package: State<PackageAddress>,
    maya_router_data: MayaRouterData,
}

#[derive(Default)]
pub struct MayaRouterData {
    maya_router_address: State<ComponentAddress>,
    resource_1: State<ResourceAddress>,
    resource_2: State<ResourceAddress>,
    resource_3: State<ResourceAddress>,
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
                    state.maya_router_data.resource_1.set(XRD);
                    state.maya_router_data.resource_2.set(result.new_resource_addresses()[0]);
                    state.maya_router_data.resource_3.set(result.new_resource_addresses()[1]);

                    state.owner_badge.set(NonFungibleGlobalId::from_public_key(&config.owner_account.public_key));
                    state.admin_1_badge.set(NonFungibleGlobalId::from_public_key(&config.admin_1_account.public_key));
                    state.admin_2_badge.set(NonFungibleGlobalId::from_public_key(&config.admin_2_account.public_key));
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

                    let admin_badge = state.admin_1_badge.get()?;

                    let admin_rule = rule!(require(admin_badge));

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
                                    admin_rule,
                                    config.admin_1_account.address,
                                ),
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
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-deposit",
                        |builder| {
                            let resource_1 = state.maya_router_data.resource_1.get()?;
                            let resource_2 = state.maya_router_data.resource_2.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .withdraw_from_account(config.swapper_account.address, resource_1, dec!(100))
                                .withdraw_from_account(config.swapper_account.address, resource_2, dec!(200))
                                .take_all_from_worktop(resource_1, "resource_1")
                                .with_bucket("resource_1", |builder, bucket| {
                                    builder.call_method(
                                        router_address,
                                        "deposit",
                                        manifest_args!(
                                            config.swapper_account.address,
                                            bucket,
                                            "SWAP:MAYA.CACAO".to_string(),
                                        ),
                                    )
                                })
                                .take_all_from_worktop(resource_2, "resource_2")
                                .with_bucket("resource_2", |builder, bucket| {
                                    builder.call_method(
                                        router_address,
                                        "deposit",
                                        manifest_args!(
                                            config.swapper_account.address,
                                            bucket,
                                            "SWAP:MAYA.CACAO".to_string(),
                                        ),
                                    )
                                })
                                .done()
                        },
                        vec![&config.swapper_account.key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out",
                        |builder| {
                            let resource_1 = state.maya_router_data.resource_1.get()?;
                            let resource_2 = state.maya_router_data.resource_2.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.admin_1_account.address,
                                        config.swapper_account.address,
                                        resource_1,
                                        dec!(10),
                                        "OUT:".to_string(),
                                    ),
                                )
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.admin_1_account.address,
                                        config.swapper_account.address,
                                        resource_2,
                                        dec!(20),
                                        "OUT:".to_string(),
                                    ),
                                )
                                .done()
                        },
                        vec![&config.admin_1_account.key],
                    )
                }
            )
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-failed-asset-not-available",
                        |builder| {
                            // resource_3 is not available in the deposited resources in MayaRouter
                            let resource_3 = state.maya_router_data.resource_3.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.admin_1_account.address,
                                        config.swapper_account.address,
                                        resource_3,
                                        dec!(30),
                                        "OUT:".to_string()
                                    ),
                                )
                                .done()
                        },
                        vec![&config.admin_1_account.key],
                    )
                },
                |_, _, _, _| Ok(()),
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-update-admin",
                        |builder| {

                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            let admin_2_rule = rule!(require(state.admin_2_badge.get()?));

                            builder
                                .call_method(
                                    router_address,
                                    "update_admin",
                                    manifest_args!(
                                        admin_2_rule,
                                        config.admin_2_account.address,
                                    ),
                                )
                                .done()
                        },
                        vec![&config.admin_1_account.key],
                    )
                }
            )
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-failed-auth-error",
                        |builder| {
                            let resource_2 = state.maya_router_data.resource_1.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.admin_1_account.address,
                                        config.swapper_account.address,
                                        resource_2,
                                        dec!(20),
                                        "OUT:".to_string(),
                                    ),
                                )
                                .done()
                        },
                        // Transaction should fail, because admin_1 badge is used
                        vec![&config.admin_1_account.key],
                    )
                },
                |_, _, _, _| Ok(()),
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-admin-2",
                        |builder| {
                            let resource_2 = state.maya_router_data.resource_1.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.admin_2_account.address,
                                        config.swapper_account.address,
                                        resource_2,
                                        dec!(20),
                                        "OUT:".to_string(),
                                    ),
                                )
                                .done()
                        },
                        //
                        vec![&config.admin_2_account.key],
                    )
                },
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("owner_account", &config.owner_account)
                        .add("owner_badge", state.owner_badge.get()?)
                        .add("admin_account", &config.admin_1_account)
                        .add("admin_badge", state.admin_1_badge.get()?)
                        .add("swapper_account", &config.swapper_account)
                        .add("swapper_badge", state.swapper_badge.get()?)
                        .add("maya_router_package", state.maya_router_package.get()?)
                        .add("maya_router_address", state.maya_router_data.maya_router_address.get()?)
                        .add("resource_1", state.maya_router_data.resource_1.get()?)
                        .add("resource_2", state.maya_router_data.resource_2.get()?)
                        .add("resource_3", state.maya_router_data.resource_3.get()?)
                })
            })
    }
}
