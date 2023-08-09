use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radix_engine_common::math::Decimal;
use std::process::Command;
use wasm_benchmarks_lib::*;
use wasmer::{self, WasmerEnv};
use wasmer_compiler_singlepass;
use wasmi::{self, AsContext, AsContextMut};

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

fn wasmi_decimal_mul_native(
    mut caller: wasmi::Caller<'_, HostState>,
    a_ptr: u32,
    b_ptr: u32,
    c_ptr: u32,
) -> Result<i64, wasmi::core::Trap> {
    let memory = match caller.get_export("memory") {
        Some(wasmi::Extern::Memory(memory)) => memory,
        _ => panic!("Failed to find memory export"),
    };

    let a_vec = wasmi_read_memory(caller.as_context(), memory, a_ptr, Decimal::BITS / 8);
    let b_vec = wasmi_read_memory(caller.as_context(), memory, b_ptr, Decimal::BITS / 8);
    let a = Decimal::try_from(&a_vec[..]).unwrap();
    let b = Decimal::try_from(&b_vec[..]).unwrap();

    let c = a * b;
    let c_vec = c.to_vec();
    wasmi_write_memory(caller.as_context_mut(), memory, c_ptr, &c_vec[..]);

    Ok(1)
}

// Instantiate WASMI instance with given WASM code
fn wasmi_get_instance(
    engine: &wasmi::Engine,
    mut store: wasmi::StoreContextMut<HostState>,
    code: &[u8],
) -> wasmi::Instance {
    let module = wasmi::Module::new(&engine, code).unwrap();
    let mut linker = <wasmi::Linker<HostState>>::new();

    let decimal_mul_native = wasmi::Func::wrap(&mut store, wasmi_decimal_mul_native);
    linker
        .define("env", "decimal_mul_native", decimal_mul_native)
        .unwrap();

    linker
        .instantiate(store.as_context_mut(), &module)
        .unwrap()
        .ensure_no_start(store.as_context_mut())
        .unwrap()
}

// wasmer
#[derive(Clone)]
struct WasmerInstanceEnv {
    instance: wasmer::LazyInit<wasmer::Instance>,
}

impl WasmerEnv for WasmerInstanceEnv {
    fn init_with_instance(
        &mut self,
        instance: &wasmer::Instance,
    ) -> Result<(), wasmer::HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}

fn wasmer_read_memory(memory: &wasmer::Memory, ptr: u32, len: usize) -> Vec<u8> {
    let ptr = ptr as usize;

    let memory_slice = unsafe { memory.data_unchecked() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        panic!("Memory access error");
    }

    memory_slice[ptr..ptr + len].to_vec()
}

fn wasmer_write_memory(memory: &wasmer::Memory, ptr: u32, data: &[u8]) {
    let ptr = ptr as usize;
    let len = data.len();

    let memory_slice = unsafe { memory.data_unchecked_mut() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        panic!("Memory access error");
    }

    memory_slice[ptr..ptr + data.len()].copy_from_slice(data);
}

fn wasmer_decimal_mul_native(
    env: &WasmerInstanceEnv,
    a_ptr: u32,
    b_ptr: u32,
    c_ptr: u32,
) -> Result<i64, wasmi::core::Trap> {
    let instance = unsafe { env.instance.get_unchecked() };
    let memory = instance
        .exports
        .get_memory("memory")
        .expect("Memory access error");

    let a_vec = wasmer_read_memory(&memory, a_ptr, Decimal::BITS / 8);
    let b_vec = wasmer_read_memory(&memory, b_ptr, Decimal::BITS / 8);
    let a = Decimal::try_from(&a_vec[..]).unwrap();
    let b = Decimal::try_from(&b_vec[..]).unwrap();

    let c = a * b;
    let c_vec = c.to_vec();
    wasmer_write_memory(&memory, c_ptr, &c_vec[..]);

    Ok(1)
}

fn wasmer_get_instance(code: &[u8]) -> wasmer::Instance {
    let compiler = wasmer_compiler_singlepass::Singlepass::new();
    let store = wasmer::Store::new(&wasmer::Universal::new(compiler).engine());
    let module = wasmer::Module::new(&store, code).unwrap();
    let mut env = WasmerInstanceEnv {
        instance: wasmer::LazyInit::new(),
    };

    let import_object = wasmer::imports! {
        "env" => {
            "decimal_mul_native" => wasmer::Function::new_native_with_env(module.store(), env.clone(), wasmer_decimal_mul_native)
        }
    };

    // instantiate
    let instance =
        wasmer::Instance::new(&module, &import_object).expect("Failed to instantiate module");

    env.init_with_instance(&instance.clone()).unwrap();

    instance
}

macro_rules! bench_ops {
    ($t:literal, $ops:literal) => {
        paste::item! {
            pub fn [< $t _ $ops _benchmark >] (c: &mut Criterion) {
                let func_name = concat!($t, "_", $ops);
                let func_call_native_name = &format!("{}_call_native", func_name);
                let mut group = c.benchmark_group(func_name);
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

                // wasmer stuff
                let wasmer_instance = wasmer_get_instance(&wasm_code[..]);
                let wasmer_func = wasmer_instance.exports.get_function(func_name).unwrap();
                let wasmer_func_call_native = wasmer_instance.exports.get_function(func_call_native_name).unwrap();

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

                // wasmi call native
                group.bench_function("wasmi-call-native", |b| {
                    b.iter(|| {
                        black_box(wasmi_func_call_native.call(store.as_context_mut(), ()).unwrap());
                    })
                });

                // wasmer call native
                group.bench_function("wasmer-call-native", |b| {
                    b.iter(|| {
                        black_box(wasmer_func_call_native.call(&[]).unwrap());
                    })
                });

                group.finish();
            }
        }
    };
}

//bench_ops!("primitive", "add");
//bench_ops!("primitive", "mul");

//bench_ops!("decimal", "add");
bench_ops!("decimal", "mul");

//bench_ops!("precise_decimal", "add");
//bench_ops!("precise_decimal", "mul");
/*
criterion_group! {
    name = primitive_benches;
    config = Criterion::default()
                .sample_size(10)
                .measurement_time(core::time::Duration::from_secs(2))
                .warm_up_time(core::time::Duration::from_millis(500));
    targets = primitive_add_benchmark,
        primitive_mul_benchmark
}
*/
criterion_group! {
    name = decimal_benches;
    config = Criterion::default()
                .sample_size(10)
                .measurement_time(core::time::Duration::from_secs(2))
                .warm_up_time(core::time::Duration::from_millis(500));
    targets = //decimal_add_benchmark,
        decimal_mul_benchmark
}
/*
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
*/
criterion_main!(decimal_benches);
