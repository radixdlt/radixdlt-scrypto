use radix_engine::constants::*;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::transaction::TransactionExecutor;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::WasmInstrumenter;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::TransactionHeader;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::ValidationConfig;
use transaction::validation::{TestIntentHashManager, TransactionValidator};

#[test]
fn test_normal_transaction_flow() {
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let intent_hash_manager = TestIntentHashManager::new();
    let validation_params = ValidationConfig {
        network: Network::LocalSimulator,
        current_epoch: 1,
        max_cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
        min_tip_percentage: 0,
    };
    let execution_params = ExecutionConfig::debug();

    let raw_transaction = create_transaction();
    let validated_transaction = TransactionValidator::validate_from_slice(
        &raw_transaction,
        &intent_hash_manager,
        &validation_params,
    )
    .expect("Invalid transaction");

    let mut executor = TransactionExecutor::new(
        &mut substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
    );
    let receipt = executor.execute_and_commit(&validated_transaction, &execution_params);

    receipt.expect_success();
}

#[test]
fn test_call_method_with_all_resources_doesnt_drop_auth_zone_proofs() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(dec!("10"), account)
        .create_proof_from_account(RADIX_TOKEN, account)
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_transaction_can_end_with_proofs_remaining_in_auth_zone() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(dec!("10"), account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_success();
}

fn create_transaction() -> Vec<u8> {
    // create key pairs
    let sk1 = EcdsaPrivateKey::from_u64(1).unwrap();
    let sk2 = EcdsaPrivateKey::from_u64(2).unwrap();
    let sk_notary = EcdsaPrivateKey::from_u64(3).unwrap();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network: Network::LocalSimulator,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 5,
            notary_public_key: sk_notary.public_key(),
            notary_as_signatory: false,
            cost_unit_limit: 1_000_000,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new(Network::LocalSimulator)
                .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build();

    transaction.to_bytes()
}
