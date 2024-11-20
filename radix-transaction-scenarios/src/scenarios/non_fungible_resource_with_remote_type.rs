use crate::internal_prelude::*;
use radix_blueprint_schema_init::*;
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

#[allow(deprecated)]
pub struct NonFungibleResourceWithRemoteTypeScenarioConfig {
    pub main_account: PreallocatedAccount,
}

#[derive(Default)]
pub struct NonFungibleResourceWithRemoteTypeScenarioState {
    pub package_with_registered_types: Option<PackageAddress>,
    pub blueprint_with_registered_types: Option<String>,
    pub non_fungible_resource_using_registered_type: Option<ResourceAddress>,
}

impl Default for NonFungibleResourceWithRemoteTypeScenarioConfig {
    fn default() -> Self {
        Self {
            main_account: secp256k1_account_1(),
        }
    }
}

pub struct NonFungibleResourceWithRemoteTypeScenarioCreator;

impl ScenarioCreator for NonFungibleResourceWithRemoteTypeScenarioCreator {
    type Config = NonFungibleResourceWithRemoteTypeScenarioConfig;
    type State = NonFungibleResourceWithRemoteTypeScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "non_fungible_resource_with_remote_type",
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
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-with-remote-type-registration",
                        |builder| {
                            // Load code and schema
                            let code = include_bytes!("../../assets/radiswap.wasm");
                            let mut schema = manifest_decode::<PackageDefinition>(include_bytes!(
                                "../../assets/radiswap.rpd"
                            ))
                            .unwrap();

                            // Register `RemoveLiquidityEvent` as TestType
                            let type_id = if let TypeRef::Static(type_id) = schema.blueprints.values_mut().next().unwrap().schema.events.event_schema.get("RemoveLiquidityEvent").unwrap() {
                                *type_id
                            } else {
                                unreachable!()
                            };
                            schema.blueprints.values_mut().next().unwrap().schema.types.type_schema.insert("RemoveLiquidityEvent".to_owned(), type_id);

                            // Build manifest for publishing the package
                            builder
                                .publish_package_advanced(
                                    None,
                                    code.to_vec(),
                                    schema,
                                    metadata_init! {
                                        "name" => "Radiswap Package (for the non-fungible resource with remote type scenario)", locked;
                                    },
                                    OwnerRole::None,
                                )
                                .deposit_entire_worktop(config.main_account.address)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| {
                    state.package_with_registered_types = Some(result.new_package_addresses()[0]);
                    state.blueprint_with_registered_types = Some("Radiswap".to_owned());
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-with-remote-type",
                        |builder| {
                            #[derive(ScryptoSbor, ManifestSbor)]
                            pub struct RemoveLiquidityEvent {
                                pub pool_units_amount: Decimal,
                                pub redeemed_resources: [(ResourceAddress, Decimal); 2],
                            }


                            builder
                                .call_function(
                                    RESOURCE_PACKAGE,
                                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                                    NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                                        owner_role: OwnerRole::None,
                                        id_type: NonFungibleIdType::Integer,
                                        track_total_supply: true,
                                        non_fungible_schema: NonFungibleDataSchema::Remote(RemoteNonFungibleDataSchema {
                                            type_id: BlueprintTypeIdentifier {
                                                package_address: state.package_with_registered_types.unwrap(),
                                                blueprint_name: state.blueprint_with_registered_types.clone().unwrap(),
                                                type_name: "RemoveLiquidityEvent".to_owned(),
                                            },
                                            mutable_fields: index_set_new()
                                        }),
                                        resource_roles: NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                                        metadata: metadata! {},
                                        address_reservation: None,
                                        entries: indexmap!(
                                            NonFungibleLocalId::integer(1) => (manifest_decode(&manifest_encode(&RemoveLiquidityEvent {
                                                pool_units_amount: dec!(5),
                                                redeemed_resources: [(XRD, dec!(1)), (XRD, dec!(1))]
                                            }).unwrap()).unwrap(), ),
                                        ),
                                    }
                                )
                                .deposit_entire_worktop(config.main_account.address)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| {
                    state.non_fungible_resource_using_registered_type = Some(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add(
                            "package_with_registered_types",
                            state.package_with_registered_types.unwrap(),
                        )
                        .add(
                            "non_fungible_resource_using_registered_type",
                            state.non_fungible_resource_using_registered_type.unwrap(),
                        ),
                })
            })
    }
}
