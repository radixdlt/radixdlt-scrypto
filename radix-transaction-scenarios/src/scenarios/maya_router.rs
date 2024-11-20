use crate::internal_prelude::*;
use crate::utils::{new_ed25519_private_key, new_secp256k1_private_key};
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

pub struct MayaRouterScenarioConfig {
    pub owner_private_key: PrivateKey,
    pub swapper_private_key: PrivateKey,
}

impl Default for MayaRouterScenarioConfig {
    fn default() -> Self {
        Self {
            owner_private_key: new_ed25519_private_key(3).into(),
            swapper_private_key: new_secp256k1_private_key(1).into(),
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
}

pub struct MayaRouterScenarioCreator;

impl ScenarioCreator for MayaRouterScenarioCreator {
    type Config = MayaRouterScenarioConfig;
    type State = MayaRouterScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "maya_router",
        protocol_min_requirement: ProtocolVersion::Bottlenose,
        protocol_max_requirement: ProtocolVersion::LATEST,
        testnet_run_at: Some(ProtocolVersion::Bottlenose),
        safe_to_run_on_used_ledger: true,
    };

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance {
        #[allow(unused_variables, deprecated)]
        ScenarioBuilder::new(core, Self::METADATA, config, start_state)
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
                    state.maya_router_data.resource_1.set(result.new_resource_addresses()[0]);
                    state.maya_router_data.resource_2.set(result.new_resource_addresses()[1]);
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
                                "maya_router_package_reservation",
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
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("owner_account", state.owner_account.get()?)
                        .add("swapper_account", state.swapper_account.get()?)
                        .add("maya_router_package", state.maya_router_package.get()?)
                        .add("maya_router_address", state.maya_router_data.maya_router_address.get()?)
                        .add("XRD", XRD)
                        .add("resource_1", state.maya_router_data.resource_1.get()?)
                        .add("resource_2", state.maya_router_data.resource_2.get()?)
                })
            })
    }
}
