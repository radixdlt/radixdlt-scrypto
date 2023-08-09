use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;
use wasm_benchmarks_lib::*;
use wasmer;
use wasmer_compiler_singlepass;
use wasmi::{self, AsContextMut};

type HostState = u32;

const WASM_BENCHMARKS_DIR: &str = "./wasm-benchmarks-lib";

const WASM_BENCHMARKS_WASM_FILE: &str =
    "../target/wasm32-unknown-unknown/release/wasm_benchmarks_lib.wasm";

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

macro_rules! bench_ops {
    ($t:literal, $ops:literal) => {
        paste::item! {
            pub fn [< $t _ $ops _benchmark >] (c: &mut Criterion) {
                let func_name = concat!($t, "_", $ops);
                let mut group = c.benchmark_group(func_name);
                let wasm_code = get_wasm_file();

                // wasmi stuff
                let engine = wasmi::Engine::default();
                let mut store = wasmi::Store::new(&engine, 42);
                let wasmi_instance = get_wasmi_instance(&engine, store.as_context_mut(), &wasm_code[..]);
                let wasmi_func = wasmi_instance
                    .get_typed_func::<(), i64>(store.as_context_mut(), func_name)
                    .unwrap();

                // wasmer stuff
                let wasmer_instance = get_wasmer_instance(&wasm_code[..]);
                let wasmer_func = wasmer_instance.exports.get_function(func_name).unwrap();

                // native
                group.bench_function("rust-native", |b| {
                    b.iter(|| {
                        black_box([< $t _ $ops >] ());
                    })

                });

                // wasmi
                group.bench_function("wasmi", |b| {
                    b.iter(|| {
                        black_box(wasmi_func.call(store.as_context_mut(), ()).unwrap());
                    })
                });

                // wasmer
                group.bench_function("wasmer", |b| {
                    b.iter(|| {
                        black_box(wasmer_func.call(&[]).unwrap());
                    })
                });

                group.finish();
            }
        }
    };
}

bench_ops!("primitive", "add");
bench_ops!("primitive", "mul");

bench_ops!("decimal", "add");
bench_ops!("decimal", "mul");

bench_ops!("precise_decimal", "add");
bench_ops!("precise_decimal", "mul");

criterion_group! {
    name = primitive_benches;
    config = Criterion::default()
                .sample_size(10)
                .measurement_time(core::time::Duration::from_secs(2))
                .warm_up_time(core::time::Duration::from_millis(500));
    targets = primitive_add_benchmark,
        primitive_mul_benchmark
}
criterion_group! {
    name = decimal_benches;
    config = Criterion::default()
                .sample_size(10)
                .measurement_time(core::time::Duration::from_secs(2))
                .warm_up_time(core::time::Duration::from_millis(500));
    targets = decimal_add_benchmark,
        decimal_mul_benchmark
}
criterion_group! {
    name = precise_decimal_benches;
    config = Criterion::default()
                .sample_size(10)
                .measurement_time(core::time::Duration::from_secs(2))
                .warm_up_time(core::time::Duration::from_millis(500));
    targets = precise_decimal_add_benchmark,
        precise_decimal_mul_benchmark
}

criterion_main!(primitive_benches, decimal_benches, precise_decimal_benches);
