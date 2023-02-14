use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use radix_engine_interface::node::NetworkDefinition;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::TransactionHeader;
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::signing::EddsaEd25519PrivateKey;
use transaction::validation::verify_ecdsa_secp256k1;
use transaction::validation::verify_eddsa_ed25519;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::TestIntentHashManager;
use transaction::validation::ValidationConfig;
use transaction::validation::{recover_ecdsa_secp256k1, TransactionValidator};

fn bench_ecdsa_secp256k1_validation(c: &mut Criterion) {
    let message = "This is a long message".repeat(100);
    let signer = EcdsaSecp256k1PrivateKey::from_u64(123123123123).unwrap();
    let signature = signer.sign(message.as_bytes());

    c.bench_function("ECDSA signature validation", |b| {
        b.iter(|| {
            let public_key = recover_ecdsa_secp256k1(message.as_bytes(), &signature).unwrap();
            verify_ecdsa_secp256k1(message.as_bytes(), &public_key, &signature);
        })
    });
}

fn bench_eddsa_ed25519_validation(c: &mut Criterion) {
    let message = "This is a long message".repeat(100);
    let signer = EddsaEd25519PrivateKey::from_u64(123123123123).unwrap();
    let public_key = signer.public_key();
    let signature = signer.sign(message.as_bytes());

    c.bench_function("ED25519 signature validation", |b| {
        b.iter(|| {
            verify_eddsa_ed25519(message.as_bytes(), &public_key, &signature);
        })
    });
}

fn bench_transaction_validation(c: &mut Criterion) {
    let bech32_decoder: Bech32Decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
    let signer = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 1,
            notary_public_key: signer.public_key().into(),
            notary_as_signatory: true,
            cost_unit_limit: 1_000_000,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new()
                .withdraw_from_account_by_amount(account1, 1u32.into(), RADIX_TOKEN)
                .call_method(
                    account2,
                    "deposit_batch",
                    args!(ManifestExpression::EntireWorktop),
                )
                .build(),
        )
        .notarize(&signer)
        .build();
    let transaction_bytes = transaction.to_bytes().unwrap();
    println!("Transaction size: {} bytes", transaction_bytes.len());

    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    c.bench_function("Transaction validation", |b| {
        b.iter(|| {
            let intent_hash_manager = TestIntentHashManager::new();

            let transaction = validator
                .check_length_and_decode_from_slice(&transaction_bytes)
                .unwrap();
            validator
                .validate(&transaction, 0, &intent_hash_manager)
                .unwrap();
        })
    });
}

criterion_group!(
    transaction,
    bench_ecdsa_secp256k1_validation,
    bench_eddsa_ed25519_validation,
    bench_transaction_validation,
);
criterion_main!(transaction);
