use crate::internal_prelude::*;
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

#[allow(deprecated)]
pub struct FungibleResourceScenarioConfig {
    pub user_account_1: PreallocatedAccount,
    pub user_account_2: PreallocatedAccount,
}

#[derive(Default)]
pub struct FungibleResourceScenarioState {
    pub max_divisibility_fungible_resource: Option<ResourceAddress>,
    pub min_divisibility_fungible_resource: Option<ResourceAddress>,
    pub vault1: Option<InternalAddress>,
    pub vault2: Option<InternalAddress>,
}

impl Default for FungibleResourceScenarioConfig {
    fn default() -> Self {
        Self {
            user_account_1: secp256k1_account_1(),
            user_account_2: secp256k1_account_2(),
        }
    }
}

pub struct FungibleResourceScenarioCreator;

impl ScenarioCreator for FungibleResourceScenarioCreator {
    type Config = FungibleResourceScenarioConfig;
    type State = FungibleResourceScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "fungible_resource",
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
                        "fungible-max-div-create",
                        |builder| {
                            builder
                                .create_fungible_resource(
                                    OwnerRole::None,
                                    false,
                                    18,
                                    FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                                    metadata!(),
                                    Some(dec!("100000")),
                                )
                                .try_deposit_entire_worktop_or_abort(
                                    config.user_account_1.address,
                                    None,
                                )
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.max_divisibility_fungible_resource =
                        Some(result.new_resource_addresses()[0]);
                    state.vault1 = Some(result.new_vault_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-mint",
                    |builder| {
                        builder
                            .mint_fungible(
                                state.max_divisibility_fungible_resource.unwrap(),
                                dec!("100"),
                            )
                            .try_deposit_entire_worktop_or_abort(
                                config.user_account_1.address,
                                None,
                            )
                    },
                    vec![],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-burn",
                    |builder| {
                        builder
                            .withdraw_from_account(
                                config.user_account_1.address,
                                state.max_divisibility_fungible_resource.unwrap(),
                                dec!("10"),
                            )
                            .burn_all_from_worktop(
                                state.max_divisibility_fungible_resource.unwrap(),
                            )
                    },
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-transfer-32-times",
                    |builder| {
                        let mut builder = builder.withdraw_from_account(
                            config.user_account_1.address,
                            state.max_divisibility_fungible_resource.unwrap(),
                            dec!("10"),
                        );
                        for _ in 0..32 {
                            let bucket = builder.generate_bucket_name("transfer");
                            builder = builder
                                .take_from_worktop(
                                    state.max_divisibility_fungible_resource.unwrap(),
                                    dec!("0.001"),
                                    &bucket,
                                )
                                .try_deposit_or_abort(config.user_account_2.address, None, bucket);
                        }
                        builder.try_deposit_entire_worktop_or_abort(
                            config.user_account_1.address,
                            None,
                        )
                    },
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-freeze-withdraw",
                    |builder| builder.freeze_withdraw(state.vault1.unwrap()),
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-freeze-deposit",
                    |builder| builder.freeze_deposit(state.vault1.unwrap()),
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-freeze-burn",
                    |builder| builder.freeze_burn(state.vault1.unwrap()),
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-recall-frozen-vault",
                    |builder| {
                        builder
                            .recall(state.vault1.unwrap(), dec!("1"))
                            .try_deposit_entire_worktop_or_abort(
                                config.user_account_2.address,
                                None,
                            )
                    },
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-unfreeze-withdraw",
                    |builder| builder.unfreeze_withdraw(state.vault1.unwrap()),
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-unfreeze-deposit",
                    |builder| builder.unfreeze_deposit(state.vault1.unwrap()),
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-unfreeze-burn",
                    |builder| builder.unfreeze_burn(state.vault1.unwrap()),
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-recall-unfrozen-vault",
                    |builder| {
                        builder
                            .recall(state.vault1.unwrap(), dec!("2"))
                            .try_deposit_entire_worktop_or_abort(
                                config.user_account_1.address,
                                None,
                            )
                    },
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-max-div-freeze-withdraw-again",
                    |builder| builder.freeze_withdraw(state.vault1.unwrap()),
                    vec![&config.user_account_1.key],
                )
            })
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "fungible-min-div-create",
                        |builder| {
                            builder
                                .create_fungible_resource(
                                    OwnerRole::None,
                                    false,
                                    0,
                                    FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                                    metadata!(),
                                    Some(dec!("100000")),
                                )
                                .try_deposit_entire_worktop_or_abort(
                                    config.user_account_1.address,
                                    None,
                                )
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.min_divisibility_fungible_resource =
                        Some(result.new_resource_addresses()[0]);
                    state.vault2 = Some(result.new_vault_addresses()[0]);

                    Ok(())
                },
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-min-div-mint-correct-granularity",
                    |builder| {
                        builder
                            .mint_fungible(
                                state.min_divisibility_fungible_resource.unwrap(),
                                dec!("166"),
                            )
                            .try_deposit_entire_worktop_or_abort(
                                config.user_account_1.address,
                                None,
                            )
                    },
                    vec![],
                )
            })
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "fungible-min-div-mint-wrong-granularity",
                        |builder| {
                            builder
                                .mint_fungible(
                                    state.min_divisibility_fungible_resource.unwrap(),
                                    dec!("1.1"),
                                )
                                .try_deposit_entire_worktop_or_abort(
                                    config.user_account_1.address,
                                    None,
                                )
                        },
                        vec![],
                    )
                },
                |core, config, state, error| Ok(()),
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-min-div-transfer-correct-granularity",
                    |builder| {
                        builder
                            .withdraw_from_account(
                                config.user_account_1.address,
                                state.min_divisibility_fungible_resource.unwrap(),
                                dec!("234"),
                            )
                            .try_deposit_entire_worktop_or_abort(
                                config.user_account_2.address,
                                None,
                            )
                    },
                    vec![&config.user_account_1.key],
                )
            })
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "fungible-min-div-transfer-wrong-granularity",
                        |builder| {
                            builder
                                .withdraw_from_account(
                                    config.user_account_1.address,
                                    state.min_divisibility_fungible_resource.unwrap(),
                                    dec!("0.0001"),
                                )
                                .try_deposit_entire_worktop_or_abort(
                                    config.user_account_2.address,
                                    None,
                                )
                        },
                        vec![&config.user_account_1.key],
                    )
                },
                |core, config, state, error| Ok(()),
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-min-div-create-proof-correct-granularity",
                    |builder| {
                        builder.create_proof_from_account_of_amount(
                            config.user_account_1.address,
                            state.min_divisibility_fungible_resource.unwrap(),
                            dec!("99"),
                        )
                    },
                    vec![&config.user_account_1.key],
                )
            })
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "fungible-min-div-create-proof-wrong-granularity",
                        |builder| {
                            builder.create_proof_from_account_of_amount(
                                config.user_account_1.address,
                                state.min_divisibility_fungible_resource.unwrap(),
                                dec!("0.0001"),
                            )
                        },
                        vec![&config.user_account_1.key],
                    )
                },
                |core, config, state, error| Ok(()),
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "fungible-min-div-recall-correct-granularity",
                    |builder| {
                        builder
                            .recall(state.vault2.unwrap(), dec!("2"))
                            .try_deposit_entire_worktop_or_abort(
                                config.user_account_1.address,
                                None,
                            )
                    },
                    vec![&config.user_account_1.key],
                )
            })
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "fungible-min-div-recall-wrong-granularity",
                        |builder| {
                            builder
                                .recall(state.vault2.unwrap(), dec!("123.12321"))
                                .try_deposit_entire_worktop_or_abort(
                                    config.user_account_1.address,
                                    None,
                                )
                        },
                        vec![&config.user_account_1.key],
                    )
                },
                |core, config, state, error| Ok(()),
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("user_account_1", config.user_account_1.address)
                        .add("user_account_2", config.user_account_2.address)
                        .add(
                            "max_divisibility_fungible_resource",
                            state.max_divisibility_fungible_resource.unwrap(),
                        )
                        .add(
                            "min_divisibility_fungible_resource",
                            state.min_divisibility_fungible_resource.unwrap(),
                        )
                        .add("fungible_vault", state.vault1.unwrap()),
                })
            })
    }
}
