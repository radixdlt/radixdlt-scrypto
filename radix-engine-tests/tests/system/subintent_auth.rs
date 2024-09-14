use radix_common::constants::TRANSACTION_PROCESSOR_PACKAGE;
use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use radix_engine_interface::macros::dec;
use radix_transactions::builder::{ManifestBuilder, ResolvableArguments};
use radix_transactions::manifest::YieldToChild;
use radix_transactions::model::{InstructionV1, ManifestNamedIntentIndex, TestTransaction};
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;

#[test]
fn should_not_be_able_to_use_root_auth_in_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: ().resolve(),
                })
                .0
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestBuilder::new_v2()
                .withdraw_from_account(account, XRD, dec!(10))
                .deposit_entire_worktop(account)
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset!(),
            )
        },
    ];

    let receipt = ledger.execute_test_transaction(TestTransaction::new_v2_from_nonce(intents));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
        )
    });
}

#[test]
fn should_be_able_to_use_separate_auth_in_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (public_key2, _, account2) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: ().resolve(),
                })
                .0
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestBuilder::new_v2()
                .withdraw_from_account(account2, XRD, dec!(10))
                .deposit_entire_worktop(account2)
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key2)],
            )
        },
    ];

    let receipt = ledger.execute_test_transaction(TestTransaction::new_v2_from_nonce(intents));

    // Assert
    receipt.expect_commit_success();
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ManifestTransactionProcessorRunInput {
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<()>,
    pub references: Vec<()>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

#[test]
fn should_not_be_able_to_call_tx_processor_in_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: ().resolve(),
                })
                .0
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let instructions: Vec<InstructionV1> = Vec::new();
            let manifest_encoded_instructions = manifest_encode(&instructions).unwrap();
            let manifest = ManifestBuilder::new_v2()
                .call_function(
                    TRANSACTION_PROCESSOR_PACKAGE,
                    TRANSACTION_PROCESSOR_BLUEPRINT,
                    TRANSACTION_PROCESSOR_RUN_IDENT,
                    ManifestTransactionProcessorRunInput {
                        manifest_encoded_instructions,
                        global_address_reservations: vec![],
                        references: vec![],
                        blobs: index_map_new(),
                    },
                )
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![],
            )
        },
    ];

    let receipt = ledger.execute_test_transaction(TestTransaction::new_v2_from_nonce(intents));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}
