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
    config: MetadataScenarioConfig,
}

pub struct MetadataScenarioConfig {
    /* Accounts */
    pub user_account_1: VirtualAccount,

    /* Entities - These get created during the scenario */
    pub package_with_metadata: Option<PackageAddress>,
    pub component_with_metadata: Option<ComponentAddress>,
    pub resource_with_metadata: Option<ResourceAddress>,
}

impl Default for MetadataScenarioConfig {
    fn default() -> Self {
        Self {
            user_account_1: secp256k1_account_1(),
            package_with_metadata: Default::default(),
            component_with_metadata: Default::default(),
            resource_with_metadata: Default::default(),
        }
    }
}

impl ScenarioDefinition for MetadataScenario {
    type Config = MetadataScenarioConfig;

    fn new_with_config(core: ScenarioCore, config: Self::Config) -> Self {
        Self { core, config }
    }
}

impl ScenarioInstance for MetadataScenario {
    fn metadata(&self) -> ScenarioMetadata {
        ScenarioMetadata {
            logical_name: "metadata",
        }
    }

    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError> {
        let MetadataScenarioConfig {
            user_account_1,
            package_with_metadata,
            component_with_metadata,
            resource_with_metadata,
        } = &mut self.config;
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
                let commit_success = core.check_commit_success(&previous)?;
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
                let commit_success = core.check_commit_success(&previous)?;
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

            _ => {
                let commit_success = core.check_commit_success(&previous)?;
                *resource_with_metadata = Some(commit_success.new_resource_addresses()[0]);

                let addresses = DescribedAddresses::new()
                    .add("user_account_1", user_account_1.address.clone())
                    .add("package_with_metadata", package_with_metadata.unwrap())
                    .add("component_with_metadata", component_with_metadata.unwrap())
                    .add("resource_with_metadata", resource_with_metadata.unwrap());
                return Ok(core.finish_scenario(addresses));
            }
        };
        Ok(NextAction::Transaction(up_next))
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
