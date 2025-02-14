//! This is the same as Radiswap scenario, but:
//! * Updated to be safe to run on a used ledger
//! * Updated to use latest (as per Dugong) scenario coding style
//!
//! And then functionally, it's been tweaked to provide examples of the [blueprint linking] feature,
//! which is purely a metadata and Gateway feature.
//!
//! [blueprint linking]: https://docs.radixdlt.com/docs/metadata-for-verification

use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

use crate::internal_prelude::*;

pub struct RadiswapV2ScenarioConfig {
    pub radiswap_dapp_definition_key: PrivateKey,
    pub storing_account_key: PrivateKey,
    pub user_account_1_key: PrivateKey,
    pub user_account_2_key: PrivateKey,
    pub user_account_3_key: PrivateKey,
}

impl Default for RadiswapV2ScenarioConfig {
    fn default() -> Self {
        Self {
            radiswap_dapp_definition_key: new_ed25519_private_key(2).into(),
            storing_account_key: new_ed25519_private_key(3).into(),
            user_account_1_key: new_ed25519_private_key(4).into(),
            user_account_2_key: new_ed25519_private_key(5).into(),
            user_account_3_key: new_ed25519_private_key(6).into(),
        }
    }
}

#[derive(Default)]
pub struct RadiswapV2ScenarioState {
    radiswap_dapp_definition_account: State<ComponentAddress>,
    storing_account: State<ComponentAddress>,
    user_account_1: State<ComponentAddress>,
    user_account_2: State<ComponentAddress>,
    user_account_3: State<ComponentAddress>,
    owner_badge: State<NonFungibleGlobalId>,
    radiswap_package: State<PackageAddress>,
    pool_1: PoolData,
    pool_2: PoolData,
}

#[derive(Default)]
struct PoolData {
    radiswap: State<ComponentAddress>,
    pool: State<ComponentAddress>,
    resource_1: State<ResourceAddress>,
    resource_2: State<ResourceAddress>,
    pool_unit: State<ResourceAddress>,
}

pub struct RadiswapV2ScenarioCreator;

impl ScenarioCreator for RadiswapV2ScenarioCreator {
    type Config = RadiswapV2ScenarioConfig;
    type State = RadiswapV2ScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "radiswap_v2",
        protocol_min_requirement: ProtocolVersion::Dugong,
        protocol_max_requirement: ProtocolVersion::LATEST,
        testnet_run_at: Some(ProtocolVersion::Dugong),
        safe_to_run_on_used_ledger: true,
    };

    #[allow(unused_variables)]
    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance {
        ScenarioBuilder::new(core, Self::METADATA, config, start_state)
            .on_next_transaction_commit(|core, config, state, result| {
                let new_accounts = result.new_component_addresses();
                state.radiswap_dapp_definition_account.set(new_accounts[0]);
                state.storing_account.set(new_accounts[1]);
                state.user_account_1.set(new_accounts[2]);
                state.user_account_2.set(new_accounts[3]);
                state.user_account_3.set(new_accounts[4]);
                Ok(())
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("new-accounts")
                    .manifest_builder(|builder| {
                        builder.lock_fee_from_faucet()
                            .create_account_with_owner(None, OwnerRole::Fixed(rule!(require(
                                signature(config.radiswap_dapp_definition_key.public_key())
                            ))))
                            .create_account_with_owner(None, OwnerRole::Fixed(rule!(require(
                                signature(config.storing_account_key.public_key())
                            ))))
                            .create_account_with_owner(None, OwnerRole::Fixed(rule!(require(
                                signature(config.user_account_1_key.public_key())
                            ))))
                            .create_account_with_owner(None, OwnerRole::Fixed(rule!(require(
                                signature(config.user_account_2_key.public_key())
                            ))))
                            .create_account_with_owner(None, OwnerRole::Fixed(rule!(require(
                                signature(config.user_account_3_key.public_key())
                            ))))
                    })
                    .complete(core)
            })
            .on_next_transaction_commit(|core, config, state, result| {
                let new_resources = result.new_resource_addresses();
                state.pool_1.resource_1.set(XRD);
                state.pool_1.resource_2.set(new_resources[0]);
                state.pool_2.resource_1.set(new_resources[1]);
                state.pool_2.resource_2.set(new_resources[2]);
                Ok(())
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("new-resources")
                    .manifest_builder(|builder| {
                        builder.lock_fee_from_faucet()
                            .create_fungible_resource(
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
                                        "name" => "Ethereum Classic".to_owned(), locked;
                                        "symbol" => "ETC".to_owned(), locked;
                                        "description" => "The native token of the Ethereum Classic blockchain".to_owned(), locked;
                                        "tags" => vec!["p2p".to_owned(), "blockchain".to_owned(), "gas".to_owned()], locked;
                                        "icon_url" => "https://www.example.com/".to_owned(), locked;
                                        "info_url" => "https://www.example.com/".to_owned(), locked;
                                    }
                                },
                                Some(100_000_000_000u64.into()),
                            )
                            .try_deposit_entire_worktop_or_abort(state.storing_account.unwrap(), None)
                    })
                    .complete(core)
            })
            .on_next_transaction_commit(|core, config, state, result| {
                let new_resources = result.new_resource_addresses();
                state.owner_badge.set(NonFungibleGlobalId::new(
                    new_resources[0],
                    NonFungibleLocalId::integer(1)
                ));
                Ok(())
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("create-owner-badge-and-configure-dapp-definition-account")
                    .manifest_builder_with_lookup(|builder, lookup| {
                        let definition_account = state.radiswap_dapp_definition_account.unwrap();
                        builder.lock_fee_from_faucet()
                            .allocate_global_address(
                                RESOURCE_PACKAGE,
                                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                                "badge_reservation",
                                "badge"
                            )
                            .call_function(
                                RESOURCE_PACKAGE,
                                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                                (
                                    {
                                        // We wish to set the owner role of the badge to itself. i.e. something like this:
                                        // OwnerRole::Fixed(rule!(require(NonFungibleGlobalId::new(
                                        //     GlobalAddress::from(lookup.named_address("badge")),
                                        //     NonFungibleLocalId::integer(1)
                                        // ))))
                                        // But currently there aren't manifest models for this, so instead we need to do this manually
                                        // with ugly manual ManifestValues.
                                        let badge_non_fungible = ManifestValue::tuple([
                                            ManifestValue::custom(ManifestCustomValue::Address(lookup.named_address("badge").into())),
                                            ManifestValue::custom(ManifestCustomValue::NonFungibleLocalId(ManifestNonFungibleLocalId::integer(1).unwrap())),
                                        ]);
                                        ManifestValue::Enum {
                                            discriminator: 1, // OwnerRole::Fixed
                                            fields: vec![
                                                ManifestValue::Enum {
                                                    discriminator: 2, // AccessRule::Protected
                                                    fields: vec![
                                                        ManifestValue::Enum {
                                                            discriminator: 0, // CompositeRequirement::BasicRequirement
                                                            fields: vec![
                                                                ManifestValue::Enum {
                                                                    discriminator: 0, // BasicRequirement::Require
                                                                    fields: vec![
                                                                        ManifestValue::Enum {
                                                                            discriminator: 0, // ResourceOrNonFungible::NonFungible
                                                                            fields: vec![badge_non_fungible],
                                                                        }
                                                                    ],
                                                                }
                                                            ],
                                                        }
                                                    ],
                                                }
                                            ],
                                        }
                                    },
                                    NonFungibleIdType::Integer,
                                    true, // Track total supply
                                    NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(),
                                    indexmap!( // Entries
                                        NonFungibleLocalId::integer(1) => ((),),
                                    ),
                                    NonFungibleResourceRoles::default(),
                                    metadata! {
                                        init {
                                            "name" => "Radiswap - dApp Owner Badge", updatable;
                                            "description" => "[EXAMPLE] The owner badge for the Radiswap dApp and associated entities", updatable;
                                            "tags" => ["badge", "dex", "pool", "radiswap"], updatable;
                                            "info_url" => UncheckedUrl::of("https://radiswap.radixdlt.com/"), updatable;
                                        }
                                    },
                                    Some(lookup.address_reservation("badge_reservation")),
                                )
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
                    })
                    .sign(&config.radiswap_dapp_definition_key)
                    .complete(core)
            })
            .on_next_transaction_commit(|core, config, state, result| {
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
            })
            .successful_transaction(|core, config, state| {
                let code = include_bytes!("../../assets/radiswap.wasm");
                let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                    "../../assets/radiswap.rpd"
                ))
                .unwrap();
                let owner_role = OwnerRole::Fixed(rule!(require(
                    state.owner_badge.get()?
                )));
                core.v2_transaction("publish-and-create-pools")
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder.lock_fee_from_faucet()
                            .allocate_global_address(
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
                                    "name" => "Radiswap Package", updatable;
                                    "description" => "[EXAMPLE] A package of the logic of a Uniswap v2 style DEX.".to_owned(), updatable;
                                    "tags" => ["dex", "pool", "radiswap"], updatable;
                                    "info_url" => UncheckedUrl::of("https://radiswap.radixdlt.com/"), updatable;
                                },
                                owner_role.clone(),
                            ).call_function(
                                lookup.named_address("radiswap_package"),
                                "Radiswap",
                                "new",
                                (
                                    owner_role.clone(),
                                    state.pool_1.resource_1.unwrap(),
                                    state.pool_1.resource_2.unwrap(),
                                )
                            )
                            .call_function(
                                lookup.named_address("radiswap_package"),
                                "Radiswap",
                                "new",
                                (
                                    owner_role.clone(),
                                    state.pool_2.resource_1.unwrap(),
                                    state.pool_2.resource_2.unwrap(),
                                )
                            )
                            .try_deposit_entire_worktop_or_abort(state.radiswap_dapp_definition_account.unwrap(), None)
                    })
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("add-liquidity")
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder.lock_fee_from_faucet()
                            .get_free_xrd_from_faucet()
                            .withdraw_from_account(
                                state.storing_account.unwrap(),
                                state.pool_1.resource_2.unwrap(),
                                7000,
                            )
                            .withdraw_from_account(
                                state.storing_account.unwrap(),
                                state.pool_2.resource_1.unwrap(),
                                5000,
                            )
                            .withdraw_from_account(
                                state.storing_account.unwrap(),
                                state.pool_2.resource_2.unwrap(),
                                8000,
                            )
                            .take_all_from_worktop(
                                state.pool_1.resource_1.unwrap(),
                                "pool_1_resource_1"
                            )
                            .take_all_from_worktop(
                                state.pool_1.resource_2.unwrap(),
                                "pool_1_resource_2"
                            )
                            .call_method(
                                state.pool_1.radiswap.unwrap(),
                                "add_liquidity",
                                (
                                    lookup.bucket("pool_1_resource_1"),
                                    lookup.bucket("pool_1_resource_2"),
                                ),
                            )
                            .take_all_from_worktop(
                                state.pool_2.resource_1.unwrap(),
                                "pool_2_resource_1",
                            )
                            .take_all_from_worktop(
                                state.pool_2.resource_2.unwrap(),
                                "pool_2_resource_2",
                            )
                            .call_method(
                                state.pool_2.radiswap.unwrap(),
                                "add_liquidity",
                                (
                                    lookup.bucket("pool_2_resource_1"),
                                    lookup.bucket("pool_2_resource_2"),
                                ),
                            )
                            .try_deposit_entire_worktop_or_abort(state.storing_account.unwrap(), None)
                        })
                        .sign(&config.storing_account_key)
                        .complete(core)
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("distribute-tokens")
                    .manifest_builder(|mut builder| {
                        builder = builder.lock_fee_from_faucet()
                            .get_free_xrd_from_faucet()
                            .try_deposit_entire_worktop_or_abort(state.storing_account.unwrap(), None);
                        for destination_account in [
                            state.user_account_1.unwrap(),
                            state.user_account_2.unwrap(),
                            state.user_account_3.unwrap(),
                        ] {
                            for resource_address in [
                                state.pool_1.resource_1.unwrap(),
                                state.pool_1.resource_2.unwrap(),
                                state.pool_2.resource_1.unwrap(),
                                state.pool_2.resource_2.unwrap(),
                                state.pool_1.pool_unit.unwrap(),
                                state.pool_2.pool_unit.unwrap(),
                            ] {
                                builder = builder.withdraw_from_account(
                                    state.storing_account.unwrap(),
                                    resource_address,
                                    333,
                                );
                            }
                            builder = builder.try_deposit_entire_worktop_or_abort(destination_account, None);
                        }
                        builder
                    })
                    .sign(&config.storing_account_key)
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("swap-tokens")
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder.lock_fee_from_faucet()
                            .withdraw_from_account(
                                state.user_account_1.unwrap(),
                                state.pool_1.resource_1.unwrap(),
                                100,
                            )
                            .take_all_from_worktop(
                                state.pool_1.resource_1.unwrap(),
                                "input",
                            ).call_method(
                                state.pool_1.radiswap.unwrap(),
                                "swap",
                                (lookup.bucket("input"),)
                            )
                            .try_deposit_entire_worktop_or_abort(state.user_account_1.unwrap(), None)
                    })
                    .sign(&config.user_account_1_key)
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("remove-tokens")
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder.lock_fee_from_faucet()
                            .withdraw_from_account(
                                state.user_account_1.unwrap(),
                                state.pool_1.pool_unit.unwrap(),
                                100,
                            )
                            .take_all_from_worktop(
                                state.pool_1.pool_unit.unwrap(),
                                "pool_units",
                            )
                            .call_method(
                                state.pool_1.radiswap.unwrap(),
                                "remove_liquidity",
                                (lookup.bucket("pool_units"),),
                            )
                            .try_deposit_entire_worktop_or_abort(state.user_account_1.unwrap(), None)
                    })
                    .sign(&config.user_account_1_key)
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                let definition = GlobalAddress::from(state.radiswap_dapp_definition_account.get()?);
                let radiswap_package = GlobalAddress::from(state.radiswap_package.get()?);
                let owner_badge = GlobalAddress::from(state.owner_badge.get()?.resource_address());
                let radiswap_1 = GlobalAddress::from(state.pool_1.radiswap.get()?);
                let pool_1 = GlobalAddress::from(state.pool_1.pool.get()?);
                let pool_unit_1 = GlobalAddress::from(state.pool_1.pool_unit.get()?);
                let radiswap_2 = GlobalAddress::from(state.pool_2.radiswap.get()?);
                let pool_2 = GlobalAddress::from(state.pool_2.pool.get()?);
                let pool_unit_2 = GlobalAddress::from(state.pool_2.pool_unit.get()?);
                // radiswap_1 and radiswap_2 will not be claimed; but will be blueprint-linked
                // Unfortunately blueprint linking can't work with pools and pool units, because they have a native blueprint!
                let claimed_entities = vec![radiswap_package, owner_badge, pool_1, pool_unit_1, pool_2, pool_unit_2];
                fn add_metadata(
                    builder: TransactionManifestV2Builder,
                    address: GlobalAddress,
                    name: &'static str,
                    description: &'static str,
                ) -> TransactionManifestV2Builder {
                    builder
                        .set_metadata(address, "name", name)
                        .set_metadata(address, "description", description)
                        .set_metadata(address, "tags", ["dex", "pool", "radiswap"])
                        .set_metadata(address, "info_url", UncheckedUrl::of("https://radiswap.radixdlt.com/"))
                }
                core.v2_transaction("set-two-way-linking")
                    .manifest_builder(|builder| {
                        builder.lock_fee_from_faucet()
                            .create_proof_from_account_of_non_fungible(
                                state.radiswap_dapp_definition_account.unwrap(),
                                state.owner_badge.unwrap(),
                            )
                            // Set up two-way-linking
                            .set_metadata(definition, "claimed_entities", claimed_entities)
                            // Note - Components use "dapp_definition" but Resources use "dapp_definitions"
                            .set_metadata(radiswap_package, "dapp_definition", definition)
                            // Technically we should ensure the constructor is private before using `enable_blueprint_linking`
                            // but for the sake of this demonstration, it's not important.
                            .set_metadata(radiswap_package, "enable_blueprint_linking", ["Radiswap"])
                            .set_metadata(pool_1, "dapp_definition", definition)
                            .set_metadata(pool_2, "dapp_definition", definition)
                            .set_metadata(owner_badge, "dapp_definitions", [definition])
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
                    })
                    .sign(&config.radiswap_dapp_definition_key)
                    .complete(core)
            })
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("radiswap_dapp_definition_account", state.radiswap_dapp_definition_account.get()?)
                        .add("radiswap_dapp_owner_badge", state.owner_badge.get()?)
                        .add("storing_account", state.storing_account.get()?)
                        .add("user_account_1", state.user_account_1.get()?)
                        .add("user_account_2", state.user_account_2.get()?)
                        .add("user_account_3", state.user_account_3.get()?)
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
