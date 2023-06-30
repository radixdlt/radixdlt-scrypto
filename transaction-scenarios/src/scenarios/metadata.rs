use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{
    MetadataValue, Origin, SingleMetadataVal, Url,
};
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::*;

use crate::internal_prelude::*;

pub struct MetadataScenario {
    core: ScenarioCore,
    metadata: ScenarioMetadata,
    config: MetadataScenarioConfig,
    state: MetadataScenarioState,
}

pub struct MetadataScenarioConfig {
    pub user_account_1: VirtualAccount,
}

impl Default for MetadataScenarioConfig {
    fn default() -> Self {
        Self {
            user_account_1: secp256k1_account_1(),
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

impl ScenarioCreator for MetadataScenario {
    type Config = MetadataScenarioConfig;
    type State = MetadataScenarioState;

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "metadata",
        };
        Box::new(Self {
            core,
            metadata,
            config,
            state: start_state,
        })
    }
}

impl ScenarioInstance for MetadataScenario {
    fn metadata(&self) -> &ScenarioMetadata {
        &self.metadata
    }

    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError> {
        let MetadataScenarioConfig { user_account_1 } = &self.config;
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

                let code = include_bytes!("../../../assets/metadata.wasm");
                let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                    "../../../assets/metadata.schema"
                ))
                .unwrap();

                core.next_transaction_with_faucet_lock_fee(
                    "metadata-create-package-with-metadata",
                    |builder| {
                        builder.allocate_global_address(
                            BlueprintId {
                                package_address: PACKAGE_PACKAGE,
                                blueprint_name: PACKAGE_BLUEPRINT.to_owned(),
                            },
                            |builder, reservation, _named_address| {
                                builder
                                    .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                                    .publish_package_advanced(
                                        Some(reservation),
                                        code.to_vec(),
                                        schema,
                                        create_metadata(),
                                        radix_engine::types::OwnerRole::Fixed(rule!(require(
                                            NonFungibleGlobalId::from_public_key(
                                                &user_account_1.public_key
                                            )
                                        ))),
                                    )
                                    .try_deposit_batch_or_abort(user_account_1.address)
                            },
                        )
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
                        builder.allocate_global_address(
                            BlueprintId {
                                package_address: package_with_metadata.unwrap(),
                                blueprint_name: "MetadataTest".to_owned(),
                            },
                            |builder, reservation, named_address| {
                                let mut builder = builder
                                    .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                                    .call_function(
                                        package_with_metadata.unwrap(),
                                        "MetadataTest",
                                        "new_with_address",
                                        manifest_args!(reservation),
                                    );
                                for (k, v) in create_metadata() {
                                    builder = builder.set_metadata(named_address, k, v);
                                }
                                builder.try_deposit_batch_or_abort(user_account_1.address)
                            },
                        )
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
                            .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                            .create_fungible_resource(
                                OwnerRole::None,
                                false,
                                18,
                                ModuleConfig {
                                    init: create_metadata().into(),
                                    roles: RolesInit::default(),
                                },
                                btreemap! {
                                    Mint => (rule!(deny_all), rule!(deny_all)),
                                    Burn => (rule!(allow_all), rule!(deny_all))
                                },
                                Some(100_000_000_000u64.into()),
                            )
                            .try_deposit_batch_or_abort(user_account_1.address)
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
                            .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                            .create_fungible_resource(
                                radix_engine::types::OwnerRole::Fixed(rule!(require(
                                    NonFungibleGlobalId::from_public_key(
                                        &user_account_1.public_key
                                    )
                                ))),
                                false,
                                18,
                                metadata! {
                                    init {
                                        "locked_on_create" => "Hello".to_owned(), locked;
                                        "locked_later" => "Hi".to_owned(), updatable;
                                    }
                                },
                                btreemap! {
                                    Mint => (rule!(deny_all), rule!(deny_all)),
                                    Burn => (rule!(allow_all), rule!(deny_all)),
                                },
                                Some(100_000_000_000u64.into()),
                            )
                            .try_deposit_batch_or_abort(user_account_1.address)
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
                        builder.freeze_metadata(
                            resource_with_metadata2.unwrap().into(),
                            "locked_later".to_string(),
                        )
                    },
                    vec![&user_account_1.key],
                )
            }
            8 => {
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
                        .add("user_account_1", user_account_1.address.clone())
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
    add(&mut metadata, "decimal", &[dec!("1"), dec!("2.1")]);
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
            NonFungibleLocalId::String(
                StringNonFungibleLocalId::new("Hello_world".to_owned()).unwrap(),
            ),
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
        &[Url("https://www.radixdlt.com".to_owned())],
    );
    add(&mut metadata, "", &[Origin("www.radixdlt.com".to_owned())]);
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
