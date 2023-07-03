use crate::internal_prelude::*;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::account::ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT;
use radix_engine_interface::*;

pub struct FungibleResourceScenario {
    core: ScenarioCore,
    metadata: ScenarioMetadata,
    config: FungibleResourceScenarioConfig,
    state: FungibleResourceScenarioState,
}

pub struct FungibleResourceScenarioConfig {
    pub user_account_1: VirtualAccount,
    pub user_account_2: VirtualAccount,
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

impl ScenarioCreator for FungibleResourceScenario {
    type Config = FungibleResourceScenarioConfig;

    type State = FungibleResourceScenarioState;

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "fungible_resource",
        };
        Box::new(Self {
            core,
            metadata,
            config,
            state: start_state,
        })
    }
}

impl ScenarioInstance for FungibleResourceScenario {
    fn metadata(&self) -> &ScenarioMetadata {
        &self.metadata
    }

    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError> {
        let FungibleResourceScenarioConfig {
            user_account_1,
            user_account_2,
        } = &mut self.config;
        let FungibleResourceScenarioState {
            max_divisibility_fungible_resource,
            min_divisibility_fungible_resource,
            vault1,
            vault2,
        } = &mut self.state;
        let core = &mut self.core;

        let up_next = match core.next_stage() {
            1 => {
                core.check_start(&previous)?;
                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-create",
                    |builder| {
                        builder
                            .create_fungible_resource(
                                OwnerRole::None,
                                false,
                                18,
                                metadata! {},
                                btreemap! {
                                    Mint => (rule!(allow_all), rule!(deny_all)),
                                    Burn =>  (rule!(allow_all), rule!(deny_all)),
                                    UpdateNonFungibleData => (rule!(allow_all), rule!(deny_all)),
                                    Withdraw => (rule!(allow_all), rule!(deny_all)),
                                    Deposit => (rule!(allow_all), rule!(deny_all)),
                                    Recall => (rule!(allow_all), rule!(deny_all)),
                                    Freeze => (rule!(allow_all), rule!(deny_all)),
                                },
                                Some(dec!("100000")),
                            )
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![],
                )
            }
            2 => {
                let commit_success = core.check_commit_success(core.check_previous(&previous)?)?;
                *max_divisibility_fungible_resource =
                    Some(commit_success.new_resource_addresses()[0]);
                *vault1 = Some(commit_success.new_vault_addresses()[0]);

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-mint",
                    |builder| {
                        builder
                            .mint_fungible(max_divisibility_fungible_resource.unwrap(), dec!("100"))
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![],
                )
            }
            3 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-burn",
                    |builder| {
                        builder
                            .withdraw_from_account(
                                user_account_1.address,
                                max_divisibility_fungible_resource.unwrap(),
                                dec!("10"),
                            )
                            .take_all_from_worktop(
                                max_divisibility_fungible_resource.unwrap(),
                                |builder, bucket| builder.burn_resource(bucket),
                            )
                    },
                    vec![&user_account_1.key],
                )
            }
            4 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-transfer-32-times",
                    |builder| {
                        let mut builder = builder.withdraw_from_account(
                            user_account_1.address,
                            max_divisibility_fungible_resource.unwrap(),
                            dec!("10"),
                        );
                        for _ in 0..32 {
                            builder = builder.take_from_worktop(
                                max_divisibility_fungible_resource.unwrap(),
                                dec!("0.001"),
                                |builder, bucket| {
                                    builder.call_method(
                                        user_account_2.address,
                                        ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                                        manifest_args!(bucket),
                                    )
                                },
                            );
                        }
                        builder.try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            5 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-freeze-withdraw",
                    |builder| builder.freeze_withdraw(vault1.unwrap()),
                    vec![&user_account_1.key],
                )
            }
            6 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-freeze-deposit",
                    |builder| builder.freeze_deposit(vault1.unwrap()),
                    vec![&user_account_1.key],
                )
            }
            7 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-freeze-deposit",
                    |builder| builder.freeze_burn(vault1.unwrap()),
                    vec![&user_account_1.key],
                )
            }
            8 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-recall-freezed-vault",
                    |builder| {
                        builder
                            .recall(vault1.unwrap(), dec!("2"))
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            9 => {
                // FIXME: re-enable this after recalling from frozen vaults is allowed.
                // core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-unfreeze-withdraw",
                    |builder| builder.unfreeze_withdraw(vault1.unwrap()),
                    vec![&user_account_1.key],
                )
            }
            10 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-unfreeze-deposit",
                    |builder| builder.unfreeze_deposit(vault1.unwrap()),
                    vec![&user_account_1.key],
                )
            }
            11 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-unfreeze-deposit",
                    |builder| builder.unfreeze_burn(vault1.unwrap()),
                    vec![&user_account_1.key],
                )
            }
            12 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-max-div-recall-unfreezed-vault",
                    |builder| {
                        builder
                            .recall(vault1.unwrap(), dec!("2"))
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![&user_account_1.key],
                )
            }

            /* MIN DIVISIBILITY */
            13 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-create",
                    |builder| {
                        builder
                            .create_fungible_resource(
                                OwnerRole::None,
                                false,
                                0,
                                metadata! {},
                                btreemap! {
                                    Mint => (rule!(allow_all), rule!(deny_all)),
                                    Burn =>  (rule!(allow_all), rule!(deny_all)),
                                    UpdateNonFungibleData => (rule!(allow_all), rule!(deny_all)),
                                    Withdraw => (rule!(allow_all), rule!(deny_all)),
                                    Deposit => (rule!(allow_all), rule!(deny_all)),
                                    Recall => (rule!(allow_all), rule!(deny_all)),
                                    Freeze => (rule!(allow_all), rule!(deny_all)),
                                },
                                Some(dec!("100000")),
                            )
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![],
                )
            }
            14 => {
                let commit_success = core.check_commit_success(core.check_previous(&previous)?)?;
                *min_divisibility_fungible_resource =
                    Some(commit_success.new_resource_addresses()[0]);
                *vault2 = Some(commit_success.new_vault_addresses()[0]);

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-mint-correct-granularity",
                    |builder| {
                        builder
                            .mint_fungible(min_divisibility_fungible_resource.unwrap(), dec!("166"))
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![],
                )
            }
            15 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-mint-wrong-granularity",
                    |builder| {
                        builder
                            .mint_fungible(min_divisibility_fungible_resource.unwrap(), dec!("1.1"))
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![],
                )
            }
            16 => {
                core.check_commit_failure(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-transfer-correct-granularity",
                    |builder| {
                        builder
                            .withdraw_from_account(
                                user_account_1.address,
                                min_divisibility_fungible_resource.unwrap(),
                                dec!("234"),
                            )
                            .try_deposit_batch_or_abort(user_account_2.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            17 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-transfer-wrong-granularity",
                    |builder| {
                        builder
                            .withdraw_from_account(
                                user_account_1.address,
                                min_divisibility_fungible_resource.unwrap(),
                                dec!("0.0001"),
                            )
                            .try_deposit_batch_or_abort(user_account_2.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            18 => {
                core.check_commit_failure(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-create-proof-correct-granularity",
                    |builder| {
                        builder.create_proof_from_account_of_amount(
                            user_account_1.address,
                            min_divisibility_fungible_resource.unwrap(),
                            dec!("99"),
                        )
                    },
                    vec![&user_account_1.key],
                )
            }
            19 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-create-proof-wrong-granularity",
                    |builder| {
                        builder.create_proof_from_account_of_amount(
                            user_account_1.address,
                            min_divisibility_fungible_resource.unwrap(),
                            dec!("0.0001"),
                        )
                    },
                    vec![&user_account_1.key],
                )
            }
            20 => {
                core.check_commit_failure(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-recall-correct-granularity",
                    |builder| {
                        builder
                            .recall(vault2.unwrap(), dec!("2"))
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            21 => {
                core.check_commit_success(core.check_previous(&previous)?)?;

                core.next_transaction_with_faucet_lock_fee(
                    "nfr-min-div-recall-wrong-granularity",
                    |builder| {
                        builder
                            .recall(vault2.unwrap(), dec!("123.12321"))
                            .try_deposit_batch_or_abort(user_account_1.address)
                    },
                    vec![&user_account_1.key],
                )
            }
            _ => {
                core.check_commit_failure(core.check_previous(&previous)?)?;

                let output = ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("user_account_1", user_account_1.address.clone())
                        .add("user_account_2", user_account_2.address.clone())
                        .add(
                            "max_divisibility_fungible_resource",
                            max_divisibility_fungible_resource.unwrap(),
                        )
                        .add(
                            "min_divisibility_fungible_resource",
                            min_divisibility_fungible_resource.unwrap(),
                        ),
                };
                return Ok(NextAction::Completed(core.finish_scenario(output)));
            }
        };
        Ok(NextAction::Transaction(up_next?))
    }
}
