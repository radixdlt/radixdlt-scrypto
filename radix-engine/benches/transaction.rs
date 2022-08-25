use criterion::{criterion_group, criterion_main, Criterion};
use scrypto::address::Bech32Decoder;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::TransactionHeader;
use transaction::signing::EcdsaPrivateKey;
use transaction::signing::Ed25519PrivateKey;
use transaction::validation::verify_ecdsa;
use transaction::validation::verify_ed25519;
use transaction::validation::TestIntentHashManager;
use transaction::validation::TransactionValidator;
use transaction::validation::ValidationConfig;

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
    let bech32_decoder: Bech32Decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    let account1 = bech32_decoder
        .validate_and_decode_component_address(
            "account_sim1qd5yl2lwcuk25q705sm9g7cven6f9kytjs8t2xvvggzq5d2mse",
        )
        .unwrap();
    let account2 = bech32_decoder
        .validate_and_decode_component_address(
            "account_sim1qdugngtu8y0e54zwe5j54mdwv3wnkse68u9840407zws6fzpn7",
        )
        .unwrap();
    let signer = EcdsaPrivateKey::from_u64(1).unwrap();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network_id: NetworkDefinition::local_simulator().id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 1,
            notary_public_key: signer.public_key(),
            notary_as_signatory: true,
            cost_unit_limit: 1_000_000,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new(&NetworkDefinition::local_simulator())
                .withdraw_from_account_by_amount(1u32.into(), RADIX_TOKEN, account1)
                .call_method(
                    account2,
                    "deposit_batch",
                    args!(Expression::new("ALL_WORKTOP_RESOURCES")),
                )
                .build(),
        )
        .notarize(&signer)
        .build();
    let transaction_bytes = transaction.to_bytes();
    println!("Transaction size: {} bytes", transaction_bytes.len());

    c.bench_function("Transaction validation", |b| {
        b.iter(|| {
            let intent_hash_manager = TestIntentHashManager::new();
            let config: ValidationConfig = ValidationConfig {
                network: &NetworkDefinition::local_simulator(),
                current_epoch: 1,
                max_cost_unit_limit: 10_000_000,
                min_tip_percentage: 0,
            };

            TransactionValidator::validate_from_slice(
                &transaction_bytes,
                &intent_hash_manager,
                &config,
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
