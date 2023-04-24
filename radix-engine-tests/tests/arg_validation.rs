use radix_engine::types::*;
use scrypto_unit::*;
use transaction::{builder::ManifestBuilder, model::TransactionManifest};

fn setup_component(test_runner: &mut TestRunner) -> ComponentAddress {
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arg_validation");

    let setup_manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(package_address, "ArgValidation", "new", manifest_args!())
        .build();
    let setup_receipt = test_runner.execute_manifest(setup_manifest, vec![]);
    setup_receipt.expect_commit(true).new_component_addresses()[0]
}

fn sink_account() -> ComponentAddress {
    ComponentAddress::virtual_account_from_public_key(&EcdsaSecp256k1PublicKey([0; 33]))
}

fn create_manifest_with_middle(
    test_runner: &mut TestRunner,
    component_address: ComponentAddress,
    constructor: ManifestConstructor,
) -> TransactionManifest {
    ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
        .take_from_worktop_by_amount(dec!("1"), RADIX_TOKEN, |builder, bucket| {
            builder.take_from_worktop_by_amount(dec!("0"), RADIX_TOKEN, |builder, empty_bucket| {
                builder.take_from_worktop_by_amount(
                    dec!("1"),
                    RADIX_TOKEN,
                    |builder, proof_bucket| {
                        builder.create_proof_from_bucket(&proof_bucket, |builder, proof| {
                            constructor(builder, component_address, empty_bucket, bucket, proof);
                            builder.return_to_worktop(proof_bucket)
                        })
                    },
                )
            })
        })
        .call_method(
            sink_account(),
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build()
}

type ManifestConstructor = fn(
    builder: &mut ManifestBuilder,
    component: ComponentAddress,
    empty_bucket: ManifestBucket,
    full_bucket: ManifestBucket,
    proof: ManifestProof,
);

/// This test just checks that the manifest constructor and ArgValidation components work right -
/// to ensure the other tests in this file are valid tests.
#[test]
fn valid_transactions_can_be_committed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest_with_default_handling = create_manifest_with_middle(
        &mut test_runner,
        component_address,
        |builder, _, empty_bucket, full_bucket, proof| {
            builder
                .return_to_worktop(empty_bucket)
                .return_to_worktop(full_bucket)
                .drop_proof(proof);
        },
    );

    let manifest_using_component = create_manifest_with_middle(
        &mut test_runner,
        component_address,
        |builder, component_address, empty_bucket, full_bucket, proof| {
            builder
                .call_method(
                    component_address,
                    "accept_empty_bucket",
                    manifest_args!(empty_bucket),
                )
                .call_method(
                    component_address,
                    "accept_and_return_bucket",
                    manifest_args!(full_bucket),
                )
                .call_method(component_address, "accept_proof", manifest_args!(proof));
        },
    );

    // Assert
    test_runner
        .execute_manifest(manifest_with_default_handling, vec![])
        .expect_commit_success();
    test_runner
        .execute_manifest(manifest_using_component, vec![])
        .expect_commit_success();
}

#[test]
#[ignore = "This test is ignored until we check the entity type at validation time"]
fn cannot_pass_bucket_for_proof_argument() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = create_manifest_with_middle(
        &mut test_runner,
        component_address,
        |builder, component_address, empty_bucket, full_bucket, proof| {
            builder
                .return_to_worktop(empty_bucket)
                .call_method(
                    component_address,
                    "accept_proof",
                    manifest_args!(full_bucket),
                )
                .drop_proof(proof);
        },
    );

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("ScryptoInputSchemaNotMatch"))
}

#[test]
#[ignore = "This test is ignored until we check the entity type at validation time"]
fn cannot_pass_proof_for_bucket_argument() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = create_manifest_with_middle(
        &mut test_runner,
        component_address,
        |builder, component_address, empty_bucket, full_bucket, proof| {
            builder
                .return_to_worktop(empty_bucket)
                .return_to_worktop(full_bucket)
                .call_method(
                    component_address,
                    "accept_empty_bucket",
                    manifest_args!(proof),
                );
        },
    );

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("ScryptoInputSchemaNotMatch"))
}
