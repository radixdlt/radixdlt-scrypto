use criterion::{criterion_group, criterion_main, Criterion};
use scrypto::core::Network;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::TransactionHeader;
use transaction::signing::EcdsaPrivateKey;
use transaction::signing::Ed25519PrivateKey;
use transaction::validation::verify_ecdsa;
use transaction::validation::verify_ed25519;
use transaction::validation::TestEpochManager;
use transaction::validation::TestIntentHashManager;
use transaction::validation::TransactionValidator;

fn bench_ecdsa_validation(c: &mut Criterion) {
    let message = "This is a long message".repeat(100);
    let signer = EcdsaPrivateKey::from_u64(123123123123).unwrap();
    let public_key = signer.public_key();
    let signature = signer.sign(message.as_bytes());

    c.bench_function("ECDSA signature validation", |b| {
        b.iter(|| {
            verify_ecdsa(message.as_bytes(), &public_key, &signature);
        })
    });
}

fn bench_ed25519_validation(c: &mut Criterion) {
    let message = "This is a long message".repeat(100);
    let signer = Ed25519PrivateKey::from_u64(123123123123).unwrap();
    let public_key = signer.public_key();
    let signature = signer.sign(message.as_bytes());

    c.bench_function("ED25519 signature validation", |b| {
        b.iter(|| {
            verify_ed25519(message.as_bytes(), &public_key, &signature);
        })
    });
}

fn bench_transaction_validation(c: &mut Criterion) {
    let account1 =
        ComponentAddress::from_str("02684fabeec72caa03cfa436547b0cccf492d88b940eb5198c4204")
            .unwrap();
    let account2 =
        ComponentAddress::from_str("027889a17c391f9a544ecd254aedae645d3b433a3f0a7abeaff09d")
            .unwrap();
    let signer = EcdsaPrivateKey::from_u64(1).unwrap();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network: Network::LocalSimulator,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 1,
            notary_public_key: signer.public_key(),
            notary_as_signatory: true,
        })
        .manifest(
            ManifestBuilder::new(Network::LocalSimulator)
                .withdraw_from_account_by_amount(1u32.into(), RADIX_TOKEN, account1)
                .call_method_with_all_resources(account2, "deposit_batch")
                .build(),
        )
        .notarize(&signer)
        .build();
    let transaction_bytes = transaction.to_bytes();
    println!("Transaction size: {} bytes", transaction_bytes.len());

    c.bench_function("Transaction validation", |b| {
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

criterion_group!(
    transaction,
    bench_ecdsa_validation,
    bench_ed25519_validation,
    bench_transaction_validation,
);
criterion_main!(transaction);
