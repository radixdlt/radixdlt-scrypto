use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::object_modules::metadata::{
    MetadataValue, SingleMetadataVal, UncheckedOrigin, UncheckedUrl,
};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

use crate::internal_prelude::*;

#[allow(deprecated)]
pub struct MetadataScenarioConfig {
    pub user_account_1: PreallocatedAccount,
    pub user_account_sandbox: PreallocatedAccount,
    pub user_account_dashboard: PreallocatedAccount,
}

impl Default for MetadataScenarioConfig {
    fn default() -> Self {
        Self {
            user_account_1: secp256k1_account_1(),
            user_account_sandbox: secp256k1_account_sandbox(),
            user_account_dashboard: secp256k1_account_dashboard(),
        }
    }
}

#[derive(Default)]
pub struct MetadataScenarioState {
    pub package_with_metadata: Option<PackageAddress>,
    pub component_with_metadata: Option<ComponentAddress>,
    pub resource_with_metadata1: Option<ResourceAddress>,
    pub resource_with_metadata2: Option<ResourceAddress>,
}

pub struct MetadataScenarioCreator;

impl ScenarioCreator for MetadataScenarioCreator {
    type Config = MetadataScenarioConfig;
    type State = MetadataScenarioState;
    type Instance = MetadataScenario;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "metadata",
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
        Self::Instance {
            core,
            metadata: Self::METADATA,
            config,
            state: start_state,
        }
    }
}

pub struct MetadataScenario {
    core: ScenarioCore,
    metadata: ScenarioMetadata,
    config: MetadataScenarioConfig,
    state: MetadataScenarioState,
}

impl ScenarioInstance for MetadataScenario {
    fn metadata(&self) -> &ScenarioMetadata {
        &self.metadata
    }

    #[allow(deprecated)]
    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError> {
        let MetadataScenarioConfig {
            user_account_1,
            user_account_sandbox,
            user_account_dashboard,
        } = &self.config;
        let MetadataScenarioState {
            package_with_metadata,
            component_with_metadata,
            resource_with_metadata1,
            resource_with_metadata2,
        } = &mut self.state;
        let core = &mut self.core;

        let up_next = match core.next_stage() {
            1 => {
                core.check_start(&previous)?;

                let code = include_bytes!("../../assets/metadata.wasm");
                let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                    "../../assets/metadata.rpd"
                ))
                .unwrap();

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-create-package-with-metadata",
                    |builder| {
                        builder
                            .allocate_global_address(
                                PACKAGE_PACKAGE,
                                PACKAGE_BLUEPRINT,
                                "metadata_package_address_reservation",
                                "metadata_package_address",
                            )
                            .get_free_xrd_from_faucet()
                            .publish_package_advanced(
                                "metadata_package_address_reservation",
                                code.to_vec(),
                                schema,
                                create_metadata(),
                                radix_engine_interface::prelude::OwnerRole::Fixed(rule!(require(
                                    NonFungibleGlobalId::from_public_key(
                                        &user_account_1.public_key
                                    )
                                ))),
                            )
                            .try_deposit_entire_worktop_or_abort(user_account_1.address, None)
                    },
                    vec![],
                )
            }
            2 => {
                let commit_success = core.check_commit_success(core.check_previous(&previous)?)?;
                *package_with_metadata = Some(commit_success.new_package_addresses()[0]);

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-create-component-with-metadata",
                    |builder| {
                        let mut builder = builder
                            .allocate_global_address(
                                package_with_metadata.unwrap(),
                                "MetadataTest",
                                "metadata_component_address_reservation",
                                "metadata_component_address",
                            )
                            .get_free_xrd_from_faucet()
                            .call_function_with_name_lookup(
                                package_with_metadata.unwrap(),
                                "MetadataTest",
                                "new_with_address",
                                |lookup| {
                                    (lookup.address_reservation(
                                        "metadata_component_address_reservation",
                                    ),)
                                },
                            );
                        let address = builder.named_address("metadata_component_address");
                        for (k, v) in create_metadata() {
                            builder = builder.set_metadata(address, k, v);
                        }
                        builder.try_deposit_entire_worktop_or_abort(user_account_1.address, None)
                    },
                    vec![],
                )
            }
            3 => {
                let commit_success = core.check_commit_success(core.check_previous(&previous)?)?;
                *component_with_metadata = Some(commit_success.new_component_addresses()[0]);

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-create-resource-with-metadata",
                    |builder| {
                        builder
                            .get_free_xrd_from_faucet()
                            .create_fungible_resource(
                                OwnerRole::None,
                                false,
                                18,
                                FungibleResourceRoles {
                                    mint_roles: mint_roles! {
                                        minter => rule!(deny_all);
                                        minter_updater => rule!(deny_all);
                                    },
                                    burn_roles: burn_roles! {
                                        burner => rule!(allow_all);
                                        burner_updater => rule!(deny_all);
                                    },
                                    ..Default::default()
                                },
                                ModuleConfig {
                                    init: create_metadata().into(),
                                    roles: RoleAssignmentInit::default(),
                                },
                                Some(100_000_000_000u64.into()),
                            )
                            .try_deposit_entire_worktop_or_abort(user_account_1.address, None)
                    },
                    vec![],
                )
            }
            4 => {
                let commit_success = core.check_commit_success(core.check_previous(&previous)?)?;
                *resource_with_metadata1 = Some(commit_success.new_resource_addresses()[0]);

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-create-resource-with-metadata-partially-locked",
                    |builder| {
                        builder
                            .get_free_xrd_from_faucet()
                            .create_fungible_resource(
                                radix_engine_interface::prelude::OwnerRole::Fixed(rule!(require(
                                    NonFungibleGlobalId::from_public_key(
                                        &user_account_1.public_key
                                    )
                                ))),
                                false,
                                18,
                                FungibleResourceRoles {
                                    mint_roles: mint_roles! {
                                        minter => rule!(deny_all);
                                        minter_updater => rule!(deny_all);
                                    },
                                    burn_roles: burn_roles! {
                                        burner => rule!(allow_all);
                                        burner_updater => rule!(deny_all);
                                    },
                                    ..Default::default()
                                },
                                metadata! {
                                    init {
                                        "locked_on_create" => "Hello".to_owned(), locked;
                                        "locked_later" => "Hi".to_owned(), updatable;
                                    }
                                },
                                Some(100_000_000_000u64.into()),
                            )
                            .try_deposit_entire_worktop_or_abort(user_account_1.address, None)
                    },
                    vec![],
                )
            }
            5 => {
                let commit_success = core.check_commit_success(core.check_previous(&previous)?)?;
                *resource_with_metadata2 = Some(commit_success.new_resource_addresses()[0]);

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-update-initially-locked-metadata-fails",
                    |builder| {
                        builder.set_metadata(
                            resource_with_metadata2.unwrap(),
                            "locked_on_create",
                            MetadataValue::Bool(true),
                        )
                    },
                    vec![&user_account_1.key],
                )
            }
            6 => {
                core.check_commit_failure(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-update-updatable-metadata-succeeds",
                    |builder| {
                        builder.set_metadata(
                            resource_with_metadata2.unwrap(),
                            "locked_later",
                            MetadataValue::Bool(true),
                        )
                    },
                    vec![&user_account_1.key],
                )
            }
            7 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-lock-metadata",
                    |builder| {
                        builder.lock_metadata(resource_with_metadata2.unwrap(), "locked_later")
                    },
                    vec![&user_account_1.key],
                )
            }
            8 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-set-metadata-on-dashboard-account-succeeds",
                    |builder| {
                        builder
                            .set_metadata(
                                user_account_dashboard.address,
                                "account_type",
                                MetadataValue::String("dapp definition".to_owned()),
                            )
                            .set_metadata(
                                user_account_dashboard.address,
                                "name",
                                MetadataValue::String("Radix Dashboard".to_owned()),
                            )
                            .set_metadata(
                                user_account_dashboard.address,
                                "description",
                                MetadataValue::String("A collection of tools to assist with standard actions, and a place to look up anything on the ledger.".to_owned()))
                    },
                    vec![&user_account_dashboard.key],
                )
            }
            9 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-set-metadata-on-sandbox-account-succeeds",
                    |builder| {
                        builder
                            .set_metadata(
                                user_account_sandbox.address,
                                "account_type",
                                MetadataValue::String("dapp definition".to_owned()),
                            )
                            .set_metadata(
                                user_account_sandbox.address,
                                "name",
                                MetadataValue::String("Radix Sandbox dApp".to_owned()),
                            )
                    },
                    vec![&user_account_sandbox.key],
                )
            }
            10 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-update-recently-locked-metadata-fails",
                    |builder| {
                        builder.set_metadata(
                            resource_with_metadata2.unwrap(),
                            "locked_on_create",
                            MetadataValue::Bool(true),
                        )
                    },
                    vec![&user_account_1.key],
                )
            }
            _ => {
                core.check_commit_failure(core.check_previous(&previous)?)?;

                let output = ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("user_account_1", user_account_1.address)
                        .add("user_account_sandbox", user_account_sandbox.address)
                        .add("user_account_dashboard", user_account_dashboard.address)
                        .add("package_with_metadata", package_with_metadata.unwrap())
                        .add("component_with_metadata", component_with_metadata.unwrap())
                        .add("resource_with_metadata1", resource_with_metadata1.unwrap())
                        .add("resource_with_metadata2", resource_with_metadata2.unwrap()),
                };
                return Ok(NextAction::Completed(core.finish_scenario(output)));
            }
        };
        Ok(NextAction::Transaction(up_next?))
    }
}

fn create_metadata() -> BTreeMap<String, MetadataValue> {
    let mut metadata = BTreeMap::<String, MetadataValue>::new();

    add(
        &mut metadata,
        "string",
        &["Hello".to_string(), "world!".to_string()],
    );
    add(&mut metadata, "bool", &[true, false]);
    add(&mut metadata, "u8", &[1u8, 2u8]);
    add(&mut metadata, "u32", &[2u32, 3u32]);
    add(&mut metadata, "u64", &[3u64, 4u64]);
    add(&mut metadata, "i32", &[4i32, 5i32]);
    add(&mut metadata, "i64", &[5i64, 6i64]);
    add(&mut metadata, "decimal", &[dec!(1), dec!("2.1")]);
    add(&mut metadata, "address", &[GlobalAddress::from(XRD)]);
    add(
        &mut metadata,
        "public_key",
        &[
            PublicKey::Ed25519(Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])),
            PublicKey::Secp256k1(Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH])),
        ],
    );
    add(
        &mut metadata,
        "non_fungible_global_id",
        &[NonFungibleGlobalId::package_of_direct_caller_badge(
            POOL_PACKAGE,
        )],
    );
    add(
        &mut metadata,
        "non_fungible_local_id",
        &[
            NonFungibleLocalId::String(StringNonFungibleLocalId::new("Hello_world").unwrap()),
            NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(42)),
            NonFungibleLocalId::Bytes(BytesNonFungibleLocalId::new(vec![1u8]).unwrap()),
            NonFungibleLocalId::RUID(RUIDNonFungibleLocalId::new([1; 32])),
        ],
    );
    add(
        &mut metadata,
        "instant",
        &[Instant {
            seconds_since_unix_epoch: 1687446137,
        }],
    );
    add(
        &mut metadata,
        "url",
        &[UncheckedUrl::of("https://www.radixdlt.com")],
    );
    add(
        &mut metadata,
        "origin",
        &[UncheckedOrigin::of("https://www.radixdlt.com")],
    );
    add(
        &mut metadata,
        "public_key_hash",
        &[
            PublicKeyHash::Ed25519(Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]).get_hash()),
            PublicKeyHash::Secp256k1(
                Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]).get_hash(),
            ),
        ],
    );
    metadata
}

fn add<T: SingleMetadataVal + Clone>(
    metadata: &mut BTreeMap<String, MetadataValue>,
    name: &str,
    values: &[T],
) {
    metadata.insert(name.to_string(), values[0].clone().to_metadata_value());
    metadata.insert(
        format!("{}_array", name),
        T::to_array_metadata_value(values.to_vec()),
    );
}
