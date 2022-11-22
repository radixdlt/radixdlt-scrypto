use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::model::extract_abi;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::InstrumentedCode;
use radix_engine::wasm::WasmEngine;
use radix_engine::wasm::WasmValidator;
use radix_engine_interface::crypto::hash;
use sbor::rust::sync::Arc;

fn bench_wasm_validation(c: &mut Criterion) {
    let code = include_bytes!("../../assets/account.wasm");
    let abi = extract_abi(code).unwrap();

    c.bench_function("WASM validation", |b| {
        b.iter(|| WasmValidator::default().validate(code, &abi))
    });
}

fn bench_wasm_instantiation(c: &mut Criterion) {
    let code = include_bytes!("../../assets/account.wasm").to_vec();
    let code_hash = hash(&code);
    let pretend_instrumented_code = InstrumentedCode {
        code: Arc::new(code),
        code_hash,
    };
    c.bench_function("WASM instantiation", |b| {
        b.iter(|| {
            let engine = DefaultWasmEngine::default();
            engine.instantiate(&pretend_instrumented_code);
        })
    });
}

fn bench_wasm_instantiation_pre_loaded(c: &mut Criterion) {
    let code = include_bytes!("../../assets/account.wasm").to_vec();
    let code_hash = hash(&code);
    let pretend_instrumented_code = InstrumentedCode {
        code: Arc::new(code),
        code_hash,
    };
    let engine = DefaultWasmEngine::default();
    engine.instantiate(&pretend_instrumented_code);
    c.bench_function("WASM instantiation (pre-loaded)", |b| {
        b.iter(|| {
            engine.instantiate(&pretend_instrumented_code);
        })
    });
}

criterion_group!(
    wasm,
    bench_wasm_validation,
    bench_wasm_instantiation,
    bench_wasm_instantiation_pre_loaded
);
criterion_main!(wasm);
