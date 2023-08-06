use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;
use wasm_benchmarks::primitive;
use wasmi::{self, AsContextMut};

type HostState = u32;

const WASM_BENCHMARKS_DIR: &str = "./wasm-benchmarks";

const WASM_BENCHMARKS_WASM_FILE: &str =
    "../target/wasm32-unknown-unknown/release/wasm_benchmarks.wasm";

// Compile wasm-benchmarks to WASM and return WASM code
// (in fact it would be enough to compile it just once before starting this benchmark,
// but couldn't find a proper place, where it could be done without bothering benchmark user)
fn get_wasm_file() -> Vec<u8> {
    let status = Command::new("cargo")
        .current_dir(WASM_BENCHMARKS_DIR)
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();
    if !status.success() {
        panic!("Failed to build wasm-benchmarks for wasm");
    }

    std::fs::read(WASM_BENCHMARKS_WASM_FILE)
        .expect(&format!("Cannot read file {:?}", WASM_BENCHMARKS_WASM_FILE))
}

// Instantiate WASMI instance with given WASM code
fn get_wasmi_instance(
    engine: &wasmi::Engine,
    mut store: wasmi::StoreContextMut<HostState>,
    code: &[u8],
) -> wasmi::Instance {
    let module = wasmi::Module::new(&engine, code).unwrap();
    let linker = <wasmi::Linker<HostState>>::new();
    linker
        .instantiate(store.as_context_mut(), &module)
        .unwrap()
        .ensure_no_start(store.as_context_mut())
        .unwrap()
}

fn add_benchmark(c: &mut Criterion) {
    let cnt = 100_u32;
    let mut group = c.benchmark_group(format!("primitive_add_{:?}x", cnt));

    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| {
            for _ in 0..cnt {
                primitive::add(1, 2);
            }
        }))
    });

    let wasm_code = get_wasm_file();
    // wasmi
    group.bench_function("wasmi", |b| {
        let engine = wasmi::Engine::default();
        let mut store = wasmi::Store::new(&engine, 42);
        let instance = get_wasmi_instance(&engine, store.as_context_mut(), &wasm_code[..]);

        let func = instance
            .get_typed_func::<(u64, u64), u64>(store.as_context_mut(), "add")
            .unwrap();

        black_box(b.iter(|| {
            for _ in 0..cnt {
                func.call(store.as_context_mut(), (1, 2)).unwrap();
            }
        }))
    });

    group.finish();
}

fn add_batch_benchmark(c: &mut Criterion) {
    let batch_len = 100_u32;
    let mut group = c.benchmark_group(format!("primitive_add_batch_{:?}x", batch_len));

    // native Rust
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| primitive::add_batch(1, 2, batch_len)))
    });

    let wasm_code = get_wasm_file();
    // wasmi
    group.bench_function("wasmi", |b| {
        let engine = wasmi::Engine::default();
        let mut store = wasmi::Store::new(&engine, 42);
        let instance = get_wasmi_instance(&engine, store.as_context_mut(), &wasm_code[..]);

        let func = instance
            .get_typed_func::<(u64, u64, u32), u64>(store.as_context_mut(), "add_batch")
            .unwrap();

        black_box(b.iter(|| {
            func.call(store.as_context_mut(), (1, 2, batch_len))
                .unwrap()
        }))
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
