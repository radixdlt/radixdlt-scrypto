use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radix_common::math::{traits::*, Decimal, PreciseDecimal};
use std::process::Command;
use wasm_benchmarks_lib::*;
use wasmi::{AsContext, AsContextMut};

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
        .unwrap_or_else(|_| panic!("Cannot read file {:?}", WASM_BENCHMARKS_WASM_FILE))
}

fn wasmi_read_memory(
    store: impl AsContext,
    memory: wasmi::Memory,
    ptr: u32,
    len: usize,
) -> Vec<u8> {
    let store_ctx = store.as_context();
    let data = memory.data(&store_ctx);
    let ptr = ptr as usize;

    if ptr > data.len() || ptr + len > data.len() {
        panic!("Memory access error");
    }
    data[ptr..ptr + len].to_vec()
}

fn wasmi_write_memory(mut store: impl AsContextMut, memory: wasmi::Memory, ptr: u32, data: &[u8]) {
    let mem_data = memory.data(store.as_context());

    if ptr as usize > mem_data.len() || ptr as usize + data.len() > mem_data.len() {
        panic!("Memory access error");
    }

    memory
        .write(&mut store.as_context_mut(), ptr as usize, data)
        .expect("Memory access error");
}

// Build wasmi native function, which:
// - reads input data from memory at offset a_ptr and b_ptr
// - and writes output data to memory at offset c_ptr
macro_rules! wasmi_native {
    ($type:ident, $ops:tt) => {
        paste::item! {
            fn [< wasmi_ $type:snake _ $ops _native >] (
                mut caller: wasmi::Caller<'_, HostState>,
                a_ptr: u32,
                b_ptr: u32,
                c_ptr: u32) -> Result<i64, wasmi::Error> {
                let memory = match caller.get_export("memory") {
                    Some(wasmi::Extern::Memory(memory)) => memory,
                    _ => panic!("Failed to find memory export"),
                };

                let a_vec = wasmi_read_memory(caller.as_context(), memory, a_ptr, <$type>::BITS / 8);
                let a = <$type>::try_from(&a_vec[..]).unwrap();

                let c = match stringify!($ops) {
                    "add" => {
                        let b_vec = wasmi_read_memory(caller.as_context(), memory, b_ptr, <$type>::BITS / 8);
                        let b = <$type>::try_from(&b_vec[..]).unwrap();
                        a.checked_add(b).unwrap()
                    },
                    "mul" => {
                        let b_vec = wasmi_read_memory(caller.as_context(), memory, b_ptr, <$type>::BITS / 8);
                        let b = <$type>::try_from(&b_vec[..]).unwrap();
                        a.checked_mul(b).unwrap()
                    },
                    "pow" => {
                        a.checked_powi(b_ptr.into()).unwrap()
                    },
                    _ => panic!("Unsupported operator!"),
                };

                let c_vec = c.to_vec();
                wasmi_write_memory(caller.as_context_mut(), memory, c_ptr, &c_vec[..]);

                Ok(1)
            }
        }
    };
}

wasmi_native!(Decimal, add);
wasmi_native!(Decimal, mul);
wasmi_native!(Decimal, pow);
wasmi_native!(PreciseDecimal, add);
wasmi_native!(PreciseDecimal, mul);
wasmi_native!(PreciseDecimal, pow);

// Instantiate WASMI instance with given WASM code
fn wasmi_get_instance(
    engine: &wasmi::Engine,
    mut store: wasmi::StoreContextMut<HostState>,
    code: &[u8],
) -> wasmi::Instance {
    let module = wasmi::Module::new(&engine, code).unwrap();
    let mut linker = <wasmi::Linker<HostState>>::new(engine);

    macro_rules! linker_define {
        ($name:tt) => {
            paste::item! {
                let $name = wasmi::Func::wrap(&mut store, [< wasmi_ $name >]);
                linker
                    .define("env", stringify!($name), $name)
                    .unwrap();
            }
        };
    }
    linker_define!(decimal_add_native);
    linker_define!(decimal_mul_native);
    linker_define!(decimal_pow_native);
    linker_define!(precise_decimal_add_native);
    linker_define!(precise_decimal_mul_native);
    linker_define!(precise_decimal_pow_native);

    linker
        .instantiate(store.as_context_mut(), &module)
        .unwrap()
        .ensure_no_start(store.as_context_mut())
        .unwrap()
}

macro_rules! bench_ops {
    ($t:literal, $ops:literal) => {
        paste::item! {
            pub fn [< $t _ $ops _benchmark >] (c: &mut Criterion) {
                let bench_name = concat!($t, "::", $ops);
                let func_name = concat!($t, "_", $ops);
                let func_call_native_name = &format!("{}_call_native", func_name);
                let mut group = c.benchmark_group(bench_name);
                let wasm_code = get_wasm_file();

                // wasmi stuff
                let engine = wasmi::Engine::default();
                let mut store = wasmi::Store::new(&engine, 42);
                let wasmi_instance = wasmi_get_instance(&engine, store.as_context_mut(), &wasm_code[..]);
                let wasmi_func = wasmi_instance
                    .get_typed_func::<(), i64>(store.as_context_mut(), func_name)
                    .unwrap();

                let wasmi_func_call_native = wasmi_instance
                    .get_typed_func::<(), i64>(store.as_context_mut(), func_call_native_name)
                    .unwrap();

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

                // wasmi call native
                group.bench_function("wasmi-call-native", |b| {
                    b.iter(|| {
                        black_box(wasmi_func_call_native.call(store.as_context_mut(), ()).unwrap());
                    })
                });

                group.finish();
            }
        }
    };
}

bench_ops!("decimal", "add");
bench_ops!("decimal", "mul");
bench_ops!("decimal", "pow");

bench_ops!("precise_decimal", "add");
bench_ops!("precise_decimal", "mul");
bench_ops!("precise_decimal", "pow");

criterion_group! {
    name = decimal_benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(core::time::Duration::from_secs(2))
        .warm_up_time(core::time::Duration::from_millis(500));
    targets = decimal_add_benchmark,
        decimal_mul_benchmark,
        decimal_pow_benchmark,
}

criterion_group! {
    name = precise_decimal_benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(core::time::Duration::from_secs(2))
        .warm_up_time(core::time::Duration::from_millis(500));
    targets = precise_decimal_add_benchmark,
        precise_decimal_mul_benchmark,
        precise_decimal_pow_benchmark,
}
criterion_main!(decimal_benches, precise_decimal_benches);
