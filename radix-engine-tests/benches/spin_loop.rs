use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use scrypto_unit::TestRunner;
use transaction::builder::ManifestBuilder;

fn bench_spin_loop(c: &mut Criterion) {
    // Set up environment.
    let mut test_runner = TestRunner::builder().without_trace().build();

    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee");
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 10u32.into())
                .call_method(test_runner.faucet_component(), "free", manifest_args!())
                .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                    builder.call_function(package_address, "Fee", "new", manifest_args!(bucket_id));
                    builder
                })
                .build(),
            vec![],
        )
        .expect_commit(true)
        .new_component_addresses()[0];

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .call_method(
            test_runner.faucet_component(),
            "lock_fee",
            manifest_args!(Decimal::from(10)),
        )
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", manifest_args!())
        .build();

    // Loop
    c.bench_function("SpinLoop::run", |b| {
        b.iter(|| {
            let receipt = test_runner.execute_manifest(manifest.clone(), vec![]);
            receipt.expect_commit_failure();
        })
    });
}

criterion_group!(spin_loop, bench_spin_loop);
criterion_main!(spin_loop);
