#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

fn bench_transfer(b: &mut Bencher) {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (key1, account1) = executor.new_public_key_with_account();
    let (_, account2) = executor.new_public_key_with_account();
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&ResourceSpecifier::Amount(1.into(), RADIX_TOKEN), account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build(vec![key1])
        .unwrap();

    b.iter(|| {
        let receipt = executor.run(transaction.clone()).unwrap();
        assert!(receipt.result.is_ok());
    });
}

benchmark_group!(radix_engine, bench_transfer);
benchmark_main!(radix_engine);
