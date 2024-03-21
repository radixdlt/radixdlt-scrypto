use criterion::{criterion_group, criterion_main, Criterion};
use radix_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn get_bls_test_data() -> (Vec<u8>, Bls12381G1PublicKey, Bls12381G2Signature) {
    let msg = hash("Test").to_vec();
    let pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
    let signature = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1";
    let pk = Bls12381G1PublicKey::from_str(pk).unwrap();
    let signature = Bls12381G2Signature::from_str(signature).unwrap();
    (msg, pk, signature)
}

fn bench_bls_native(c: &mut Criterion) {
    let (msg, pk, signature) = get_bls_test_data();

    c.bench_function("wasm_vs_native::bls_native", |b| {
        b.iter(|| verify_bls12381_v1(&msg, &pk, &signature))
    });
}

fn bench_bls_native_via_wasm(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let (msg, pk, signature) = get_bls_test_data();

    c.bench_function("wasm_vs_native::bls_native_via_wasm", |b| {
        b.iter(|| {
            ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee(ledger.faucet_component(), 500u32)
                    .call_function(
                        package_address,
                        "CryptoScrypto",
                        "bls12381_v1_verify",
                        manifest_args!(msg.clone(), pk, signature),
                    )
                    .build(),
                vec![],
            )
        })
    });
}

fn bench_bls_wasm(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let (msg, pk, signature) = get_bls_test_data();

    c.bench_function("wasm_vs_native::bls_wasm", |b| {
        b.iter(|| {
            ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee(ledger.faucet_component(), 500u32)
                    .call_function(
                        package_address,
                        "CryptoScrypto",
                        "bls12381_v1_verify_in_wasm",
                        manifest_args!(msg.clone(), pk, signature),
                    )
                    .build(),
                vec![],
            )
        })
    });
}

criterion_group!(
    name = wasm_vs_native;
    config = Criterion::default().sample_size(10);
    targets = bench_bls_native, bench_bls_native_via_wasm, bench_bls_wasm
);
criterion_main!(wasm_vs_native);
