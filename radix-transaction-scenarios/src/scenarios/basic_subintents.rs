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
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("parent_account", state.parent_account.get()?)
                        .add("child_account", state.child_account.get()?),
                })
            })
    }
}
