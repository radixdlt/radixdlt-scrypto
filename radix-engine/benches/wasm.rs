use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::model::extract_abi;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::WasmEngine;
use radix_engine::wasm::WasmValidator;

fn bench_wasm_validation(c: &mut Criterion) {
    let code = include_bytes!("../../assets/account.wasm");
    let abi = extract_abi(code).unwrap();

    c.bench_function("WASM validation", |b| {
        b.iter(|| WasmValidator::default().validate(code, &abi))
    });
}

fn bench_wasm_instantiation(c: &mut Criterion) {
    let code = include_bytes!("../../assets/account.wasm");
    c.bench_function("WASM instantiation", |b| {
        b.iter(|| {
            let mut engine = DefaultWasmEngine::new();
            engine.instantiate(code);
        })
    });
}

fn bench_wasm_instantiation_pre_loaded(c: &mut Criterion) {
    let code = include_bytes!("../../assets/account.wasm");
    let mut engine = DefaultWasmEngine::new();
    engine.instantiate(code);
    c.bench_function("WASM instantiation (pre-loaded)", |b| {
        b.iter(|| {
            engine.instantiate(code);
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
