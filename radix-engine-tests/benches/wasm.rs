use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::EntityType;
use radix_engine::utils::extract_schema;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::InstrumentedCode;
use radix_engine::wasm::WasmEngine;
use radix_engine::wasm::WasmMeteringConfig;
use radix_engine::wasm::WasmValidator;
use radix_engine_common::types::package_address;
use sbor::rust::sync::Arc;

fn bench_wasm_validation(c: &mut Criterion) {
    let code = include_bytes!("../../assets/faucet.wasm");
    let abi = extract_schema(code).unwrap();

    c.bench_function("WASM::validate_wasm", |b| {
        b.iter(|| WasmValidator::default().validate(code, &abi))
    });
}

fn bench_wasm_instantiation(c: &mut Criterion) {
    let package_address = package_address(EntityType::GlobalPackage, 77);
    let code = include_bytes!("../../assets/faucet.wasm").to_vec();
    let pretend_instrumented_code = InstrumentedCode {
        metered_code_key: (package_address, WasmMeteringConfig::V0),
        code: Arc::new(code),
    };
    c.bench_function("WASM::instantiate_wasm", |b| {
        b.iter(|| {
            let engine = DefaultWasmEngine::default();
            engine.instantiate(&pretend_instrumented_code);
        })
    });
}

fn bench_wasm_instantiation_pre_loaded(c: &mut Criterion) {
    let package_address = package_address(EntityType::GlobalPackage, 99);
    let code = include_bytes!("../../assets/faucet.wasm").to_vec();
    let pretend_instrumented_code = InstrumentedCode {
        metered_code_key: (package_address, WasmMeteringConfig::V0),
        code: Arc::new(code),
    };
    let engine = DefaultWasmEngine::default();
    engine.instantiate(&pretend_instrumented_code);
    c.bench_function("WASM::instantiate_wasm_preloaded", |b| {
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
