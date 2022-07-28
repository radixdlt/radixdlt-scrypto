#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::transaction::TransactionReceipt;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn run_manifest<F>(f: F) -> TransactionReceipt
where
    F: FnOnce(ComponentAddress) -> TransactionManifest,
{
    // Basic setup
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Publish package and instantiate component
    let package_address = test_runner.extract_and_publish_package("fee");
    let receipt1 = test_runner.execute_manifest(
        ManifestBuilder::new(Network::LocalSimulator)
            .pay_fee(10.into(), account)
            .withdraw_from_account_by_amount(1000.into(), RADIX_TOKEN, account)
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_function(package_address, "Fee", "new", to_struct!(Bucket(bucket_id)));
                builder
            })
            .build(),
        vec![public_key],
    );
    let component_address = receipt1.new_component_addresses[0];

    // Run the provided manifest
    let manifest = f(component_address);
    test_runner.execute_manifest(manifest, vec![])
}

#[test]
fn should_succeed_when_fee_is_paid() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(Network::LocalSimulator)
            .call_method(component_address, "pay_fee", to_struct!(Decimal::from(10)))
            .build()
    });

    receipt.expect_success();
}

#[test]
fn should_be_rejected_when_no_fee_is_paid() {
    let receipt = run_manifest(|_| ManifestBuilder::new(Network::LocalSimulator).build());

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_insufficient_balance() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(Network::LocalSimulator)
            .call_method(
                component_address,
                "pay_fee_with_empty_vault",
                to_struct!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_non_xrd() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(Network::LocalSimulator)
            .call_method(
                component_address,
                "pay_fee_with_doge",
                to_struct!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_system_loan_is_not_fully_repaid() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(Network::LocalSimulator)
            .call_method(
                component_address,
                "pay_fee",
                to_struct!(Decimal::from_str("0.001").unwrap()), // = 1000 cost units
            )
            .build()
    });

    receipt.expect_rejection();
}
