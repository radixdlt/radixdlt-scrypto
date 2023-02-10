use ::sha2::Sha512_256;
use sha2::digest::Digest;
use blake2::{digest::consts::U32, Blake2b};
use blake3;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use radix_engine_interface::crypto::{sha256, sha256_twice, sha3};
//use webb_pedersen_hash;
//
use blake2b_simd::Params;

//use dusk_bls12_381::BlsScalar;
//use dusk_bytes::Serializable;
//use dusk_poseidon::sponge;

const KB: u32 = 1024;
const MB: u32 = 1024 * KB;

fn sha2_512_256<T: AsRef<[u8]>>(data: T) -> [u8; 32] {
    let mut hasher = Sha512_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn blake2b_hash<T: AsRef<[u8]>>(data: T) -> [u8; 32] {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn blake2b_simd_hash<T: AsRef<[u8]>>(data: T) -> [u8; 32] {
    let mut hasher = Params::new().hash_length(32).to_state();
    hasher.update(data.as_ref());
    hasher.finalize().as_bytes().try_into().expect("incorrect slice length")
}
/*
fn poseidon_hash<T: AsRef<[u8]>>(data: T) -> [u8; 32] {
    let mut vec_scalar = Vec::<BlsScalar>::new();

    for i in (0..data.as_ref().len()).step_by(32) {
        let scalar = BlsScalar::from_bytes(data.as_ref()[i..i + 32].try_into().unwrap()).unwrap();
        vec_scalar.push(scalar);
    }
    sponge::hash(&vec_scalar).to_bytes()
}

mod k12 {
    use k12::digest::{Update, ExtendableOutput};
    use k12::KangarooTwelve;

    pub fn k12_hash(data:  &[u8]) -> [u8; 32] {
        let mut hasher = KangarooTwelve::new();
        let mut out: [u8; 32] = [0; 32];
        hasher.update(data);
        hasher.finalize_xof_into(&mut out);
        out
    }
}
*/
fn bench_hash(c: &mut Criterion) {
    let sizes = [(64, "64B"), (512, "512B"), (MB, "1MB")];
    let mut group = c.benchmark_group("hash");
    for size in sizes {
        let data = vec![0_u8; size.0 as usize];

        group.throughput(Throughput::Bytes(size.0 as u64));

        // SHA2-256
        group.bench_with_input(BenchmarkId::new("SHA2-256", size.1), &data[..], |b, d| {
            b.iter(|| sha256(d))
        });

        // SHA2-256_twice
        group.bench_with_input(
            BenchmarkId::new("SHA2-256_twice", size.1),
            &data[..],
            |b, d| b.iter(|| sha256_twice(d)),
        );

        // SHA2-512/256
        group.bench_with_input(
            BenchmarkId::new("SHA2-512_256", size.1),
            &data[..],
            |b, d| b.iter(|| sha2_512_256(d)),
        );

        // SHA3-256
        group.bench_with_input(BenchmarkId::new("SHA3-256", size.1), &data[..], |b, d| {
            b.iter(|| sha3(d))
        });

        // Blake2b
        group.bench_with_input(BenchmarkId::new("Blake2b", size.1), &data[..], |b, d| {
            b.iter(|| blake2b_hash(d))
        });

        // blake2b_simd
        group.bench_with_input(BenchmarkId::new("blake2b_simd", size.1), &data[..], |b, d| {
            b.iter(|| blake2b_simd_hash(d))
        });

        // Blake3
        group.bench_with_input(BenchmarkId::new("Blake3", size.1), &data[..], |b, d| {
            b.iter(|| blake3::hash(d))
        });

        // Pedersen Hash
//        group.bench_with_input(BenchmarkId::new("Pedersen", size.1), &data[..], |b, d| {
//            b.iter(|| webb_pedersen_hash::hash(d))
//        });

        // Poseidon Hash
//        group.bench_with_input(BenchmarkId::new("Poseidon", size.1), &data[..], |b, d| {
//            b.iter(|| poseidon_hash(d))
//        });

        // KangarooTwelve hash
//        group.bench_with_input(BenchmarkId::new("k12", size.1), &data[..], |b, d| {
//            b.iter(|| k12::k12_hash(d))
//        });
    }
}

criterion_group!(bench_main, bench_hash,);
criterion_main!(bench_main);
