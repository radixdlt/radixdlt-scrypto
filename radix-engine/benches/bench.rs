#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

fn bench_transfer(b: &mut Bencher) {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account1 = executor.create_account();
    let account2 = executor.create_account();
    let transaction = TransactionBuilder::new(&executor)
        .withdraw(1.into(), RADIX_TOKEN, account1)
        .deposit_all(account2)
        .build(Vec::new())
        .unwrap();

    b.iter(|| {
        let receipt = executor.run(transaction.clone(), false);
        assert!(receipt.success);
    });
}

benchmark_group!(radix_engine, bench_transfer);
benchmark_main!(radix_engine);
