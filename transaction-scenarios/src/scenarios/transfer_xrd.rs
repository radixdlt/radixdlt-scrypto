use crate::internal_prelude::*;

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

pub enum TransferXrdScenarioCreator {}

impl ScenarioCreator for TransferXrdScenarioCreator {
    type Config = TransferXrdConfig;
    type State = ();

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "transfer_xrd",
        };

        #[allow(unused_variables)]
        ScenarioBuilder::new(core, metadata, config, start_state)
            .successful_transaction(|core, config, state| -> Result<_, ScenarioError> {
                core.next_transaction_free_xrd_from_faucet(config.from_account.address)
            })
            .successful_transaction(|core, config, state| -> Result<_, ScenarioError> {
                core.next_transaction_with_faucet_lock_fee(
                    "transfer--try_deposit_batch_or_abort",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_abort(config.to_account_1.address)
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| -> Result<_, ScenarioError> {
                core.next_transaction_with_faucet_lock_fee(
                    "transfer--try_deposit_batch_or_refund",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_refund(config.to_account_1.address)
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| -> Result<_, ScenarioError> {
                core.next_transaction_with_faucet_lock_fee(
                    "self-transfer--deposit_batch",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .deposit_batch(config.from_account.address)
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| -> Result<_, ScenarioError> {
                core.next_transaction_with_faucet_lock_fee(
                    "multi-transfer--deposit_batch",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_abort(config.to_account_1.address)
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_batch_or_abort(config.to_account_2.address)
                    },
                    vec![&config.from_account.key],
                )
            })
            .finalize(|core, config, state| -> Result<_, ScenarioError> {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("from_account", config.from_account.address)
                        .add("to_account_1", config.to_account_1.address)
                        .add("to_account_2", config.to_account_2.address),
                })
            })
    }
}
