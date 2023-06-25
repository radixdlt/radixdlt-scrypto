use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use transaction::{
    prelude::Secp256k1PrivateKey,
    validation::{recover_secp256k1, verify_secp256k1},
};

fn bench_decode_sbor(c: &mut Criterion) {
    let payload = include_bytes!("../../assets/radiswap.schema");
    println!("Payload size: {}", payload.len());
    c.bench_function("costing::decode_sbor", |b| {
        b.iter(|| manifest_decode::<ManifestValue>(payload))
    });
}

fn bench_validate_secp256k1(c: &mut Criterion) {
    let message = "m".repeat(1_000_000);
    let message_hash = hash(message.as_bytes());
    let signer = Secp256k1PrivateKey::from_u64(123123123123).unwrap();
    let signature = signer.sign(&message_hash);

    c.bench_function("costing::validate_secp256k1", |b| {
        b.iter(|| {
            let public_key = recover_secp256k1(&message_hash, &signature).unwrap();
            verify_secp256k1(&message_hash, &public_key, &signature);
        })
    });
}

criterion_group!(costing, bench_decode_sbor, bench_validate_secp256k1);
criterion_main!(costing);
