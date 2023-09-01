use crate::internal_prelude::*;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::{PackageDefinition, PACKAGE_BLUEPRINT};
use radix_engine_interface::*;

#[derive(Default)]
pub struct InnerNOuterScenarioState(Option<(PackageAddress, ComponentAddress)>);

pub struct InnerNOuterScenarioCreator;

impl ScenarioCreator for InnerNOuterScenarioCreator {
    type Config = ();
    type State = InnerNOuterScenarioState;

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "inner_n_outer",
        };

        #[allow(unused_variables)]
        ScenarioBuilder::new(core, metadata, config, start_state)
            .successful_transaction_with_result_handler(
                |core, state, _| {
                    let code = include_bytes!("../../../assets/inner_n_outer.wasm");
                    let schema = manifest_decode::<PackageDefinition>(include_bytes!(
                        "../../../assets/inner_n_outer.rpd"
                    ))
                    .unwrap();

                    core.next_transaction_with_faucet_lock_fee(
                        "inner_n_outer_emitting_events",
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
                                            Some("package_address_reservation".to_owned()),
                                            code.to_vec(),
                                            schema,
                                            MetadataInit::default(),
                                            OwnerRole::None,
                                        )
                                        .call_function(
                                            package_address,
                                            "Outer",
                                            "new",
                                            manifest_args!(),
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
                        .add("inner_n_outer_package_address", state.0.unwrap().0)
                        .add("inner_n_outer_component_address", state.0.unwrap().1),
                })
            })
    }
}
