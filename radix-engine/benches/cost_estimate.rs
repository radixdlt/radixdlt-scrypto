use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::model::extract_package;
use radix_engine::wasm::WasmValidator;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::Network;
use transaction::model::TransactionHeader;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::TestEpochManager;
use transaction::validation::TestIntentHashManager;
use transaction::validation::TransactionValidator;

fn bench_transaction_validation(c: &mut Criterion) {
    let account1 =
        ComponentAddress::from_str("02684fabeec72caa03cfa436547b0cccf492d88b940eb5198c4204")
            .unwrap();
    let account2 =
        ComponentAddress::from_str("027889a17c391f9a544ecd254aedae645d3b433a3f0a7abeaff09d")
            .unwrap();
    let signer = EcdsaPrivateKey::from_u64(1).unwrap();

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(1u32.into(), RADIX_TOKEN, account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build();
    let transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network: Network::InternalTestnet,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 1,
            notary_public_key: signer.public_key(),
            notary_as_signatory: true,
        })
        .manifest(manifest)
        .notarize(&signer)
        .build();
    let transaction_bytes = transaction.to_bytes();
    println!("Transaction size: {} bytes", transaction_bytes.len());

    // Loop
    c.bench_function("Validate Transaction", |b| {
        b.iter(|| {
            let epoch_manager = TestEpochManager::new(0);
            let intent_hash_manager = TestIntentHashManager::new();

            TransactionValidator::validate_from_slice(
                &transaction_bytes,
                &intent_hash_manager,
                &epoch_manager,
            )
            .unwrap();
        })
    });
}

fn bench_wasm_validation(c: &mut Criterion) {
    let code = include_bytes!("../../assets/account.wasm");
    let package = extract_package(code.to_vec()).unwrap();

    c.bench_function("Validate Wasm", |b| {
        b.iter(|| WasmValidator::default().validate(&package.code, &package.blueprints))
    });
}

criterion_group!(
    cost_estimate,
    bench_transaction_validation,
    bench_wasm_validation
);
criterion_main!(cost_estimate);
