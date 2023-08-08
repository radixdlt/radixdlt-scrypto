use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
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

fn get_wasmi_call_host_instance(
    engine: &wasmi::Engine,
    mut store: wasmi::StoreContextMut<HostState>,
    code: &[u8],
    func: fn(i64, i64, i64) -> i64,
) -> wasmi::Instance {
    let module = wasmi::Module::new(&engine, code).unwrap();
    let mut linker = <wasmi::Linker<HostState>>::new();
    let host_func = wasmi::Func::wrap(
        &mut store,
        move |_caller: wasmi::Caller<'_, HostState>,
              x: i64,
              y: i64,
              cnt: i64|
              -> Result<i64, wasmi::core::Trap> {
            let x = func(x, y, cnt);
            Ok(x)
        },
    );
    linker.define("host", "host_func", host_func).unwrap();

    linker
        .instantiate(store.as_context_mut(), &module)
        .unwrap()
        .ensure_no_start(store.as_context_mut())
        .unwrap()
}

macro_rules! bench_ops {
    ($t:literal, $ops:literal, $x:expr, $y:expr, $range:expr) => {
        paste::item! {
            pub fn [< $t _ $ops _benchmark >] (c: &mut Criterion) {
                let func_name = concat!($t, "_", $ops);
                let mut group = c.benchmark_group(func_name);
                let wasm_code = get_wasm_file();
                let wasm_call_host_code = wabt::wat2wasm(&WAT_CALL_HOST).unwrap();

                // wasmi stuff
                let engine = wasmi::Engine::default();
                let mut store = wasmi::Store::new(&engine, 42);
                let wasmi_instance = get_wasmi_instance(&engine, store.as_context_mut(), &wasm_code[..]);
                let wasmi_func = wasmi_instance
                    .get_typed_func::<(i64, i64, i64), i64>(store.as_context_mut(), func_name)
                    .unwrap();

                // wasmi stuff that calls native method
                let mut store_call_host = wasmi::Store::new(&engine, 42);
                let wasmi_call_host_instance = get_wasmi_call_host_instance(&engine, store_call_host.as_context_mut(), &wasm_call_host_code[..], [< $t _ $ops >]);
                let wasmi_call_host_func = wasmi_call_host_instance
                    .get_typed_func::<(i64, i64, i64), i64>(store_call_host.as_context_mut(), "local_call_host_func")
                    .unwrap();

                // wasmer stuff
                let wasmer_instance = get_wasmer_instance(&wasm_code[..]);
                let wasmer_func = wasmer_instance.exports.get_function(func_name).unwrap();

                for i in $range {
                    // native
                    group.bench_with_input(BenchmarkId::new("rust-native", i), &i, |b, i| {
                        b.iter(|| {
                            for _ in 0..*i {
                                black_box([< $t _ $ops >] ($x, $y, 0));
                            }
                        })

                    });
                    // wasmi call native
                    group.bench_with_input(BenchmarkId::new("wasmi-call-native", i), &i, |b, i| {
                        b.iter(|| {
                            for _ in 0..*i {
                                black_box(wasmi_call_host_func.call(store_call_host.as_context_mut(), ($x, $y, 0)).unwrap());
                            }
                        })
                    });
                    // wasmi
                    group.bench_with_input(BenchmarkId::new("wasmi", i), &i, |b, i| {
                        b.iter(|| {
                            for _ in 0..*i {
                                black_box(wasmi_func.call(store.as_context_mut(), ($x, $y, 0)).unwrap());
                            }
                        })
                    });

                    // wasmer
                    group.bench_with_input(BenchmarkId::new("wasmer", i), &i, |b, i| {
                        b.iter(|| {
                            for _ in 0..*i {
                                black_box(wasmer_func.call(&[
                                    wasmer::Value::I64($x),
                                    wasmer::Value::I64($y),
                                    wasmer::Value::I64(0)
                                ])
                                .unwrap());
                            }
                        })
                    });
                }

                group.finish();
            }
        }
    };
}

macro_rules! bench_ops_batch {
    ($t:literal, $ops:literal, $x:expr, $y:expr, $range:expr) => {
        paste::item! {
            pub fn [< $t _ $ops _batch_benchmark >] (c: &mut Criterion) {
                let func_name = concat!($t, "_", $ops, "_batch");
                let mut group = c.benchmark_group(func_name);
                let wasm_code = get_wasm_file();
                let wasm_call_host_code = wabt::wat2wasm(&WAT_CALL_HOST).unwrap();

                // wasmi stuff
                let engine = wasmi::Engine::default();
                let mut store = wasmi::Store::new(&engine, 42);
                let wasmi_instance = get_wasmi_instance(&engine, store.as_context_mut(), &wasm_code[..]);
                let wasmi_func = wasmi_instance
                    .get_typed_func::<(i64, i64, i64), i64>(store.as_context_mut(), func_name)
                    .unwrap();

                // wasmi stuff that calls native method
                let mut store_call_host = wasmi::Store::new(&engine, 42);
                let wasmi_call_host_instance = get_wasmi_call_host_instance(&engine, store_call_host.as_context_mut(), &wasm_call_host_code[..], [< $t _ $ops >]);
                let wasmi_call_host_func = wasmi_call_host_instance
                    .get_typed_func::<(i64, i64, i64), i64>(store_call_host.as_context_mut(), "local_call_host_func")
                    .unwrap();

                // wasmer stuff
                let wasmer_instance = get_wasmer_instance(&wasm_code[..]);
                let wasmer_func = wasmer_instance.exports.get_function(func_name).unwrap();

                for i in $range {
                    // native
                    group.bench_with_input(BenchmarkId::new("rust-native", i), &i, |b, i| {
                        black_box(b.iter(|| {
                            [< $t _ $ops _batch >] ($x, $y, *i);
                        }))
                    });
                    // wasmi call native
                    group.bench_with_input(BenchmarkId::new("wasmi-call-native", i), &i, |b, i| {
                        black_box(b.iter(|| {
                            wasmi_call_host_func.call(store_call_host.as_context_mut(), ($x, $y, *i)).unwrap();
                        }))
                    });
                    // wasmi
                    group.bench_with_input(BenchmarkId::new("wasmi", i), &i, |b, i| {
                        black_box(b.iter(|| {
                            wasmi_func.call(store.as_context_mut(), ($x, $y, *i)).unwrap();
                        }))
                    });

                    // wasmer
                    group.bench_with_input(BenchmarkId::new("wasmer", i), &i, |b, i| {
                        black_box(b.iter(|| {
                            wasmer_func.call(&[
                                    wasmer::Value::I64($x),
                                    wasmer::Value::I64($y),
                                    wasmer::Value::I64(*i)
                            ])
                            .unwrap();
                        }))
                    });
                }

                group.finish();
            }
        }
    };
}

#[cfg(feature = "full_wasm_benchmarks")]
const BENCH_RANGE: [i64; 3] = [1, 10, 10];
#[cfg(not(feature = "full_wasm_benchmarks"))]
const BENCH_RANGE: [i64; 1] = [1];

bench_ops!("primitive", "add", 1, 1, BENCH_RANGE);
bench_ops_batch!("primitive", "add", 1, 1, BENCH_RANGE);
bench_ops!("primitive", "mul", 1, 1, BENCH_RANGE);
bench_ops_batch!("primitive", "mul", 1, 1, BENCH_RANGE);
bench_ops!("primitive", "pow", 2, 20, BENCH_RANGE);
bench_ops_batch!("primitive", "pow", 2, 20, BENCH_RANGE);
bench_ops!("primitive", "fib", 1, 0, [1, 5, 20]);

// "no_conversion" function in benchmarked lib has the Decimal/PreciseDecimal
// operand hardcoded.
// Remaining functions convert the incoming arguments to Decimal/PreciseDecimal
// from i64, which impacts the benchmark time.
// The expected correlation is:
//  add = add_no_conversion + 2 x conversion time
//  conversion time = (add - add_no_conversion) / 2
//  mul = mul_no_conversion + 2 x conversion time
//  conversion time = (mul - add_no_conversion) / 2
bench_ops!("decimal", "add", 1, 1, BENCH_RANGE);
bench_ops!("decimal", "add_no_conversion", 1, 1, BENCH_RANGE);
bench_ops_batch!("decimal", "add", 1, 1, BENCH_RANGE);
bench_ops!("decimal", "mul", 1, 1, BENCH_RANGE);
bench_ops_batch!("decimal", "mul", 1, 1, BENCH_RANGE);
bench_ops!("decimal", "mul_no_conversion", 1, 1, BENCH_RANGE);
bench_ops!("decimal", "pow", 2, 20, BENCH_RANGE);
bench_ops_batch!("decimal", "pow", 2, 20, BENCH_RANGE);
bench_ops!("decimal", "fib", 1, 0, [1, 5, 20]);

bench_ops!("precise_decimal", "add", 1, 1, BENCH_RANGE);
bench_ops!("precise_decimal", "add_no_conversion", 1, 1, BENCH_RANGE);
bench_ops_batch!("precise_decimal", "add", 1, 1, BENCH_RANGE);
bench_ops!("precise_decimal", "mul", 1, 1, BENCH_RANGE);
bench_ops!("precise_decimal", "mul_no_conversion", 1, 1, BENCH_RANGE);
bench_ops_batch!("precise_decimal", "mul", 1, 1, BENCH_RANGE);
bench_ops!("precise_decimal", "pow", 2, 20, BENCH_RANGE);
bench_ops_batch!("precise_decimal", "pow", 2, 20, BENCH_RANGE);
bench_ops!("precise_decimal", "fib", 1, 0, [1, 5, 20]);

#[cfg(feature = "full_wasm_benchmarks")]
mod benches {
    use super::*;

    criterion_group! {
        name = primitive_benches;
        config = Criterion::default()
                    .sample_size(10)
                    .measurement_time(core::time::Duration::from_secs(2))
                    .warm_up_time(core::time::Duration::from_millis(500));
        targets = primitive_add_benchmark,
            primitive_add_batch_benchmark,
            primitive_mul_benchmark,
            primitive_mul_batch_benchmark,
            primitive_pow_benchmark,
            primitive_pow_batch_benchmark,
            primitive_fib_benchmark
    }
    criterion_group! {
        name = decimal_benches;
        config = Criterion::default()
                    .sample_size(10)
                    .measurement_time(core::time::Duration::from_secs(2))
                    .warm_up_time(core::time::Duration::from_millis(500));
        targets = decimal_add_benchmark,
            decimal_add_no_conversion_benchmark,
            decimal_add_batch_benchmark,
            decimal_mul_benchmark,
            decimal_mul_no_conversion_benchmark,
            decimal_mul_batch_benchmark,
            decimal_pow_benchmark,
            decimal_pow_batch_benchmark,
            decimal_fib_benchmark
    }
    criterion_group! {
        name = precise_decimal_benches;
        config = Criterion::default()
                    .sample_size(10)
                    .measurement_time(core::time::Duration::from_secs(2))
                    .warm_up_time(core::time::Duration::from_millis(500));
        targets = precise_decimal_add_benchmark,
            precise_decimal_add_no_conversion_benchmark,
            precise_decimal_add_batch_benchmark,
            precise_decimal_mul_benchmark,
            precise_decimal_mul_no_conversion_benchmark,
            precise_decimal_mul_batch_benchmark,
            precise_decimal_pow_benchmark,
            precise_decimal_pow_batch_benchmark,
            precise_decimal_fib_benchmark
    }
}
#[cfg(not(feature = "full_wasm_benchmarks"))]
mod benches {
    use super::*;

    criterion_group! {
        name = primitive_benches;
        config = Criterion::default()
                    .sample_size(10)
                    .measurement_time(core::time::Duration::from_secs(2))
                    .warm_up_time(core::time::Duration::from_millis(500));
        targets = primitive_mul_benchmark
    }
    criterion_group! {
        name = decimal_benches;
        config = Criterion::default()
                    .sample_size(10)
                    .measurement_time(core::time::Duration::from_secs(2))
                    .warm_up_time(core::time::Duration::from_millis(500));
        targets = decimal_mul_benchmark,
            decimal_mul_no_conversion_benchmark
    }
    criterion_group! {
        name = precise_decimal_benches;
        config = Criterion::default()
                    .sample_size(10)
                    .measurement_time(core::time::Duration::from_secs(2))
                    .warm_up_time(core::time::Duration::from_millis(500));
        targets = precise_decimal_mul_benchmark,
            precise_decimal_mul_no_conversion_benchmark
    }
}
criterion_main!(
    benches::primitive_benches,
    benches::decimal_benches,
    benches::precise_decimal_benches
);
