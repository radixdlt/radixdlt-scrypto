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
    child_token: State<ResourceAddress>,
    repeated_partial: State<DetailedSignedPartialTransactionV2>,
}

pub struct BasicSubintentsScenarioCreator;

impl ScenarioCreator for BasicSubintentsScenarioCreator {
    type Config = BasicSubintentsScenarioConfig;
    type State = BasicSubintentsScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "basic_subintents",
        protocol_min_requirement: ProtocolVersion::CuttlefishPart1,
        // This scenario requires exactly `CuttlefishPart1` to run, because it accidentally uses
        // the fact that `VERIFY_PARENT` verified the root transaction intent rather than the
        // direct parent intent in `CuttlefishPart1`. This was fixed in `CuttlefishPart2`.
        protocol_max_requirement: ProtocolVersion::CuttlefishPart1,
        testnet_run_at: Some(ProtocolVersion::CuttlefishPart1),
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
                let resource_addresses = result.new_resource_addresses();
                state.child_token.set(resource_addresses[0]);

                // We prepare a partial at this point for later
                let repeated_partial = core
                    .v2_subintent()
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder
                            .withdraw_from_account(
                                state.child_account.unwrap(),
                                state.child_token.unwrap(),
                                50,
                            )
                            .take_all_from_worktop(
                                state.child_token.unwrap(),
                                "free-gift-single-use",
                            )
                            .yield_to_parent((lookup.bucket("free-gift-single-use"),))
                    })
                    .sign(&config.child_account_key)
                    .complete(core);
                state.repeated_partial.set(repeated_partial);
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
                            .create_fungible_resource(
                                OwnerRole::Fixed(rule!(require(
                                    config.child_account_key.public_key().signature_proof()
                                ))),
                                true,
                                18,
                                FungibleResourceRoles::default_with_owner_mint_burn(),
                                metadata!(
                                    init {
                                        "name" => "Example Scenario Tokens", locked;
                                    }
                                ),
                                Some(dec!(1000)),
                            )
                            .allocate_global_address(
                                ACCOUNT_PACKAGE,
                                ACCOUNT_BLUEPRINT,
                                "child_account",
                                "child_account",
                            )
                            .create_account_with_owner(
                                "child_account",
                                OwnerRole::Fixed(rule!(require(
                                    config.child_account_key.public_key().signature_proof()
                                ))),
                            )
                            .try_deposit_entire_worktop_or_abort("child_account", None)
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
                    Some(Instant::new(0)),
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
            .successful_transaction(|core, config, state| {
                let trading_subintent = core
                    .v2_subintent()
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder
                            .withdraw_from_account(
                                state.child_account.unwrap(),
                                state.child_token.unwrap(),
                                10,
                            )
                            .take_all_from_worktop(state.child_token.unwrap(), "sold_to_parent")
                            .assert_next_call_returns_only(
                                ManifestResourceConstraints::new()
                                    .with_at_least_amount(XRD, dec!(23.32)),
                            )
                            .yield_to_parent((lookup.bucket("sold_to_parent"),))
                            .take_all_from_worktop(XRD, "bought_from_parent")
                            .deposit(state.child_account.unwrap(), "bought_from_parent")
                            .yield_to_parent(())
                    })
                    .sign(&config.child_account_key)
                    .complete(core);

                core.v2_transaction("trading_with_subintent")
                    .add_signed_child("trading_subintent", trading_subintent)
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder
                            .lock_standard_test_fee(state.parent_account.unwrap())
                            .yield_to_child("trading_subintent", ())
                            .take_all_from_worktop(state.child_token.unwrap(), "bought")
                            .deposit(state.parent_account.unwrap(), "bought")
                            .withdraw_from_account(state.parent_account.unwrap(), XRD, dec!(23.5))
                            .take_all_from_worktop(XRD, "sold")
                            .yield_to_child("trading_subintent", (lookup.bucket("sold"),))
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                let child_sending_resources: DetailedSignedPartialTransactionV2 = core
                    .v2_subintent()
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder
                            .verify_parent(rule!(require(
                                config.parent_account_key.public_key().signature_proof()
                            )))
                            .withdraw_from_account(
                                state.child_account.unwrap(),
                                state.child_token.unwrap(),
                                10,
                            )
                            .take_all_from_worktop(state.child_token.unwrap(), "given_to_parent")
                            .assert_next_call_returns_no_resources()
                            .yield_to_parent((lookup.bucket("given_to_parent"),))
                    })
                    .sign(&config.child_account_key)
                    .complete(core);

                let child_bouncing_resources = core
                    .v2_subintent()
                    .manifest_builder(|builder| {
                        builder
                            .yield_to_parent(())
                            .yield_to_parent((ManifestExpression::EntireWorktop,))
                    })
                    .complete(core);

                let complex_subintent = core
                    .v2_subintent()
                    .add_signed_child(
                        "child_sending_resources_to_verified_parent",
                        child_sending_resources,
                    )
                    .add_signed_child("child_bouncing_resources", child_bouncing_resources)
                    .manifest_builder_with_lookup(|builder, lookup| {
                        builder
                            .assert_worktop_is_empty()
                            .assert_next_call_returns_include(
                                ManifestResourceConstraints::new()
                                    .with_at_least_amount(state.child_token.unwrap(), 5),
                            )
                            .yield_to_child("child_sending_resources_to_verified_parent", ())
                            .assert_worktop_resources_include(
                                ManifestResourceConstraints::new()
                                    .with_exact_amount(state.child_token.unwrap(), 10),
                            )
                            .take_all_from_worktop(state.child_token.unwrap(), "bucket")
                            .assert_bucket_contents(
                                "bucket",
                                ManifestResourceConstraint::NonZeroAmount,
                            )
                            .assert_next_call_returns_no_resources()
                            .yield_to_child("child_bouncing_resources", (lookup.bucket("bucket"),))
                            .assert_next_call_returns_only(
                                ManifestResourceConstraints::new().with_amount_range(
                                    state.child_token.unwrap(),
                                    5,
                                    100,
                                ),
                            )
                            .yield_to_child("child_bouncing_resources", ())
                            .take_from_worktop(state.child_token.unwrap(), 10, "final_bucket")
                            .yield_to_parent((lookup.bucket("final_bucket"),))
                    })
                    .complete(core);
                core.v2_transaction("transaction_with_complex_subintent")
                    .add_signed_child("complex_subintent", complex_subintent)
                    .manifest_builder(|builder| {
                        builder
                            .lock_standard_test_fee(state.parent_account.unwrap())
                            .yield_to_child("complex_subintent", ())
                            .assert_worktop_resources_only(
                                ManifestResourceConstraints::new()
                                    .with_exact_amount(state.child_token.unwrap(), 10),
                            )
                            .deposit_entire_worktop(state.parent_account.unwrap())
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .failed_transaction(|core, config, state| {
                core.v2_transaction("first_transaction_with_subintent_which_fails")
                    .add_signed_child("subintent-with-free-gift", state.repeated_partial.unwrap())
                    .manifest_builder(|builder| {
                        builder
                            .lock_standard_test_fee(state.parent_account.unwrap())
                            .yield_to_child("subintent-with-free-gift", ())
                            .assert_worktop_contains(XRD, 1) // Fail the transaction
                            .deposit_entire_worktop(state.parent_account.unwrap())
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .successful_transaction(|core, config, state| {
                core.v2_transaction("second_transaction_with_subintent_can_succeed")
                    .add_signed_child(
                        "repeated-subintent-with-free-gift",
                        state.repeated_partial.unwrap(),
                    )
                    .manifest_builder(|builder| {
                        builder
                            .lock_standard_test_fee(state.parent_account.unwrap())
                            .yield_to_child("repeated-subintent-with-free-gift", ())
                            .deposit_entire_worktop(state.parent_account.unwrap())
                    })
                    .sign(&config.parent_account_key)
                    .complete(core)
            })
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("parent_account", state.parent_account.get()?)
                        .add("child_account", state.child_account.get()?)
                        .add("child_token", state.child_token.get()?),
                })
            })
    }
}
