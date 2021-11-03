use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

/// NOTE - THE BELOW IS NOT POSSIBLE AND DOES NOT COMPILE - IT'S PSEUDO CODE FOR DEMONSTRATION
/// OF WHAT A !create_transaction_in_code MACRO MIGHT BE NICE (IF IT'S EVEN POSSIBLE)

#[test]
fn test1_via_separate_transactions_to_ledger() {
    // Set up environment.
    let mut (ledger, test_executor, account) = TestHelper::create_test_executor_with_default_account();

    let collateral_token = RADIX_TOKEN;

    // The vague idea is that, upon exporting ABI, we actually create:
    // - Two types for use inside the engine, as per currently
    //   - SyntheticPool
    //   - SyntheticPoolBluePrint
    // - Two types for use in tests:
    //   - SyntheticPoolOnLedger
    //   - SyntheticPoolBluePrintOnLedger
    // - A SyntheticPoolTestHelper which lets you create a SyntheticPoolOnLedger and SyntheticPoolBluePrintOnLedger
    //
    // In tests, you can use macros such as !create_method_call which will
    // convert calls in SyntheticPoolOnLedger to the relevant transaction

    let price_oracle: PriceOracleOnLedger = PriceOracleTestHelper::Create(test_executor);

    let (blueprint_address, synthetic_pool_blueprint: SyntheticPoolBlueprintOnLedger) = !deploy()

    let (new_pool: SyntheticPoolOnLedger, synthetic_tokens: BucketOnLedger) = TestTransaction::new(test_executor)
    .build_from_on_ledger_references(!create_transaction_in_code(
        // Returns a struct with returned values (output), as well as EG created resoures.
        let pool_creation_result = synthetic_pool_blueprint.new(price_oracle, collateral_token, 400000000);
        
        let newly_created_pool: SyntheticPoolOnLedger = pool_creation_result.output();

        let synthetic_tokens: BucketInTransaction = newly_created_pool.mint_synthetic(
            "Forex",
            "USD",
            400000000,
            BucketReference(1.into(), collateral_token)
        ).output()

        synthetic_tokens.send_to(account_on_ledger)

        (newly_created_pool, synthetic_tokens)
    ))
    .run()
    // The .run() outputs a struct with the transaction results, and lets you get the final output of the last method call
    .output();

}
