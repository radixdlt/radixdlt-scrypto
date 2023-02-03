use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use radix_engine_interface::crypto::{sha256, sha256_twice};

const KB: u32 = 1024;
const MB: u32 = 1024 * KB;

fn bench_hash(c: &mut Criterion) {
    let sizes = [(500, "500B"), (MB, "1MB")];
    let mut group = c.benchmark_group("hash");
    for size in sizes {
        let data = vec![0_u8; size.0 as usize];

        group.throughput(Throughput::Bytes(size.0 as u64));

        group.bench_with_input(BenchmarkId::new("sha256", size.1), &data[..], |b, d| {
            b.iter(|| sha256(d))
        });

        group.bench_with_input(
            BenchmarkId::new("sha256_twice", size.1),
            &data[..],
            |b, d| b.iter(|| sha256_twice(d)),
        );
    }
}

criterion_group!(bench_main, bench_hash,);
criterion_main!(bench_main);
