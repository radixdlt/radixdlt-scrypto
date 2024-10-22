use crate::internal_prelude::*;
use crate::utils::{new_ed25519_private_key, new_secp256k1_private_key};
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::*;

pub struct BasicSubintentsScenarioConfig {
    pub parent_account_key: PrivateKey,
    pub child_account_key: PrivateKey,
}

impl Default for BasicSubintentsScenarioConfig {
    fn default() -> Self {
        Self {
            parent_account_key: new_ed25519_private_key(3).into(),
            child_account_key: new_secp256k1_private_key(1).into(),
        }
    }
}

#[derive(Default)]
pub struct BasicSubintentsScenarioState {
    parent_account: State<ComponentAddress>,
    child_account: State<ComponentAddress>,
}

pub struct BasicSubintentsScenarioCreator;

impl ScenarioCreator for BasicSubintentsScenarioCreator {
    type Config = BasicSubintentsScenarioConfig;
    type State = BasicSubintentsScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "basic_subintents",
        protocol_min_requirement: ProtocolVersion::Cuttlefish,
        testnet_run_at: Some(ProtocolVersion::Cuttlefish),
        safe_to_run_on_used_ledger: true,
    };

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance {
        #[allow(unused_variables)]
        ScenarioBuilder::new(core, Self::METADATA, config, start_state)
            .on_next_transaction_commit(|core, config, state, result| {
                let component_addresses = result.new_component_addresses();
                state.parent_account.set(component_addresses[0]);
                state.child_account.set(component_addresses[1]);
                Ok(())
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("create-accounts")
                    .manifest_builder(|builder| {
                        builder
                            .lock_fee_from_faucet()
                            .allocate_global_address(
                                ACCOUNT_PACKAGE,
                                ACCOUNT_BLUEPRINT,
                                "parent_account",
                                "parent_account",
                            )
                            .create_account_with_owner(
                                "parent_account",
                                OwnerRole::Fixed(rule!(require(
                                    config.parent_account_key.public_key().signature_proof()
                                ))),
                            )
                            .get_free_xrd_from_faucet()
                            .take_all_from_worktop(XRD, "free_xrd")
                            .deposit("parent_account", "free_xrd")
                            .create_account_with_owner(
                                None,
                                OwnerRole::Fixed(rule!(require(
                                    config.child_account_key.public_key().signature_proof()
                                ))),
                            )
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                let trivial_child = core
                    .v2_subintent()
                    .manifest_builder(|builder| builder.yield_to_parent(()))
                    .complete(core);
                core.v2_transaction("trivial_subintent")
                    .add_signed_child("trivial_child", trivial_child)
                    .manifest_builder(|builder| {
                        builder
                            .lock_standard_test_fee(state.parent_account.unwrap())
                            .yield_to_child("trivial_child", ())
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction_with_timestamp_range(
                    "with_timestamp_range",
                    None,
                    Some(Instant::new(i64::MAX)),
                )
                .manifest_builder(|builder| {
                    builder.lock_standard_test_fee(state.parent_account.unwrap())
                })
                .sign(&config.parent_account_key)
                .complete(core)
            })
            .successful_transaction(|core, config, state| {
                // We verify depth five fails in the subintent_structure.rs tests
                let depth_four_child = core
                    .v2_subintent()
                    .manifest_builder(|builder| builder.yield_to_parent(()))
                    .complete(core);

                let depth_three_child = core
                    .v2_subintent()
                    .add_signed_child("depth_four_child", depth_four_child)
                    .manifest_builder(|builder| {
                        builder
                            .yield_to_child("depth_four_child", ())
                            .yield_to_parent(())
                    })
                    .complete(core);

                let depth_two_child = core
                    .v2_subintent()
                    .add_signed_child("depth_three_child", depth_three_child)
                    .manifest_builder(|builder| {
                        builder
                            .yield_to_child("depth_three_child", ())
                            .yield_to_parent(())
                    })
                    .complete(core);

                // Root transaction intent is depth 1
                core.v2_transaction("depth_four_transaction")
                    .add_signed_child("depth_two_child", depth_two_child)
                    .manifest_builder(|builder| {
                        builder
                            .lock_standard_test_fee(state.parent_account.unwrap())
                            .yield_to_child("depth_two_child", ())
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .failed_transaction(|core, config, state| {
                let trivial_child1 = core
                    .v2_subintent()
                    .manifest_builder(|builder| builder.yield_to_parent(()))
                    .complete(core);
                let trivial_child2 = core
                    .v2_subintent()
                    .manifest_builder(|builder| builder.yield_to_parent(()))
                    .complete(core);

                let complex_subintent = core
                    .v2_subintent()
                    .add_signed_child("trivial_child1", trivial_child1)
                    .add_signed_child("trivial_child2", trivial_child2)
                    .manifest_builder(|builder| {
                        builder
                            .assert_worktop_resources_only(
                                ManifestResourceConstraints::new().with_at_least_non_fungibles(
                                    GLOBAL_CALLER_RESOURCE,
                                    [NonFungibleLocalId::integer(1)],
                                ),
                            )
                            .assert_worktop_resources_include(
                                ManifestResourceConstraints::new().with_exact_amount(XRD, 100),
                            )
                            .take_all_from_worktop(XRD, "xrd_bucket")
                            .assert_bucket_contents(
                                "xrd_bucket",
                                ManifestResourceConstraint::NonZeroAmount,
                            )
                            .assert_next_call_returns_include(
                                ManifestResourceConstraints::new().with_exact_non_fungibles(
                                    GLOBAL_CALLER_RESOURCE,
                                    [NonFungibleLocalId::integer(1)],
                                ),
                            )
                            .yield_to_child_with_name_lookup("trivial_child1", |lookup| {
                                (lookup.bucket("xrd_bucket"),)
                            })
                            .assert_next_call_returns_only(
                                ManifestResourceConstraints::new().with_amount_range(XRD, 0, 100),
                            )
                            .yield_to_child("trivial_child2", ())
                            .verify_parent(rule!(require(XRD)))
                            .yield_to_parent(())
                    })
                    .complete(core);
                core.v2_transaction("transaction_with_complex_subintent")
                    .add_signed_child("complex_subintent", complex_subintent)
                    .manifest_builder(|builder| {
                        builder
                            .lock_standard_test_fee(state.parent_account.unwrap())
                            .yield_to_child("complex_subintent", ())
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("parent_account", state.parent_account.get()?)
                        .add("child_account", state.child_account.get()?),
                })
            })
    }
}
