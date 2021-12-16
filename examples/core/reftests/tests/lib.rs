use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_new() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("reftests"));

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "Hello", "new", vec![], None)
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, false).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);
    let _component = receipt1.component(0).unwrap();
}

macro_rules! mktest {
    ($name:ident, $namestr:literal, $amount:literal) => {
        #[test]
        fn $name() {
            // Set up environment.
            let mut ledger = InMemoryLedger::with_bootstrap();
            let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
            let key = executor.new_public_key();
            let account = executor.new_account(key);
            let package = executor.publish_package(include_code!("reftests"));

            // Test the `new` function.
            let transaction1 = TransactionBuilder::new(&executor)
                .call_function(package, "Hello", "new", vec![], None)
                .drop_all_bucket_refs()
                .deposit_all_buckets(account)
                .build(vec![key])
                .unwrap();
            let receipt1 = executor.run(transaction1, false).unwrap();
            println!("{:?}\n", receipt1);
            assert!(receipt1.success);
            let component = receipt1.component(0).unwrap();

            // Test the `free_token` method.
            let transaction2 = TransactionBuilder::new(&executor)
                .call_method(component, $namestr, vec![format!($amount)], Some(account))
                .drop_all_bucket_refs()
                .deposit_all_buckets(account)
                .build(vec![key])
                .unwrap();
            let receipt2 = executor.run(transaction2, true).unwrap();
            println!("{:?}\n", receipt2);
            assert!(receipt2.success);
        }
    };
}

mktest!(test_show, "show", "100");
mktest!(test_show_amount, "show_amount", "100");
mktest!(test_show_a, "show_a", "100");
mktest!(test_show_b, "show_b", "100");
mktest!(test_have, "have", "100");
mktest!(test_have_b, "have_b", "100");
mktest!(test_have_b2, "have_b2", "100");
mktest!(test_have_c, "have_c", "100");
mktest!(test_have_c2, "have_c2", "100");
mktest!(test_have_d, "have_d", "100");
mktest!(test_have_e, "have_e", "100");
mktest!(test_show_not_fixed, "show_not_fixed", "100");

mktest!(test_show_0, "show", "0");
mktest!(test_show_amount_0, "show_amount", "0");
mktest!(test_show_a_0, "show_a", "0");
mktest!(test_show_b_0, "show_b", "0");
mktest!(test_have_0, "have", "0");
mktest!(test_have_b_0, "have_b", "0");
mktest!(test_have_b2_0, "have_b2", "0");
mktest!(test_have_c_0, "have_c", "0");
mktest!(test_have_c2_0, "have_c2", "0");
mktest!(test_have_d_0, "have_d", "0");
mktest!(test_have_e_0, "have_e", "0");
mktest!(test_show_not_fixed_0, "show_not_fixed", "0");
