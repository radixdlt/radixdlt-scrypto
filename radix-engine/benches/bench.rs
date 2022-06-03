#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::engine::TransactionExecutor;
use radix_engine::ledger::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::RADIX_TOKEN;
use transaction::builder::ManifestBuilder;

fn bench_transfer(b: &mut Bencher) {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, false);
    let (pk, sk, account1) = executor.new_account();
    let (_, _, account2) = executor.new_account();
    let transaction = ManifestBuilder::new()
        .withdraw_from_account_by_amount(1.into(), RADIX_TOKEN, account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build();
    let signers = vec![pk];

    b.iter(|| {
        let receipt = executor.execute(&transaction).unwrap();
        receipt.result.expect("It should work");
    });
}

benchmark_group!(radix_engine, bench_transfer);
benchmark_main!(radix_engine);
