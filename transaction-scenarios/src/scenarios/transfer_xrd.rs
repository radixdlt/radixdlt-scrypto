use crate::internal_prelude::*;

pub struct TransferXrdScenario {
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

impl Scenario for TransferXrdScenario {
    type Config = TransferXrdConfig;

    fn new_with_config(config: Self::Config) -> Self {
        Self { config }
    }
}

impl ScenarioCore for TransferXrdScenario {
    fn logical_name(&self) -> &'static str {
        "transfer_xrd"
    }

    fn next(
        &mut self,
        context: &mut ScenarioContext,
        previous: Option<&TransactionReceipt>,
    ) -> Result<Option<NextTransaction>, ScenarioError> {
        // Destructure config for beauty
        let TransferXrdConfig {
            from_account,
            to_account_1,
            to_account_2,
        } = &self.config;

        // Handle the previous result, return the next result
        let up_next = match context.next_stage() {
            1 => {
                context.check_start(&previous)?;
                context.next_transaction_free_xrd_from_faucet(from_account.address)
            }
            2 => {
                context.check_commit_success(&previous)?;
                context.next_transaction_with_faucet_lock_fee(
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
                context.check_commit_success(&previous)?;
                context.next_transaction_with_faucet_lock_fee(
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
                context.check_commit_success(&previous)?;
                context.next_transaction_with_faucet_lock_fee(
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
                context.check_commit_success(&previous)?;
                context.next_transaction_with_faucet_lock_fee(
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
                context.check_commit_success(&previous)?;
                context.finish_scenario()
            }
        };
        Ok(up_next)
    }
}
