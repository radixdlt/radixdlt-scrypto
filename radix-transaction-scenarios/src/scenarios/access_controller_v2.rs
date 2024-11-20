use crate::internal_prelude::*;
use crate::utils::*;
use radix_engine::updates::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::*;

pub struct AccessControllerV2ScenarioConfig {
    primary_role_private_key: PrivateKey,
    recovery_role_private_key: PrivateKey,
    confirmation_role_private_key: PrivateKey,
}

impl AccessControllerV2ScenarioConfig {
    pub fn access_rule(&self, selector: impl FnOnce(&Self) -> &PrivateKey) -> AccessRule {
        rule!(require(signature(&selector(self).public_key())))
    }

    pub fn rule_set(&self) -> RuleSet {
        RuleSet {
            primary_role: self.access_rule(|this| &this.primary_role_private_key),
            recovery_role: self.access_rule(|this| &this.recovery_role_private_key),
            confirmation_role: self.access_rule(|this| &this.confirmation_role_private_key),
        }
    }
}

#[derive(Default)]
pub struct AccessControllerV2ScenarioState {
    pub(crate) access_controller_component_address: State<ComponentAddress>,
}

impl Default for AccessControllerV2ScenarioConfig {
    fn default() -> Self {
        Self {
            primary_role_private_key: new_ed25519_private_key(1).into(),
            recovery_role_private_key: new_ed25519_private_key(2).into(),
            confirmation_role_private_key: new_ed25519_private_key(3).into(),
        }
    }
}

pub struct AccessControllerV2ScenarioCreator;

impl ScenarioCreator for AccessControllerV2ScenarioCreator {
    type Config = AccessControllerV2ScenarioConfig;
    type State = AccessControllerV2ScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "access-controller-v2",
        protocol_min_requirement: ProtocolVersion::Bottlenose,
        protocol_max_requirement: ProtocolVersion::LATEST,
        testnet_run_at: Some(ProtocolVersion::Bottlenose),
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
                |core, config, _| {
                    core.next_transaction_with_faucet_lock_fee(
                        "access-controller-v2-instantiate",
                        |builder| {
                            builder
                                .get_free_xrd_from_faucet()
                                .take_all_from_worktop(XRD, "bucket")
                                .with_bucket("bucket", |builder, bucket| {
                                    builder.call_function(
                                        ACCESS_CONTROLLER_PACKAGE,
                                        ACCESS_CONTROLLER_BLUEPRINT,
                                        ACCESS_CONTROLLER_CREATE_IDENT,
                                        AccessControllerCreateManifestInput {
                                            address_reservation: None,
                                            timed_recovery_delay_in_minutes: None,
                                            rule_set: config.rule_set(),
                                            controlled_asset: bucket,
                                        },
                                    )
                                })
                        },
                        vec![],
                    )
                },
                |_, _, state, result| {
                    state
                        .access_controller_component_address
                        .set(result.new_component_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction(|core, _, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "access-controller-v2-deposit-fees-xrd",
                    |builder| {
                        builder
                            .get_free_xrd_from_faucet()
                            .take_all_from_worktop(XRD, "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.access_controller_component_address.unwrap(),
                                    ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
                                    AccessControllerContributeRecoveryFeeManifestInput { bucket },
                                )
                            })
                    },
                    vec![],
                )
            })
            .successful_transaction(|core, config, state| {
                core.v1_transaction("access-controller-v2-lock-fee-and-recover")
                    .manifest(ManifestBuilder::new_v1()
                        .call_method(
                            state.access_controller_component_address.unwrap(),
                            ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT,
                            AccessControllerLockRecoveryFeeInput { amount: dec!(10) },
                        )
                        .call_method(
                            state.access_controller_component_address.unwrap(),
                            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
                            AccessControllerInitiateRecoveryAsPrimaryInput {
                                rule_set: config.rule_set(),
                                timed_recovery_delay_in_minutes: None,
                            },
                        )
                        .call_method(
                            state.access_controller_component_address.unwrap(),
                            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
                            AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
                                rule_set: config.rule_set(),
                                timed_recovery_delay_in_minutes: None,
                            },
                        )
                        .build()
                    )
                    .sign(&config.primary_role_private_key)
                    .sign(&config.recovery_role_private_key)
                    .complete(core)
            })
            .finalize(|_, _, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new().add(
                        "access_controller_v2_component_address",
                        state.access_controller_component_address.get()?,
                    ),
                })
            })
    }
}
