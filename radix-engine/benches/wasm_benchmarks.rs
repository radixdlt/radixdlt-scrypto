use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wasm_benchmarks::primitive;

fn add_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitive_add");
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| {
            for _ in 0..100 {
                primitive::add(1, 2);
            }
        }))
    });

    group.finish();
}

fn add_batch_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitive_add_batch");
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| primitive::add_batch(1, 2, 100)))
    });

    group.finish();
}

fn pow_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitive_pow");
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| {
            for _ in 0..100 {
                primitive::pow(2, 20);
            }
        }))
    });

    group.finish();
}

fn pow_batch_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitive_pow_batch");
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| primitive::pow_batch(2, 20, 100)))
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = add_benchmark, add_batch_benchmark, pow_benchmark, pow_batch_benchmark
}
criterion_main!(benches);
