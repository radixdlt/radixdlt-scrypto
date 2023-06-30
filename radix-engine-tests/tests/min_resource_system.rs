use radix_engine::types::*;
use scrypto::prelude::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

enum ExpectedBehavior {
    Success,
    Failure(&'static str),
}

fn test_mini_resource_system(test_case: &str, expected: ExpectedBehavior) {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address =
        test_runner.compile_and_publish("./tests/blueprints/mini_resource_system");

    // Run test case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_function(package_address, "MiniUser", test_case, manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    match expected {
        ExpectedBehavior::Success => {
            receipt.expect_commit_success();
        }
        ExpectedBehavior::Failure(msg) => {
            let error_message = receipt
                .expect_commit_failure()
                .outcome
                .expect_failure()
                .to_string();
            if !error_message.contains(msg) {
                panic!("Expected {}, but was {}", msg, error_message);
            }
        }
    };
}

// Note that we may decide to disallow borrowed reference in Scrypto, then
// many of the tests here will fail.

#[test]
pub fn test_create_bucket_proof_and_do_nothing() {
    test_mini_resource_system(
        "create_bucket_proof_and_do_nothing",
        ExpectedBehavior::Failure("NodeOrphaned"),
    );
}

#[test]
pub fn test_create_bucket_proof_and_query_amount() {
    test_mini_resource_system(
        "create_bucket_proof_and_query_amount",
        ExpectedBehavior::Success,
    );
}

#[test]
pub fn test_create_bucket_proof_and_drop_proof_and_drop_bucket() {
    test_mini_resource_system(
        "create_bucket_proof_and_drop_proof_and_drop_bucket",
        ExpectedBehavior::Success,
    );
}

#[test]
pub fn test_create_bucket_proof_and_drop_bucket_and_drop_proof() {
    test_mini_resource_system(
        "create_bucket_proof_and_drop_bucket_and_drop_proof",
        ExpectedBehavior::Failure("NodeBorrowed"),
    );
}

#[test]
pub fn test_create_bucket_proof_and_return_both() {
    test_mini_resource_system(
        "create_bucket_proof_and_return_both",
        ExpectedBehavior::Success,
    );
}

#[test]
pub fn test_create_proof_and_drop_the_bucket_in_another_frame() {
    test_mini_resource_system(
        "create_proof_and_drop_the_bucket_in_another_frame",
        ExpectedBehavior::Failure("NodeBorrowed"),
    );
}

#[test]
pub fn test_create_proof_and_drop_the_proof_in_another_frame() {
    test_mini_resource_system(
        "create_proof_and_drop_the_proof_in_another_frame",
        ExpectedBehavior::Success,
    );
}
