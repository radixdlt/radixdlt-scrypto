use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;

fn bench_decode_sbor(c: &mut Criterion) {
    let payload = include_bytes!("../../assets/radiswap.schema");
    println!("Payload size: {}", payload.len());
    c.bench_function("costing::decode_sbor", |b| {
        b.iter(|| manifest_decode::<ManifestValue>(payload))
    });
}

criterion_group!(costing, bench_decode_sbor);
criterion_main!(costing);
