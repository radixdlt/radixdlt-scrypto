use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;

fn bench_decode_sbor(c: &mut Criterion) {
    c.bench_function("costing::decode_sbor", |b| {
        b.iter(|| manifest_decode::<ManifestValue>(include_bytes!("../../assets/radiswap.schema")))
    });
}

criterion_group!(costing, bench_decode_sbor,);
criterion_main!(costing);
