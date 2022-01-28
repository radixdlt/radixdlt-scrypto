#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

fn bench_transfer(b: &mut Bencher) {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0, false);
    let key1 = executor.new_public_key();
    let account1 = executor.new_account(key1);
    let key2 = executor.new_public_key();
    let account2 = executor.new_account(key2);
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(
            &ResourceAmount::Fungible {
                amount: 1.into(),
                resource_address: RADIX_TOKEN,
            },
            account1,
        )
        .call_method_with_all_resources(account2, "deposit_batch")
        .build(vec![key1])
        .unwrap();

    b.iter(|| {
        let receipt = executor.run(transaction.clone()).unwrap();
        assert!(receipt.error.is_none());
    });
}

benchmark_group!(radix_engine, bench_transfer);
benchmark_main!(radix_engine);
