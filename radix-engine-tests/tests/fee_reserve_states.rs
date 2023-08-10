use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_fee_states() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee_reserve_states");

    // Run test case
    let fee_locked = dec!(500);
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, fee_locked)
            .call_function(
                package_address,
                "FeeReserveChecker",
                "check",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let (cost_unit_limit, cost_unit_price, tip_percentage, remaining_fee_balance) = receipt
        .expect_commit_success()
        .output::<(u32, Decimal, u32, Decimal)>(1);
    assert_eq!(cost_unit_limit, EXECUTION_COST_UNIT_LIMIT);
    assert_eq!(
        cost_unit_price,
        Decimal::try_from(EXECUTION_COST_UNIT_PRICE_IN_XRD).unwrap()
    );
    assert_eq!(tip_percentage, DEFAULT_TIP_PERCENTAGE as u32);
    // At the time checking fee balance, it should be still using system loan. This is because
    // loan is designed to be slightly more than what it takes to `lock_fee` from a component.
    // Therefore, the balance should be between `fee_locked` and `fee_locked + loan_in_xrd`.
    let loan_in_xrd = receipt.effective_execution_cost_unit_price() * EXECUTION_COST_UNIT_LOAN
        + receipt.effective_finalization_cost_unit_price() * FINALIZATION_COST_UNIT_LOAN;
    assert!(fee_locked < remaining_fee_balance);
    assert!(remaining_fee_balance < fee_locked + loan_in_xrd);
}
