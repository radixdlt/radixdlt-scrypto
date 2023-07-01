use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_fee_states() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee_reserve_states");

    // Run test case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_function(
                package_address,
                "FeeReserveChecker",
                "check",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let output: (u32, Decimal, u32, Decimal) = receipt.expect_commit_success().output(1);
    assert_eq!(output.0, DEFAULT_COST_UNIT_LIMIT);
    assert_eq!(
        output.1,
        Decimal::try_from(DEFAULT_COST_UNIT_PRICE_IN_XRD).unwrap()
    );
    assert_eq!(output.2, DEFAULT_TIP_PERCENTAGE as u32);
    // At the time checking fee balance, it should be still using system loan. This is because
    // loan is designed to be slightly more than what it takes to `lock_fee` from a component.
    // Therefore, the balance should be between `500` and `500 + loan_in_xrd`.
    assert!(
        output.3 > dec!(500)
            && output.3
                < dec!(500)
                    + Decimal::from(DEFAULT_SYSTEM_LOAN)
                        * (Decimal::try_from(DEFAULT_COST_UNIT_PRICE_IN_XRD).unwrap()
                            * (dec!(1) + Decimal::from(DEFAULT_TIP_PERCENTAGE) / dec!(100)))
    );
}
