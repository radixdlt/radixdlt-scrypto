use radix_engine::engine::TransactionExecutor;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::WasmInstrumenter;
use scrypto::core::Network;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::TransactionHeader;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::{TestEpochManager, TestIntentHashManager, TransactionValidator};

#[test]
fn test_normal_transaction_flow() {
    let substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let epoch_manager = TestEpochManager::new(0);
    let intent_hash_manager = TestIntentHashManager::new();

    let raw_transaction = create_transaction();
    let validated_transaction = TransactionValidator::validate_from_slice(
        &raw_transaction,
        &intent_hash_manager,
        &epoch_manager,
    )
    .expect("Invalid transaction");

    let mut executor = TransactionExecutor::new(
        substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
        true,
    );
    let receipt = executor.execute(&validated_transaction);

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
        })
        .manifest(
            ManifestBuilder::new(Network::LocalSimulator)
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build();

    transaction.to_bytes()
}
