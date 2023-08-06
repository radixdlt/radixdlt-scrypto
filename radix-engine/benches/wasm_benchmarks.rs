use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;
use wasm_benchmarks::primitive;
use wasmer;
use wasmer_compiler_singlepass;
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

fn get_wasmer_instance(code: &[u8]) -> wasmer::Instance {
    let compiler = wasmer_compiler_singlepass::Singlepass::new();
    let store = wasmer::Store::new(&wasmer::Universal::new(compiler).engine());
    let module = wasmer::Module::new(&store, code).unwrap();
    let import_object = wasmer::imports! {};

    // instantiate
    wasmer::Instance::new(&module, &import_object).expect("Failed to instantiate module")
}

fn primitive_add_benchmark(c: &mut Criterion) {
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

    // wasmer
    group.bench_function("wasmer", |b| {
        let instance = get_wasmer_instance(&wasm_code[..]);
        let func = instance.exports.get_function("add").unwrap();

        black_box(b.iter(|| {
            for _ in 0..cnt {
                func.call(&[wasmer::Value::I64(1), wasmer::Value::I64(2)])
                    .unwrap();
            }
        }))
    });

    group.finish();
}

fn primitive_add_batch_benchmark(c: &mut Criterion) {
    let batch_len = 100_i32;
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
            .get_typed_func::<(u64, u64, i32), u64>(store.as_context_mut(), "add_batch")
            .unwrap();

        black_box(b.iter(|| {
            func.call(store.as_context_mut(), (1, 2, batch_len))
                .unwrap()
        }))
    });
    // wasmer
    group.bench_function("wasmer", |b| {
        let instance = get_wasmer_instance(&wasm_code[..]);
        let func = instance.exports.get_function("add_batch").unwrap();

        black_box(b.iter(|| {
            func.call(&[
                wasmer::Value::I64(1),
                wasmer::Value::I64(2),
                wasmer::Value::I32(batch_len),
            ])
            .unwrap();
        }))
    });

    group.finish();
}

fn primitive_mul_benchmark(c: &mut Criterion) {
    let cnt = 100_u32;
    let mut group = c.benchmark_group(format!("primitive_mul_{:?}x", cnt));

    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| {
            for _ in 0..cnt {
                primitive::mul(1, 2);
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
            .get_typed_func::<(u64, u64), u64>(store.as_context_mut(), "mul")
            .unwrap();

        black_box(b.iter(|| {
            for _ in 0..cnt {
                func.call(store.as_context_mut(), (1, 2)).unwrap();
            }
        }))
    });

    // wasmer
    group.bench_function("wasmer", |b| {
        let instance = get_wasmer_instance(&wasm_code[..]);
        let func = instance.exports.get_function("mul").unwrap();

        black_box(b.iter(|| {
            for _ in 0..cnt {
                func.call(&[wasmer::Value::I64(1), wasmer::Value::I64(2)])
                    .unwrap();
            }
        }))
    });

    group.finish();
}

fn primitive_mul_batch_benchmark(c: &mut Criterion) {
    let batch_len = 100_i32;
    let mut group = c.benchmark_group(format!("primitive_mul_batch_{:?}x", batch_len));

    // native Rust
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| primitive::mul_batch(1, 2, batch_len)))
    });

    let wasm_code = get_wasm_file();
    // wasmi
    group.bench_function("wasmi", |b| {
        let engine = wasmi::Engine::default();
        let mut store = wasmi::Store::new(&engine, 42);
        let instance = get_wasmi_instance(&engine, store.as_context_mut(), &wasm_code[..]);

        let func = instance
            .get_typed_func::<(u64, u64, i32), u64>(store.as_context_mut(), "mul_batch")
            .unwrap();

        black_box(b.iter(|| {
            func.call(store.as_context_mut(), (1, 2, batch_len))
                .unwrap()
        }))
    });
    // wasmer
    group.bench_function("wasmer", |b| {
        let instance = get_wasmer_instance(&wasm_code[..]);
        let func = instance.exports.get_function("mul_batch").unwrap();

        black_box(b.iter(|| {
            func.call(&[
                wasmer::Value::I64(1),
                wasmer::Value::I64(2),
                wasmer::Value::I32(batch_len),
            ])
            .unwrap();
        }))
    });

    group.finish();
}

fn primitive_pow_benchmark(c: &mut Criterion) {
    let cnt = 100_u32;
    let mut group = c.benchmark_group(format!("primitive_pow_{:?}x", cnt));

    // native Rust
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| {
            for _ in 0..cnt {
                primitive::pow(2, 20);
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
            .get_typed_func::<(i64, u32), i64>(store.as_context_mut(), "pow")
            .unwrap();

        black_box(b.iter(|| {
            for _ in 0..cnt {
                func.call(store.as_context_mut(), (2, 20)).unwrap();
            }
        }))
    });

    // wasmer
    group.bench_function("wasmer", |b| {
        let instance = get_wasmer_instance(&wasm_code[..]);
        let func = instance.exports.get_function("pow").unwrap();

        black_box(b.iter(|| {
            for _ in 0..cnt {
                func.call(&[wasmer::Value::I64(2), wasmer::Value::I32(20)])
                    .unwrap();
            }
        }))
    });

    group.finish();
}

fn primitive_pow_batch_benchmark(c: &mut Criterion) {
    let batch_len = 100_i32;
    let mut group = c.benchmark_group(format!("primitive_pow_batch_{:?}x", batch_len));

    // native Rust
    group.bench_function("rust-native", |b| {
        black_box(b.iter(|| primitive::pow_batch(2, 20, batch_len)))
    });

    let wasm_code = get_wasm_file();
    // wasmi
    group.bench_function("wasmi", |b| {
        let engine = wasmi::Engine::default();
        let mut store = wasmi::Store::new(&engine, 42);
        let instance = get_wasmi_instance(&engine, store.as_context_mut(), &wasm_code[..]);

        let func = instance
            .get_typed_func::<(i64, u32, i32), i64>(store.as_context_mut(), "pow_batch")
            .unwrap();

        black_box(b.iter(|| {
            func.call(store.as_context_mut(), (2, 20, batch_len))
                .unwrap()
        }))
    });
    // wasmer
    group.bench_function("wasmer", |b| {
        let instance = get_wasmer_instance(&wasm_code[..]);
        let func = instance.exports.get_function("pow_batch").unwrap();

        black_box(b.iter(|| {
            func.call(&[
                wasmer::Value::I64(2),
                wasmer::Value::I32(20),
                wasmer::Value::I32(batch_len),
            ])
            .unwrap();
        }))
    });

    group.finish();
}

criterion_group! {
    primitive_benches,
    primitive_add_benchmark,
    primitive_add_batch_benchmark,
    primitive_mul_benchmark,
    primitive_mul_batch_benchmark,
    primitive_pow_benchmark,
    primitive_pow_batch_benchmark
}
criterion_main!(primitive_benches);
