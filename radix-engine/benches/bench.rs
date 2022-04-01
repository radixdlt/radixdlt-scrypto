#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

fn bench_transfer(b: &mut Bencher) {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, account1) = executor.new_account();
    let (_, _, account2) = executor.new_account();
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account_by_amount(1.into(), RADIX_TOKEN, account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build(&[pk])
        .unwrap()
        .sign(&[sk]);

    b.iter(|| {
        let receipt = executor.validate_and_execute(&transaction).unwrap();
        assert!(receipt.result.is_ok());
    });
}

benchmark_group!(radix_engine, bench_transfer);
benchmark_main!(radix_engine);
