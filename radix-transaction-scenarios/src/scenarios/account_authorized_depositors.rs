use crate::internal_prelude::*;
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::blueprints::account::*;

#[allow(deprecated)]
pub struct AccountAuthorizedDepositorsScenarioConfig {
    pub source_account: PreallocatedAccount,
    pub destination_account: PreallocatedAccount,
}

#[derive(Default)]
pub struct AccountAuthorizedDepositorsScenarioState {
    pub badge: Option<ResourceAddress>,
}

impl Default for AccountAuthorizedDepositorsScenarioConfig {
    fn default() -> Self {
        Self {
            source_account: secp256k1_account_1(),
            destination_account: secp256k1_account_2(),
        }
    }
}

pub struct AccountAuthorizedDepositorsScenarioCreator;

impl ScenarioCreator for AccountAuthorizedDepositorsScenarioCreator {
    type Config = AccountAuthorizedDepositorsScenarioConfig;
    type State = AccountAuthorizedDepositorsScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "account_authorized_depositors",
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
                |core, config, _| {
                    let AccountAuthorizedDepositorsScenarioConfig {
                        source_account,
                        destination_account,
                    } = &config;

                    core.next_transaction_with_faucet_lock_fee(
                        "account-authorized-depositors-configure-accounts",
                        |builder| {
                            builder
                                .call_method(
                                    source_account.address,
                                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                                    AccountSetDefaultDepositRuleInput {
                                        default: DefaultDepositRule::Reject,
                                    },
                                )
                                .call_method(
                                    destination_account.address,
                                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                                    AccountSetDefaultDepositRuleInput {
                                        default: DefaultDepositRule::Reject,
                                    },
                                )
                                .allocate_global_address(
                                    RESOURCE_PACKAGE,
                                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                                    "address_reservation",
                                    "address",
                                )
                                .then(|builder| {
                                    let address_reservation =
                                        builder.address_reservation("address_reservation");
                                    builder.call_function(
                                        RESOURCE_PACKAGE,
                                        FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                                        FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                                        FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                                            owner_role: OwnerRole::None,
                                            track_total_supply: true,
                                            divisibility: 18,
                                            initial_supply: 1.into(),
                                            resource_roles: Default::default(),
                                            metadata: Default::default(),
                                            address_reservation: Some(address_reservation),
                                        },
                                    )
                                })
                                .with_name_lookup(|builder, lookup| {
                                    builder.call_method(
                                        destination_account.address,
                                        ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                                        AccountAddAuthorizedDepositorManifestInput {
                                            badge: ManifestResourceOrNonFungible::Resource(lookup.named_address("address").into()),
                                        },
                                    )
                                })
                                .deposit_entire_worktop(source_account.address)
                        },
                        vec![
                            &source_account.key,
                            &destination_account.key
                        ],
                    )
                },
                |_, _, state, result| {
                    let resource_address = result.new_resource_addresses()[0];
                    state.badge = Some(resource_address);
                    Ok(())
                },
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-authorized-depositors-attempt-deposit-success",
                    |builder| {
                        let badge_resource_address = state.badge.unwrap();
                        let badge = ResourceOrNonFungible::Resource(badge_resource_address);
                        builder
                            .create_proof_from_account_of_amount(config.source_account.address, badge_resource_address, 1)
                            .get_free_xrd_from_faucet()
                            .take_all_from_worktop(XRD, "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    config.destination_account.address,
                                    ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                                    AccountTryDepositOrRefundManifestInput {
                                        authorized_depositor_badge: Some(badge.into()),
                                        bucket
                                    }
                                )
                            })
                    },
                    vec![
                        &config.source_account.key
                    ],
                )
            })
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "account-authorized-depositors-attempt-deposit-failure-if-badge-is-not-present",
                        |builder| {
                            let badge_resource_address = state.badge.unwrap();
                            let badge = ResourceOrNonFungible::Resource(badge_resource_address);
                            builder
                                .get_free_xrd_from_faucet()
                                .take_all_from_worktop(XRD, "bucket")
                                .with_bucket("bucket", |builder, bucket| {
                                    builder.call_method(
                                        config.destination_account.address,
                                        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                                        AccountTryDepositOrRefundManifestInput {
                                            authorized_depositor_badge: Some(badge.into()),
                                            bucket
                                        }
                                    )
                                })
                        },
                        vec![
                            &config.source_account.key
                        ],
                    )
                },
                |_, _, _, _| Ok(()),
            )
            .failed_transaction_with_error_handler(
                |core, config, _| {
                    core.next_transaction_with_faucet_lock_fee(
                        "account-authorized-depositors-attempt-deposit-failure-if-badge-is-not-an-authorized-depositor",
                        |builder| {
                            builder
                                .get_free_xrd_from_faucet()
                                .take_all_from_worktop(XRD, "bucket")
                                .with_bucket("bucket", |builder, bucket| {
                                    builder.call_method(
                                        config.destination_account.address,
                                        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                                        AccountTryDepositOrRefundManifestInput {
                                            authorized_depositor_badge: Some(ResourceOrNonFungible::Resource(ACCOUNT_OWNER_BADGE).into()),
                                            bucket
                                        }
                                    )
                                })
                        },
                        vec![
                            &config.source_account.key
                        ],
                    )
                },
                |_, _, _, _| Ok(()),
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("source_account", config.source_account.address)
                        .add("destination_account", config.destination_account.address)
                        .add("authorized_deposit_badge", state.badge.unwrap()),
                })
            })
    }
}
