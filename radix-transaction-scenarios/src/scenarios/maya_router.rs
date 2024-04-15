use crate::internal_prelude::*;
use crate::utils::{new_ed25519_private_key, new_secp256k1_private_key};
use radix_engine::updates::{ProtocolUpdate, ProtocolVersion};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

pub struct MayaRouterScenarioConfig {
    pub owner_private_key: PrivateKey,
    pub swapper_private_key: PrivateKey,
    pub asgard_vault_1_private_key: PrivateKey,
    pub asgard_vault_2_private_key: PrivateKey,
    pub asgard_vault_1_public_key: Ed25519PublicKey,
    pub asgard_vault_2_public_key: Ed25519PublicKey,
}

impl Default for MayaRouterScenarioConfig {
    fn default() -> Self {
        let key_1 = new_ed25519_private_key(1);
        let key_2 = new_ed25519_private_key(2);
        let pub_key_1 = key_1.public_key();
        let pub_key_2 = key_2.public_key();
        Self {
            owner_private_key: new_ed25519_private_key(3).into(),
            swapper_private_key: new_secp256k1_private_key(1).into(),
            asgard_vault_1_private_key: key_1.into(),
            asgard_vault_2_private_key: key_2.into(),
            asgard_vault_1_public_key: pub_key_1,
            asgard_vault_2_public_key: pub_key_2,
        }
    }
}

#[derive(Default)]
pub struct MayaRouterScenarioState {
    owner_account: State<ComponentAddress>,
    swapper_account: State<ComponentAddress>,

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
    type Instance = Scenario<Self::Config, Self::State>;
    const SCENARIO_PROTOCOL_REQUIREMENT: ProtocolVersion =
        ProtocolVersion::ProtocolUpdate(ProtocolUpdate::Bottlenose);

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance {
        let metadata = ScenarioMetadata {
            logical_name: "maya_router",
        };

        #[allow(unused_variables)]
        ScenarioBuilder::new(core, metadata, config, start_state)
            .successful_transaction_with_result_handler(
                |core, config, _| {
                    core.next_transaction_with_faucet_lock_fee(
                        "maya-router-create-accounts",
                        |builder| {
                            [
                                &config.owner_private_key,
                                &config.swapper_private_key,
                            ]
                            .iter()
                            .fold(builder, |builder, key| {
                                builder.call_function(
                                    ACCOUNT_PACKAGE,
                                    ACCOUNT_BLUEPRINT,
                                    ACCOUNT_CREATE_ADVANCED_IDENT,
                                    AccountCreateAdvancedManifestInput {
                                        address_reservation: None,
                                        owner_role: OwnerRole::Fixed(rule!(require(
                                            NonFungibleGlobalId::from_public_key(&key.public_key())
                                        ))),
                                    },
                                )
                            })
                        },
                        vec![],
                    )
                },
                |_, _, state, result| {
                    state
                        .owner_account
                        .set(result.new_component_addresses()[0]);
                    state
                        .swapper_account
                        .set(result.new_component_addresses()[1]);
                    Ok(())
                },
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_free_xrd_from_faucet(state.swapper_account.get()?)
            })
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-create-resources",
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
                                        "name" => "Fizz".to_owned(), locked;
                                        "symbol" => "FIZZ".to_owned(), locked;
                                        "description" => "Fizz token".to_owned(), locked;
                                        "tags" => vec!["test".to_owned()], locked;
                                        "icon_url" => "https://example.com/icon.png".to_owned(), locked;
                                        "info_url" => "https://example.com".to_owned(), locked;
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
                                        "name" => "Buzz".to_owned(), locked;
                                        "symbol" => "BUZZ".to_owned(), locked;
                                        "description" => "Buzz".to_owned(), locked;
                                        "tags" => vec!["test".to_owned()], locked;
                                        "icon_url" => "https://example.com/icon.png".to_owned(), locked;
                                        "info_url" => "https://example.com".to_owned(), locked;
                                    }
                                },
                                Some(100_000_000_000u64.into()),
                            )
                            .try_deposit_entire_worktop_or_abort(state.swapper_account.get()?, None)
                            .done()
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.maya_router_data.resource_1.set(XRD);
                    state.maya_router_data.resource_2.set(result.new_resource_addresses()[0]);
                    state.maya_router_data.resource_3.set(result.new_resource_addresses()[1]);
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
                                OwnerRole::None,
                            )
                            .call_function(
                                lookup.named_address("maya_router_package"),
                                "MayaRouter",
                                "instantiate",
                                manifest_args!(),
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
                        "maya-router-deposit-to-asgard-vault-1",
                        |builder| {
                            let resource_2 = state.maya_router_data.resource_2.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            let swapper_account = state.swapper_account.get()?;

                            builder
                                .withdraw_from_account(state.swapper_account.get()?, XRD, dec!(100))
                                .withdraw_from_account(state.swapper_account.get()?, resource_2, dec!(200))
                                .take_all_from_worktop(XRD, "xrds")
                                .with_bucket("xrds", |builder, bucket| {
                                    builder.call_method(
                                        router_address,
                                        "deposit",
                                        manifest_args!(
                                            swapper_account,
                                            config.asgard_vault_1_public_key,
                                            bucket,
                                            "=:MAYA.CACAO:maya12ehykd8m4a79av36x0m9wzvq3uf06x5xa2yzd2::wr:100".to_string(),
                                        ),
                                    )
                                })
                                .take_all_from_worktop(resource_2, "resource_2")
                                .with_bucket("resource_2", |builder, bucket| {
                                    builder.call_method(
                                        router_address,
                                        "deposit",
                                        manifest_args!(
                                            swapper_account,
                                            config.asgard_vault_1_public_key,
                                            bucket,
                                            "=:MAYA.CACAO:maya12ehykd8m4a79av36x0m9wzvq3uf06x5xa2yzd2::wr:100".to_string()
                                        ),
                                    )
                                })
                                .done()
                        },
                        vec![&config.swapper_private_key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    let swapper_account = state.swapper_account.get()?;
                    let swapper_account_str = swapper_account.to_string(AddressDisplayContext::with_encoder(&core.encoder()));
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-from-asgard-vault-1",
                        |builder| {
                            let resource_2 = state.maya_router_data.resource_2.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.asgard_vault_1_public_key,
                                        swapper_account,
                                        XRD,
                                        dec!(10),
                                        format!("=:XRD.XRD:{}::wr:100", swapper_account_str),
                                    ),
                                )
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.asgard_vault_1_public_key,
                                        swapper_account,
                                        resource_2,
                                        dec!(20),
                                        format!("=:XRD.FIZZ:{}::wr:100", swapper_account_str),
                                    ),
                                )
                                .done()
                        },
                        vec![&config.asgard_vault_1_private_key],
                    )
                }
            )
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    let swapper_account = state.swapper_account.get()?;
                    let swapper_account_str = swapper_account.to_string(AddressDisplayContext::with_encoder(&core.encoder()));
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-from-asgard-vault-1-resource-3-failed-asset-not-available",
                        |builder| {
                            // resource_3 is not available in the deposited resources in MayaRouter
                            let resource_3 = state.maya_router_data.resource_3.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.asgard_vault_1_public_key,
                                        swapper_account,
                                        resource_3,
                                        dec!(30),
                                        format!("=:XRD.BUZZ:{}::wr:100", swapper_account_str),
                                    ),
                                )
                                .done()
                        },
                        vec![&config.asgard_vault_1_private_key],
                    )
                },
                |_, _, _, _| Ok(()),
            )
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    let swapper_account = state.swapper_account.get()?;
                    let swapper_account_str = swapper_account.to_string(AddressDisplayContext::with_encoder(&core.encoder()));
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-failed-auth-error",
                        |builder| {
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.asgard_vault_1_public_key,
                                        state.swapper_account.get()?,
                                        XRD,
                                        dec!(20),
                                        format!("=:XRD.XRD:{}::wr:100", swapper_account_str),
                                    ),
                                )
                                .done()
                        },
                        // Transaction should fail, because asgard_vault_2 key is used
                        vec![&config.asgard_vault_2_private_key],
                    )
                },
                |_, _, _, _| Ok(()),
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-between-asgard-vaults",
                        |builder| {
                            let resource_2 = state.maya_router_data.resource_2.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_between_asgard_vaults",
                                    manifest_args!(
                                        config.asgard_vault_1_public_key,
                                        config.asgard_vault_2_public_key,
                                        XRD,
                                        "migrate:3494355".to_string(),
                                    ),
                                )
                                .call_method(
                                    router_address,
                                    "transfer_between_asgard_vaults",
                                    manifest_args!(
                                        config.asgard_vault_1_public_key,
                                        config.asgard_vault_2_public_key,
                                        resource_2,
                                        "migrate:3494355".to_string(),
                                    ),
                                )
                                .done()
                        },
                        //
                        vec![&config.asgard_vault_1_private_key],
                    )
                },
            )
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    let swapper_account = state.swapper_account.get()?;
                    let swapper_account_str = swapper_account.to_string(AddressDisplayContext::with_encoder(&core.encoder()));
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-from-asgard-vault-1-resource-1-failed-asset-not-available",
                        |builder| {
                            // XRD shall no longer be available in the Asgard Vault 1
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.asgard_vault_1_public_key,
                                        swapper_account,
                                        XRD,
                                        dec!(30),
                                        format!("=:XRD.XRD:{}::wr:100", swapper_account_str),
                                    ),
                                )
                                .done()
                        },
                        vec![&config.asgard_vault_1_private_key],
                    )
                },
                |_, _, _, _| Ok(()),
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-deposit-to-asgard-vault-2",
                        |builder| {
                            let resource_2 = state.maya_router_data.resource_2.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            let swapper_account = state.swapper_account.get()?;
                            builder
                                .withdraw_from_account(state.swapper_account.get()?, resource_2, dec!(200))
                                .take_all_from_worktop(resource_2, "resource_2")
                                .with_bucket("resource_2", |builder, bucket| {
                                    builder.call_method(
                                        router_address,
                                        "deposit",
                                        manifest_args!(
                                            swapper_account,
                                            config.asgard_vault_2_public_key,
                                            bucket,
                                            "=:MAYA.CACAO:maya1x5979k5wqgq58f4864glr7w2rtgyuqqm6l2zhx::wr:100".to_string(),
                                        ),
                                    )
                                })
                                .done()
                        },
                        vec![&config.swapper_private_key],
                    )
                }
            )
            .successful_transaction(
                |core, config, state| {
                    let swapper_account = state.swapper_account.get()?;
                    let swapper_account_str = swapper_account.to_string(AddressDisplayContext::with_encoder(&core.encoder()));
                    core.next_transaction_with_faucet_lock_fee_fallible(
                        "maya-router-transfer-out-asgard-vault-2",
                        |builder| {
                            let resource_2 = state.maya_router_data.resource_2.get()?;
                            let router_address = state.maya_router_data.maya_router_address.get()?;
                            builder
                                .call_method(
                                    router_address,
                                    "transfer_out",
                                    manifest_args!(
                                        config.asgard_vault_2_public_key,
                                        swapper_account,
                                        resource_2,
                                        dec!(20),
                                        format!("=:XRD.FIZZ:{}::wr:100", swapper_account_str),
                                    ),
                                )
                                .done()
                        },
                        //
                        vec![&config.asgard_vault_2_private_key],
                    )
                },
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("owner_account", state.owner_account.get()?)
                        .add("swapper_account", state.swapper_account.get()?)
                        .add("maya_router_package", state.maya_router_package.get()?)
                        .add("maya_router_address", state.maya_router_data.maya_router_address.get()?)
                        .add("resource_1", state.maya_router_data.resource_1.get()?)
                        .add("resource_2", state.maya_router_data.resource_2.get()?)
                        .add("resource_3", state.maya_router_data.resource_3.get()?)
                })
            })
    }
}
