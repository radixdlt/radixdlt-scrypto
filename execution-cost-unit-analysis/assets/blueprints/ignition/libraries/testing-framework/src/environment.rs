// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![allow(clippy::arithmetic_side_effects)]

use crate::prelude::*;

pub type ScryptoUnitEnv = Environment<ScryptoUnitEnvironmentSpecifier>;

pub trait EnvironmentSpecifier {
    // Environment
    type Environment;

    // Components
    type Ignition;
    type SimpleOracle;
    type OciswapV1Pool;
    type OciswapV2Pool;
    type DefiPlazaV2Pool;
    type CaviarnineV1Pool;

    type OciswapV1Adapter;
    type OciswapV2Adapter;
    type DefiPlazaV2Adapter;
    type CaviarnineV1Adapter;

    // Badges
    type Badge;
}

pub struct ScryptoUnitEnvironmentSpecifier;

impl EnvironmentSpecifier for ScryptoUnitEnvironmentSpecifier {
    // Environment
    type Environment = DefaultLedgerSimulator;

    // Components
    type Ignition = ComponentAddress;
    type SimpleOracle = ComponentAddress;
    type OciswapV1Pool = ComponentAddress;
    type OciswapV2Pool = ComponentAddress;
    type DefiPlazaV2Pool = ComponentAddress;
    type CaviarnineV1Pool = ComponentAddress;

    type OciswapV1Adapter = ComponentAddress;
    type OciswapV2Adapter = ComponentAddress;
    type DefiPlazaV2Adapter = ComponentAddress;
    type CaviarnineV1Adapter = ComponentAddress;

    // Badges
    type Badge = (PublicKey, PrivateKey, ComponentAddress, ResourceAddress);
}

/// The environment that Ignition is tested in.
///
/// This offers a set of convince methods for creating and initializing the
/// environment such that it is in a state that can easily be tested. First,
/// the appropriate package's substates are loaded and then flashed to the
/// environment making them available for use there, these substates come from
/// the mainnet state tree. Additionally, the needed resources are created,
/// and an oracle is created and registered in Ignition.
///
/// Ignition will be initialized with a protocol owner and a protocol manager
/// whose badges will be created in the initialization of the environment and
/// returned back to the caller. Additionally, the auth module will be disabled
/// by default for the created test environment. If it needs to be enabled then
/// that must happen after the creation of the environment.  
// Not quite a todo but more of a thought for the future. We publish all of the
// packages and create pools for all of the dexes when we realistically will not
// be using more than a single dex in a test (and if we do then its weird.). It
// would have been better for this to only be initialized with the state of the
// dex that we want instead of all of them. That would make it much faster to
// run.
pub struct Environment<S>
where
    S: EnvironmentSpecifier,
{
    /* Various entities */
    pub resources: ResourceInformation<ResourceAddress>,
    pub protocol: ProtocolEntities<S>,
    /* Supported Dexes */
    pub ociswap_v1: DexEntities<S::OciswapV1Pool, S::OciswapV1Adapter>,
    pub ociswap_v2: DexEntities<S::OciswapV2Pool, S::OciswapV2Adapter>,
    pub defiplaza_v2: DexEntities<S::DefiPlazaV2Pool, S::DefiPlazaV2Adapter>,
    pub caviarnine_v1: DexEntities<S::CaviarnineV1Pool, S::CaviarnineV1Adapter>,
}

impl<S> Environment<S>
where
    S: EnvironmentSpecifier,
{
    const PACKAGES_BINARY: &'static [u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/uncompressed_state.bin"));

    const PACKAGE_NAMES: [&'static str; 5] = [
        "ignition",
        "simple-oracle",
        "ociswap-v1-adapter-v1",
        "caviarnine-v1-adapter-v1",
        "caviarnine-v1-adapter-v2",
    ];

    const RESOURCE_DIVISIBILITIES: ResourceInformation<u8> = ResourceInformation::<u8> {
        bitcoin: 8,
        ethereum: 18,
        usdc: 6,
        usdt: 6,
    };
}

impl ScryptoUnitEnv {
    pub fn new(
        package_loader: impl FnMut(&std::path::Path) -> (Vec<u8>, PackageDefinition),
        ledger: &mut DefaultLedgerSimulator,
    ) -> Self {
        Self::new_with_configuration(Configuration::default(), package_loader, ledger)
    }

    pub fn new_with_configuration(
        configuration: Configuration,
        mut package_loader: impl FnMut(&std::path::Path) -> (Vec<u8>, PackageDefinition),
        ledger: &mut DefaultLedgerSimulator,
    ) -> Self {
        // Flash the substates to the ledger so that they can be used in tests.
        let (addresses, db_flash) =
            scrypto_decode::<(Vec<NodeId>, DbFlash)>(Self::PACKAGES_BINARY).expect("Can't fail!");

        let caviarnine_v1_package = PackageAddress::try_from(addresses[0]).unwrap();
        let ociswap_v1_package = PackageAddress::try_from(addresses[1]).unwrap();

        ledger.substate_db_mut().commit(&DatabaseUpdates {
            node_updates: db_flash
                .into_iter()
                .map(|(db_node_key, partition_num_to_updates_mapping)| {
                    (
                        db_node_key,
                        NodeDatabaseUpdates {
                            partition_updates: partition_num_to_updates_mapping
                                .into_iter()
                                .map(|(partition_num, substates)| {
                                    (
                                        partition_num,
                                        PartitionDatabaseUpdates::Delta {
                                            substate_updates: substates
                                                .into_iter()
                                                .map(|(db_sort_key, value)| {
                                                    (db_sort_key, DatabaseUpdate::Set(value))
                                                })
                                                .collect(),
                                        },
                                    )
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        });

        // The account that everything gets deposited into throughout the tests.
        let (public_key, private_key, account) = ledger.new_account(false);

        let protocol_manager_badge = ledger.create_fungible_resource(dec!(1), 0, account);
        let protocol_owner_badge = ledger.create_fungible_resource(dec!(1), 0, account);

        let protocol_manager_rule = rule!(require(protocol_manager_badge));
        let protocol_owner_rule = rule!(require(protocol_owner_badge));

        let [ignition_package, simple_oracle_package, ociswap_v1_adapter_v1_package, caviarnine_v1_adapter_v1_package, caviarnine_v1_adapter_v2_package] =
            Self::PACKAGE_NAMES.map(|package_name| {
                let (code, definition) = package_loader(
                    std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                        .join("..")
                        .join("..")
                        .join("packages")
                        .join(package_name)
                        .join("Cargo.toml")
                        .as_path(),
                );
                ledger.publish_package((code, definition), Default::default(), OwnerRole::None)
            });
        let caviarnine_v1_adapter_package = match configuration.caviarnine_adapter_version {
            CaviarnineAdapterVersion::One => caviarnine_v1_adapter_v1_package,
            CaviarnineAdapterVersion::Two => caviarnine_v1_adapter_v2_package,
        };

        let resource_addresses = Self::RESOURCE_DIVISIBILITIES.map(|divisibility| {
            ledger.create_freely_mintable_fungible_resource(
                OwnerRole::None,
                None,
                *divisibility,
                account,
            )
        });

        let [ociswap_v1_liquidity_receipt_resource, ociswap_v2_liquidity_receipt_resource, defiplaza_v2_liquidity_receipt_resource, caviarnine_v1_liquidity_receipt_resource] =
            std::array::from_fn(|_| {
                ledger
                    .execute_manifest(
                        ManifestBuilder::new()
                            .lock_fee_from_faucet()
                            .call_function(
                                RESOURCE_PACKAGE,
                                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT,
                                NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                                    owner_role: OwnerRole::None,
                                    track_total_supply: true,
                                    non_fungible_schema:
                                        NonFungibleDataSchema::new_local_without_self_package_replacement::<
                                            LiquidityReceipt<AnyValue>,
                                        >(),
                                    entries: vec![],
                                    resource_roles: NonFungibleResourceRoles {
                                        mint_roles: mint_roles! {
                                            minter => rule!(allow_all);
                                            minter_updater => rule!(allow_all);
                                        },
                                        burn_roles: burn_roles! {
                                            burner => rule!(allow_all);
                                            burner_updater => rule!(allow_all);
                                        },
                                        ..Default::default()
                                    },
                                    metadata: Default::default(),
                                    address_reservation: Default::default(),
                                },
                            )
                            .build(),
                        vec![],
                    )
                    .expect_commit_success()
                    .new_resource_addresses()
                    .first()
                    .copied()
                    .unwrap()
            });

        let ociswap_v1_pools = resource_addresses.map(|resource_address| {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .ociswap_v1_pool_instantiate(
                    ociswap_v1_package,
                    *resource_address,
                    XRD,
                    configuration.fees,
                    FAUCET,
                )
                .build();
            let component_address = *ledger
                .execute_manifest(manifest, vec![])
                .expect_commit_success()
                .new_component_addresses()
                .first()
                .unwrap();

            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(XRD, dec!(100_000_000))
                .mint_fungible(*resource_address, dec!(100_000_000))
                .take_all_from_worktop(XRD, "xrd_bucket")
                .take_all_from_worktop(*resource_address, "other_bucket")
                .with_name_lookup(|builder, _| {
                    let xrd_bucket = builder.bucket("xrd_bucket");
                    let other_bucket = builder.bucket("other_bucket");
                    builder.ociswap_v1_pool_add_liquidity(
                        component_address,
                        xrd_bucket,
                        other_bucket,
                    )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build();
            ledger
                .execute_manifest_without_auth(manifest)
                .expect_commit_success();

            component_address
        });

        let caviarnine_v1_pools = resource_addresses.map(|resource_address| {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .allocate_global_address(
                    caviarnine_v1_package,
                    "QuantaSwap",
                    "reservation",
                    "address",
                )
                .mint_fungible(XRD, dec!(100_000_000))
                .mint_fungible(*resource_address, dec!(100_000_000))
                .take_all_from_worktop(XRD, "xrd_bucket")
                .take_all_from_worktop(*resource_address, "other_bucket")
                .with_name_lookup(|builder, _| {
                    let reservation = builder.address_reservation("reservation");
                    let address = builder.named_address("address");

                    let xrd_bucket = builder.bucket("xrd_bucket");
                    let other_bucket = builder.bucket("other_bucket");

                    builder
                        .caviarnine_v1_pool_new(
                            caviarnine_v1_package,
                            rule!(allow_all),
                            rule!(allow_all),
                            *resource_address,
                            XRD,
                            50,
                            Some(reservation),
                        )
                        .caviarnine_v1_pool_add_liquidity(
                            address,
                            other_bucket,
                            xrd_bucket,
                            vec![(27000, dec!(100_000_000), dec!(100_000_000))],
                        )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build();
            *ledger
                .execute_manifest_without_auth(manifest)
                .expect_commit_success()
                .new_component_addresses()
                .first()
                .unwrap()
        });

        let (ociswap_v2_package, ociswap_v2_adapter_v1_package, ociswap_v2_pools) = {
            let ociswap_v2_pool_package = {
                let ociswap_v2_package_wasm = include_bytes!("../assets/ociswap_v2_pool.wasm");
                let ociswap_v2_package_rpd = include_bytes!("../assets/ociswap_v2_pool.rpd");
                let ociswap_v2_package_definition =
                    manifest_decode::<PackageDefinition>(ociswap_v2_package_rpd).unwrap();

                ledger.publish_package(
                    (
                        ociswap_v2_package_wasm.to_vec(),
                        ociswap_v2_package_definition,
                    ),
                    Default::default(),
                    Default::default(),
                )
            };
            let ociswap_v2_registry_package = {
                let ociswap_v2_package_wasm = include_bytes!("../assets/ociswap_v2_registry.wasm");
                let ociswap_v2_package_rpd = include_bytes!("../assets/ociswap_v2_registry.rpd");
                let ociswap_v2_package_definition =
                    manifest_decode::<PackageDefinition>(ociswap_v2_package_rpd).unwrap();

                ledger.publish_package(
                    (
                        ociswap_v2_package_wasm.to_vec(),
                        ociswap_v2_package_definition,
                    ),
                    Default::default(),
                    Default::default(),
                )
            };

            let registry = ledger
                .execute_manifest(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .ociswap_v2_registry_instantiate(
                            ociswap_v2_registry_package,
                            GLOBAL_CALLER_RESOURCE,
                            dec!(0.03),
                            10080,
                            20,
                        )
                        .build(),
                    vec![],
                )
                .expect_commit_success()
                .new_component_addresses()
                .first()
                .copied()
                .unwrap();

            let (code, definition) = package_loader(
                std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                    .join("..")
                    .join("..")
                    .join("packages")
                    .join("ociswap-v2-adapter-v1")
                    .join("Cargo.toml")
                    .as_path(),
            );
            let ociswap_v2_adapter_v1_package =
                ledger.publish_package((code, definition), Default::default(), OwnerRole::None);

            let ociswap_v2_pools = resource_addresses.map(|resource_address| {
                let (resource_x, resource_y) = if XRD < *resource_address {
                    (XRD, *resource_address)
                } else {
                    (*resource_address, XRD)
                };

                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .ociswap_v2_pool_instantiate(
                        ociswap_v2_pool_package,
                        resource_x,
                        resource_y,
                        pdec!(1),
                        60,
                        dec!(0.03),
                        dec!(0.03),
                        registry,
                        vec![],
                        FAUCET,
                    )
                    .build();
                let component_address = *ledger
                    .execute_manifest(manifest, vec![])
                    .expect_commit_success()
                    .new_component_addresses()
                    .first()
                    .unwrap();

                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .mint_fungible(XRD, dec!(100_000_000))
                    .mint_fungible(*resource_address, dec!(100_000_000))
                    .take_all_from_worktop(resource_x, "resource_x_bucket")
                    .take_all_from_worktop(resource_y, "resource_y_bucket")
                    .with_name_lookup(|builder, _| {
                        let resource_x_bucket = builder.bucket("resource_x_bucket");
                        let resource_y_bucket = builder.bucket("resource_y_bucket");
                        builder.ociswap_v2_pool_add_liquidity(
                            component_address,
                            -10_000,
                            10_000,
                            resource_x_bucket,
                            resource_y_bucket,
                        )
                    })
                    .try_deposit_entire_worktop_or_abort(account, None)
                    .build();
                ledger
                    .execute_manifest_without_auth(manifest)
                    .expect_commit_success();

                component_address
            });

            (
                ociswap_v2_pool_package,
                ociswap_v2_adapter_v1_package,
                ociswap_v2_pools,
            )
        };

        let (defiplaza_v2_package, defiplaza_v2_adapter_v1_package, defiplaza_v2_pools) = {
            let defiplaza_v2_pool_package = {
                let defiplaza_v2_package_wasm = include_bytes!("../assets/defiplaza_v2.wasm");
                let defiplaza_v2_package_rpd = include_bytes!("../assets/defiplaza_v2.rpd");
                let defiplaza_v2_package_definition =
                    manifest_decode::<PackageDefinition>(defiplaza_v2_package_rpd).unwrap();

                ledger.publish_package(
                    (
                        defiplaza_v2_package_wasm.to_vec(),
                        defiplaza_v2_package_definition,
                    ),
                    Default::default(),
                    Default::default(),
                )
            };

            let (code, definition) = package_loader(
                std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
                    .join("..")
                    .join("..")
                    .join("packages")
                    .join("defiplaza-v2-adapter-v1")
                    .join("Cargo.toml")
                    .as_path(),
            );
            let defiplaza_v2_adapter_v1_package =
                ledger.publish_package((code, definition), Default::default(), OwnerRole::None);

            let defiplaza_v2_pools = resource_addresses.map(|resource_address| {
                let (resource_x, resource_y) = if XRD < *resource_address {
                    (XRD, *resource_address)
                } else {
                    (*resource_address, XRD)
                };

                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .defi_plaza_v2_pool_instantiate_pair(
                        defiplaza_v2_pool_package,
                        OwnerRole::None,
                        resource_x,
                        resource_y,
                        PairConfig {
                            k_in: dec!("0.4"),
                            k_out: dec!("1"),
                            fee: dec!("0"),
                            decay_factor: dec!("0.9512"),
                        },
                        dec!(1),
                    )
                    .build();
                let component_address = *ledger
                    .execute_manifest(manifest, vec![])
                    .expect_commit_success()
                    .new_component_addresses()
                    .first()
                    .unwrap();

                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .mint_fungible(XRD, dec!(100_000_000))
                    .mint_fungible(*resource_address, dec!(100_000_000))
                    .take_all_from_worktop(resource_x, "resource_x_bucket")
                    .take_all_from_worktop(resource_y, "resource_y_bucket")
                    .with_name_lookup(|builder, _| {
                        let resource_x_bucket = builder.bucket("resource_x_bucket");
                        let resource_y_bucket = builder.bucket("resource_y_bucket");
                        builder
                            .defi_plaza_v2_pool_add_liquidity(
                                component_address,
                                resource_x_bucket,
                                None,
                            )
                            .defi_plaza_v2_pool_add_liquidity(
                                component_address,
                                resource_y_bucket,
                                None,
                            )
                    })
                    .try_deposit_entire_worktop_or_abort(account, None)
                    .build();
                ledger
                    .execute_manifest_without_auth(manifest)
                    .expect_commit_success();

                component_address
            });

            (
                defiplaza_v2_pool_package,
                defiplaza_v2_adapter_v1_package,
                defiplaza_v2_pools,
            )
        };

        let simple_oracle = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        simple_oracle_package,
                        "SimpleOracle",
                        "instantiate",
                        (
                            protocol_manager_rule.clone(),
                            MetadataInit::default(),
                            OwnerRole::None,
                            None::<ManifestAddressReservation>,
                        ),
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();

        // Submitting some dummy prices to the oracle to get things going.
        resource_addresses.map(|resource_address| {
            ledger
                .execute_manifest_without_auth(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .call_method(
                            simple_oracle,
                            "set_price",
                            (*resource_address, XRD, dec!(1)),
                        )
                        .build(),
                )
                .expect_commit_success();
        });

        // Initializing ignition with information
        let ignition = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        ignition_package,
                        "Ignition",
                        "instantiate",
                        (
                            MetadataInit::default(),
                            OwnerRole::None,
                            protocol_owner_rule,
                            protocol_manager_rule,
                            XRD,
                            simple_oracle,
                            configuration.maximum_allowed_price_staleness_in_seconds_seconds,
                            configuration.maximum_allowed_relative_price_difference,
                            InitializationParametersManifest::default(),
                            None::<ManifestAddressReservation>,
                        ),
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();

        // Submit the matching factor to Ignition
        {
            let manifest = ociswap_v1_pools
                .iter()
                .chain(ociswap_v2_pools.iter())
                .chain(caviarnine_v1_pools.iter())
                .chain(defiplaza_v2_pools.iter())
                .fold(
                    ManifestBuilder::new().lock_fee_from_faucet(),
                    |builder, address| {
                        builder.call_method(
                            ignition,
                            "upsert_matching_factor",
                            (address, Decimal::ONE),
                        )
                    },
                )
                .build();
            ledger
                .execute_manifest_without_auth(manifest)
                .expect_commit_success();
        }

        let [ociswap_v1_adapter_v1, ociswap_v2_adapter_v1, defiplaza_v2_adapter_v1, caviarnine_v1_adapter] =
            [
                (ociswap_v1_adapter_v1_package, "OciswapV1Adapter"),
                (ociswap_v2_adapter_v1_package, "OciswapV2Adapter"),
                (defiplaza_v2_adapter_v1_package, "DefiPlazaV2Adapter"),
                (caviarnine_v1_adapter_package, "CaviarnineV1Adapter"),
            ]
            .map(|(package_address, blueprint_name)| {
                ledger
                    .execute_manifest(
                        ManifestBuilder::new()
                            .lock_fee_from_faucet()
                            .call_function(
                                package_address,
                                blueprint_name,
                                "instantiate",
                                (
                                    rule!(allow_all),
                                    rule!(allow_all),
                                    MetadataInit::default(),
                                    OwnerRole::None,
                                    None::<ManifestAddressReservation>,
                                ),
                            )
                            .build(),
                        vec![],
                    )
                    .expect_commit_success()
                    .new_component_addresses()
                    .first()
                    .copied()
                    .unwrap()
            });

        ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        defiplaza_v2_adapter_v1,
                        "add_pair_configs",
                        (defiplaza_v2_pools
                            .iter()
                            .map(|address| {
                                (
                                    address,
                                    PairConfig {
                                        k_in: dec!("0.4"),
                                        k_out: dec!("1"),
                                        fee: dec!("0"),
                                        decay_factor: dec!("0.9512"),
                                    },
                                )
                            })
                            .collect::<IndexMap<_, _>>(),),
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success();

        // Cache the addresses of the various Caviarnine pools.
        ledger
            .execute_manifest(
                TransactionManifestV1 {
                    instructions: std::iter::once(InstructionV1::CallMethod {
                        address: FAUCET.into(),
                        method_name: "lock_fee".into(),
                        args: manifest_args!(dec!(100)).into(),
                    })
                    .chain(
                        caviarnine_v1_pools
                            .iter()
                            .map(|address| InstructionV1::CallMethod {
                                address: caviarnine_v1_adapter.into(),
                                method_name: "preload_pool_information".to_owned(),
                                args: manifest_args!(address).into(),
                            }),
                    )
                    .collect(),
                    blobs: Default::default(),
                },
                vec![],
            )
            .expect_commit_success();

        {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(ignition, "set_is_open_position_enabled", (true,))
                .call_method(ignition, "set_is_close_position_enabled", (true,))
                .call_method(
                    ignition,
                    "add_reward_rate",
                    (LockupPeriod::from_months(6).unwrap(), dec!(0.2)),
                )
                .call_method(
                    ignition,
                    "add_reward_rate",
                    (LockupPeriod::from_months(12).unwrap(), dec!(0.4)),
                )
                .mint_fungible(XRD, dec!(200_000_000_000_000))
                .take_from_worktop(XRD, dec!(100_000_000_000_000), "volatile")
                .take_from_worktop(XRD, dec!(100_000_000_000_000), "non_volatile")
                .with_name_lookup(|builder, _| {
                    let volatile = builder.bucket("volatile");
                    let non_volatile = builder.bucket("non_volatile");

                    builder
                        .call_method(
                            ignition,
                            "deposit_protocol_resources",
                            (volatile, Volatility::Volatile),
                        )
                        .call_method(
                            ignition,
                            "deposit_protocol_resources",
                            (non_volatile, Volatility::NonVolatile),
                        )
                })
                .with_name_lookup(|mut builder, _| {
                    let ResourceInformation {
                        bitcoin,
                        ethereum,
                        usdc,
                        usdt,
                    } = resource_addresses;

                    for instruction in [
                        (bitcoin, Volatility::Volatile),
                        (ethereum, Volatility::Volatile),
                        (usdc, Volatility::NonVolatile),
                        (usdt, Volatility::NonVolatile),
                    ]
                    .map(|(address, volatility)| InstructionV1::CallMethod {
                        address: ignition.into(),
                        method_name: "insert_user_resource_volatility".to_owned(),
                        args: manifest_args!(address, volatility).into(),
                    }) {
                        builder = builder.add_instruction_advanced(instruction).0;
                    }

                    for (
                        adapter_address,
                        pools,
                        liquidity_receipt,
                        package_address,
                        blueprint_name,
                    ) in [
                        (
                            ociswap_v1_adapter_v1,
                            ociswap_v1_pools,
                            ociswap_v1_liquidity_receipt_resource,
                            ociswap_v1_package,
                            "BasicPool",
                        ),
                        (
                            ociswap_v2_adapter_v1,
                            ociswap_v2_pools,
                            ociswap_v2_liquidity_receipt_resource,
                            ociswap_v2_package,
                            "LiquidityPool",
                        ),
                        (
                            defiplaza_v2_adapter_v1,
                            defiplaza_v2_pools,
                            defiplaza_v2_liquidity_receipt_resource,
                            defiplaza_v2_package,
                            "PlazaPair",
                        ),
                        (
                            caviarnine_v1_adapter,
                            caviarnine_v1_pools,
                            caviarnine_v1_liquidity_receipt_resource,
                            caviarnine_v1_package,
                            "QuantaSwap",
                        ),
                    ] {
                        builder = builder.call_method(
                            ignition,
                            "insert_pool_information",
                            (
                                BlueprintId {
                                    package_address,
                                    blueprint_name: blueprint_name.to_owned(),
                                },
                                (
                                    adapter_address,
                                    pools.iter().collect::<Vec<_>>(),
                                    liquidity_receipt,
                                ),
                            ),
                        );
                    }

                    builder
                })
                .build();
            ledger
                .execute_manifest_with_enabled_modules(
                    manifest,
                    EnabledModules::for_test_transaction()
                        & !EnabledModules::AUTH
                        & !EnabledModules::COSTING,
                )
                .expect_commit_success();
        }

        Self {
            resources: resource_addresses,
            protocol: ProtocolEntities {
                ignition_package_address: ignition_package,
                ignition,
                oracle_package_address: simple_oracle_package,
                oracle: simple_oracle,
                protocol_owner_badge: (
                    public_key.into(),
                    Secp256k1PrivateKey::from_bytes(&private_key.to_bytes())
                        .unwrap()
                        .into(),
                    account,
                    protocol_owner_badge,
                ),
                protocol_manager_badge: (
                    public_key.into(),
                    private_key.into(),
                    account,
                    protocol_manager_badge,
                ),
            },
            ociswap_v1: DexEntities {
                package: ociswap_v1_package,
                pools: ociswap_v1_pools,
                adapter_package: ociswap_v1_adapter_v1_package,
                adapter: ociswap_v1_adapter_v1,
                liquidity_receipt: ociswap_v1_liquidity_receipt_resource,
            },
            ociswap_v2: DexEntities {
                package: ociswap_v2_package,
                pools: ociswap_v2_pools,
                adapter_package: ociswap_v2_adapter_v1_package,
                adapter: ociswap_v2_adapter_v1,
                liquidity_receipt: ociswap_v2_liquidity_receipt_resource,
            },
            defiplaza_v2: DexEntities {
                package: defiplaza_v2_package,
                pools: defiplaza_v2_pools,
                adapter_package: defiplaza_v2_adapter_v1_package,
                adapter: defiplaza_v2_adapter_v1,
                liquidity_receipt: defiplaza_v2_liquidity_receipt_resource,
            },
            caviarnine_v1: DexEntities {
                package: caviarnine_v1_package,
                pools: caviarnine_v1_pools,
                adapter_package: caviarnine_v1_adapter_package,
                adapter: caviarnine_v1_adapter,
                liquidity_receipt: caviarnine_v1_liquidity_receipt_resource,
            },
        }
    }
}

#[derive(Debug)]
pub struct ProtocolEntities<S>
where
    S: EnvironmentSpecifier,
{
    /* Ignition */
    pub ignition_package_address: PackageAddress,
    pub ignition: S::Ignition,
    /* Oracle */
    pub oracle_package_address: PackageAddress,
    pub oracle: S::SimpleOracle,
    /* Badges */
    pub protocol_owner_badge: S::Badge,
    pub protocol_manager_badge: S::Badge,
}

/// A struct that defines the entities that belong to a Decentralized Exchange.
/// it contains the package address as well as generic items [`T`] which are
/// the stubs used to call the pools.
#[derive(Copy, Clone, Debug)]
pub struct DexEntities<P, A> {
    /* Packages */
    pub package: PackageAddress,
    /* Pools */
    pub pools: ResourceInformation<P>,
    /* Adapter */
    pub adapter_package: PackageAddress,
    pub adapter: A,
    /* Receipt */
    pub liquidity_receipt: ResourceAddress,
}

#[derive(Clone, Debug, Copy)]
pub struct ResourceInformation<T> {
    pub bitcoin: T,
    pub ethereum: T,
    pub usdc: T,
    pub usdt: T,
}

impl<T> ResourceInformation<T> {
    pub fn map<F, O>(&self, mut map: F) -> ResourceInformation<O>
    where
        F: FnMut(&T) -> O,
    {
        ResourceInformation::<O> {
            bitcoin: map(&self.bitcoin),
            ethereum: map(&self.ethereum),
            usdc: map(&self.usdc),
            usdt: map(&self.usdt),
        }
    }

    pub fn try_map<F, O, E>(&self, mut map: F) -> Result<ResourceInformation<O>, E>
    where
        F: FnMut(&T) -> Result<O, E>,
    {
        Ok(ResourceInformation::<O> {
            bitcoin: map(&self.bitcoin)?,
            ethereum: map(&self.ethereum)?,
            usdc: map(&self.usdc)?,
            usdt: map(&self.usdt)?,
        })
    }

    pub fn iter(self) -> impl Iterator<Item = T> {
        [self.bitcoin, self.ethereum, self.usdc, self.usdt].into_iter()
    }

    pub fn zip<O>(self, other: ResourceInformation<O>) -> ResourceInformation<(T, O)> {
        ResourceInformation {
            bitcoin: (self.bitcoin, other.bitcoin),
            ethereum: (self.ethereum, other.ethereum),
            usdc: (self.usdc, other.usdc),
            usdt: (self.usdt, other.usdt),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Configuration {
    pub fees: Decimal,
    pub maximum_allowed_price_staleness_in_seconds_seconds: i64,
    pub maximum_allowed_relative_price_difference: Decimal,
    pub caviarnine_adapter_version: CaviarnineAdapterVersion,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            // 1%
            fees: dec!(0.01),
            // 5 Minutes
            maximum_allowed_price_staleness_in_seconds_seconds: 300i64,
            // 1%
            maximum_allowed_relative_price_difference: dec!(0.01),
            // Version One.
            caviarnine_adapter_version: CaviarnineAdapterVersion::One,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CaviarnineAdapterVersion {
    One,
    Two,
}
