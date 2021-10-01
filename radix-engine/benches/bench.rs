#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

fn create_account<L: Ledger>(executor: &mut TransactionExecutor<L>) -> Address {
    let transaction1 = TransactionBuilder::new().new_account().build().unwrap();
    let receipt1 = executor.run(transaction1, false);
    assert!(receipt1.success);

    let account = receipt1.nth_component(0).unwrap();

    let transaction2 = TransactionBuilder::new()
        .mint_resource(1000.into(), RADIX_TOKEN)
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt2 = executor.run(transaction2, false);
    assert!(receipt2.success);

    account
}

fn bench_transfer(b: &mut Bencher) {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account1 = create_account(&mut executor);
    let account2 = create_account(&mut executor);

    b.iter(|| {
        let transaction = TransactionBuilder::new()
            .withdraw(1.into(), RADIX_TOKEN, account1)
            .deposit_all(account2)
            .build()
            .unwrap();
        let receipt = executor.run(transaction, false);
        assert!(receipt.success);
    });
}

benchmark_group!(radix_engine, bench_transfer);
benchmark_main!(radix_engine);
