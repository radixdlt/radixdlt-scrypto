use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::TransactionHeaderV1;
use transaction::model::TransactionPayload;
use transaction::signing::ed25519::Ed25519PrivateKey;
use transaction::signing::secp256k1::Secp256k1PrivateKey;
use transaction::validation::verify_ed25519;
use transaction::validation::verify_secp256k1;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::ValidationConfig;
use transaction::validation::{recover_secp256k1, TransactionValidator};

fn bench_secp256k1_validation(c: &mut Criterion) {
    let message_hash = hash("This is a long message".repeat(100));
    let signer = Secp256k1PrivateKey::from_u64(123123123123).unwrap();
    let signature = signer.sign(&message_hash);

    c.bench_function("Validation::verify_ecdsa", |b| {
        b.iter(|| {
            let public_key = recover_secp256k1(&message_hash, &signature).unwrap();
            verify_secp256k1(&message_hash, &public_key, &signature);
        })
    });
}

fn bench_ed25519_validation(c: &mut Criterion) {
    let message_hash = hash("This is a long message".repeat(100));
    let signer = Ed25519PrivateKey::from_u64(123123123123).unwrap();
    let public_key = signer.public_key();
    let signature = signer.sign(&message_hash);

    c.bench_function("Validation::verify_ed25519", |b| {
        b.iter(|| {
            verify_ed25519(&message_hash, &public_key, &signature);
        })
    });
}

fn bench_transaction_validation(c: &mut Criterion) {
    let address_bech32_decoder: AddressBech32Decoder =
        AddressBech32Decoder::new(&NetworkDefinition::simulator());

    let account1 = ComponentAddress::try_from_bech32(
        &address_bech32_decoder,
        "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
    )
    .unwrap();
    let account2 = ComponentAddress::try_from_bech32(
        &address_bech32_decoder,
        "account_sim1cyzfj6p254jy6lhr237s7pcp8qqz6c8ahq9mn6nkdjxxxat5syrgz9",
    )
    .unwrap();
    let signer = Secp256k1PrivateKey::from_u64(1).unwrap();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(100),
            nonce: 1,
            notary_public_key: signer.public_key().into(),
            notary_is_signatory: true,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new()
                .withdraw_from_account(account1, RADIX_TOKEN, 1u32.into())
                .call_method(
                    account2,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
        )
        .notarize(&signer)
        .build();
    let transaction_bytes = transaction.to_payload_bytes().unwrap();
    println!("Transaction size: {} bytes", transaction_bytes.len());

    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    c.bench_function("Validation::validate_manifest", |b| {
        b.iter(|| {
            black_box(
                validator
                    .validate_from_payload_bytes(&transaction_bytes)
                    .unwrap(),
            )
        })
    });
}

criterion_group!(
    validation,
    bench_secp256k1_validation,
    bench_ed25519_validation,
    bench_transaction_validation,
);
criterion_main!(validation);
