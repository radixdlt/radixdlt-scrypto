use crate::internal_prelude::*;
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::package::{PackageDefinition, PACKAGE_BLUEPRINT};

#[derive(Default)]
pub struct MaxTransactionScenarioState(Option<PackageAddress>, Option<ComponentAddress>);

pub struct MaxTransactionScenarioCreator;

impl ScenarioCreator for MaxTransactionScenarioCreator {
    type Config = MaxTransactionScenarioState;
    type State = MaxTransactionScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "max_transaction",
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
                |core, state, _| {
                    let code = include_bytes!("../../assets/max_transaction.wasm").to_vec();
                    let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                        "../../assets/max_transaction.rpd"
                    ))
                    .unwrap();

                    core.next_transaction_with_faucet_lock_fee(
                        "max_transaction-publish-package",
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
                                            "MaxTransaction",
                                            "new",
                                            manifest_args!(),
                                        )
                                })
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.0 = Some(result.new_package_addresses()[0]);
                    state.1 = Some(result.new_component_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "max_transaction-with-large-events",
                    |builder| {
                        builder.call_function(
                            state.0.unwrap(),
                            "MaxTransaction",
                            "max_events",
                            manifest_args!(255u32), // 1 for lock fee
                        )
                    },
                    vec![],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "max_transaction-with-large-state-updates",
                    |builder| {
                        builder.call_method(
                            state.1.unwrap(),
                            "max_state_updates",
                            manifest_args!(21u32),
                        )
                    },
                    vec![],
                )
            })
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("component_with_large_state", state.1.unwrap()),
                })
            })
    }
}
