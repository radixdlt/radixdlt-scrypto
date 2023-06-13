use crate::internal_prelude::*;

pub struct TransferXrdScenario {
    core: ScenarioCore,
    config: TransferXrdConfig,
}

pub struct TransferXrdConfig {
    pub from_account: VirtualAccount,
    pub to_account_1: VirtualAccount,
    pub to_account_2: VirtualAccount,
}

impl Default for TransferXrdConfig {
    fn default() -> Self {
        Self {
            from_account: secp256k1_account_1(),
            to_account_1: secp256k1_account_2(),
            to_account_2: ed25519_account_3(),
        }
    }
}

impl ScenarioDefinition for TransferXrdScenario {
    type Config = TransferXrdConfig;

    fn new_with_config(core: ScenarioCore, config: Self::Config) -> Self {
        Self { core, config }
    }
}

impl ScenarioInstance for TransferXrdScenario {
    fn metadata(&self) -> ScenarioMetadata {
        ScenarioMetadata {
            logical_name: "transfer_xrd",
        }
    }

    fn next(&mut self, previous: Option<&TransactionReceipt>) -> Result<NextAction, ScenarioError> {
        // Destructure config for beauty
        let TransferXrdConfig {
            from_account,
            to_account_1,
            to_account_2,
        } = &self.config;
        let core = &mut self.core;

        // Handle the previous result, return the next result
        let up_next = match core.next_stage() {
            1 => {
                core.check_start(&previous)?;
                core.next_transaction_free_xrd_from_faucet(from_account.address)
            }
            2 => {
                core.check_commit_success(&previous)?;
                core.next_transaction_with_faucet_lock_fee(
                    "transfer--try_deposit_batch_or_abort",
                    |builder| {
                        builder
                            .withdraw_from_account(from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_abort(to_account_1.address)
                    },
                    vec![&from_account.key],
                )
            }
            3 => {
                core.check_commit_success(&previous)?;
                core.next_transaction_with_faucet_lock_fee(
                    "transfer--try_deposit_batch_or_refund",
                    |builder| {
                        builder
                            .withdraw_from_account(from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_refund(to_account_1.address)
                    },
                    vec![&from_account.key],
                )
            }
            4 => {
                core.check_commit_success(&previous)?;
                core.next_transaction_with_faucet_lock_fee(
                    "self-transfer--deposit_batch",
                    |builder| {
                        builder
                            .withdraw_from_account(from_account.address, XRD, dec!(1))
                            .deposit_batch(from_account.address)
                    },
                    vec![&from_account.key],
                )
            }
            5 => {
                core.check_commit_success(&previous)?;
                core.next_transaction_with_faucet_lock_fee(
                    "multi-transfer--deposit_batch",
                    |builder| {
                        builder
                            .withdraw_from_account(from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_abort(to_account_1.address)
                            .withdraw_from_account(from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_abort(to_account_2.address)
                    },
                    vec![&from_account.key],
                )
            }
            _ => {
                core.check_commit_success(&previous)?;
                return Ok(core.finish_scenario());
            }
        };
        Ok(NextAction::Transaction(up_next))
    }
}
