use crate::internal_prelude::*;
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::package::{PackageDefinition, PACKAGE_BLUEPRINT};

#[derive(Default)]
pub struct KVStoreScenarioState(Option<(PackageAddress, ComponentAddress)>);

pub struct KVStoreScenarioCreator;

impl ScenarioCreator for KVStoreScenarioCreator {
    type Config = ();
    type State = KVStoreScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "kv_store_with_remote_type",
        protocol_min_requirement: ProtocolVersion::Babylon,
        protocol_max_requirement: ProtocolVersion::LATEST,
        testnet_run_at: Some(ProtocolVersion::Babylon),
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
                |core, state, _| {
                    let code = include_bytes!("../../assets/kv_store.wasm");
                    let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                        "../../assets/kv_store.rpd"
                    ))
                    .unwrap();

                    core.next_transaction_with_faucet_lock_fee(
                        "kv-store-with-remote-type",
                        |builder| {
                            builder
                                .allocate_global_address(
                                    PACKAGE_PACKAGE,
                                    PACKAGE_BLUEPRINT,
                                    "package_address_reservation",
                                    "package_address",
                                )
                                .with_name_lookup(|builder, namer| {
                                    let package_address = namer.named_address("package_address");
                                    builder
                                        .publish_package_advanced(
                                            "package_address_reservation",
                                            code.to_vec(),
                                            schema,
                                            MetadataInit::default(),
                                            OwnerRole::None,
                                        )
                                        .call_function(
                                            package_address,
                                            "KVStore",
                                            "create_key_value_store_with_remote_type",
                                            manifest_args!(package_address, "KVStore", "TestType"),
                                        )
                                })
                        },
                        vec![],
                    )
                },
                |_, _, state, result| {
                    let component_address = result.new_component_addresses()[0];
                    let package_address = result.new_package_addresses()[0];
                    state.0 = Some((package_address, component_address));
                    Ok(())
                },
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add(
                            "kv_store_with_remote_type_package_address",
                            state.0.unwrap().0,
                        )
                        .add(
                            "kv_store_with_remote_type_component_address",
                            state.0.unwrap().1,
                        ),
                })
            })
    }
}
