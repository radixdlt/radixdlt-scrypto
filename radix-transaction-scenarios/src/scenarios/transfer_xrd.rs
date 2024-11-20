use radix_engine::updates::ProtocolVersion;

use crate::internal_prelude::*;

#[allow(deprecated)]
pub struct TransferXrdConfig {
    pub from_account: PreallocatedAccount,
    pub to_account_1: PreallocatedAccount,
    pub to_account_2: PreallocatedAccount,
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

pub struct TransferXrdScenarioCreator;

impl ScenarioCreator for TransferXrdScenarioCreator {
    type Config = TransferXrdConfig;
    type State = ();
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "transfer_xrd",
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
        #[allow(unused_variables)]
        ScenarioBuilder::new(core, Self::METADATA, config, start_state)
            .successful_transaction(|core, config, state| {
                core.next_transaction_free_xrd_from_faucet(config.from_account.address)
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee_fallible(
                    "transfer--try_deposit_or_abort",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .take_from_worktop(XRD, dec!(1), "xrd")
                            .try_deposit_or_abort(config.to_account_1.address, None, "xrd")
                            .done()
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee_fallible(
                    "transfer--try_deposit_or_refund",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .take_from_worktop(XRD, dec!(1), "xrd")
                            .try_deposit_or_refund(config.to_account_1.address, None, "xrd")
                            .done()
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee_fallible(
                    "transfer--try_deposit_batch_or_abort",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_entire_worktop_or_abort(config.to_account_1.address, None)
                            .done()
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee_fallible(
                    "transfer--try_deposit_batch_or_refund",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_entire_worktop_or_refund(config.to_account_1.address, None)
                            .done()
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee_fallible(
                    "self-transfer--deposit_batch",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .deposit_entire_worktop(config.from_account.address)
                            .done()
                    },
                    vec![&config.from_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee_fallible(
                    "multi-transfer--deposit_batch",
                    |builder| {
                        builder
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_entire_worktop_or_abort(config.to_account_1.address, None)
                            .withdraw_from_account(config.from_account.address, XRD, dec!(1))
                            .try_deposit_entire_worktop_or_abort(config.to_account_2.address, None)
                            .done()
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
