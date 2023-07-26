use radix_engine::{
    errors::{CallFrameError, KernelError, RuntimeError, SystemError},
    kernel::call_frame::PassMessageError,
    types::*,
};
use scrypto_unit::*;
use transaction::prelude::*;

fn setup_component(test_runner: &mut DefaultTestRunner) -> ComponentAddress {
    let package_address = test_runner.compile_and_publish("./tests/blueprints/data_validation");

    let setup_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "DataValidation", "new", manifest_args!())
        .build();
    let setup_receipt = test_runner.execute_manifest(setup_manifest, vec![]);
    setup_receipt.expect_commit(true).new_component_addresses()[0]
}

fn sink_account() -> ComponentAddress {
    ComponentAddress::virtual_account_from_public_key(&Secp256k1PublicKey([0; 33]))
}

fn create_manifest_with_middle(
    component_address: ComponentAddress,
    constructor: ManifestConstructor,
) -> TransactionManifestV1 {
    ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_from_worktop(XRD, dec!(1), "bucket")
        .take_from_worktop(XRD, dec!("0"), "empty_bucket")
        .take_from_worktop(XRD, dec!(1), "proof_bucket")
        .create_proof_from_bucket_of_all("proof_bucket", "proof")
        .with_name_lookup(|builder, lookup| {
            constructor(
                builder,
                component_address,
                lookup.bucket("empty_bucket"),
                lookup.bucket("bucket"),
                lookup.proof("proof"),
            )
        })
        .return_to_worktop("proof_bucket")
        .try_deposit_batch_or_abort(sink_account())
        .build()
}

type ManifestConstructor = fn(
    builder: ManifestBuilder,
    component: ComponentAddress,
    empty_bucket: ManifestBucket,
    full_bucket: ManifestBucket,
    proof: ManifestProof,
) -> ManifestBuilder;

/// This test just checks that the manifest constructor and DataValidation components work right -
/// to ensure the other tests in this file are valid tests.
#[test]
fn valid_transactions_can_be_committed() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest_with_default_handling = create_manifest_with_middle(
        component_address,
        |builder, _, empty_bucket, full_bucket, proof| {
            builder
                .return_to_worktop(empty_bucket)
                .return_to_worktop(full_bucket)
                .drop_proof(proof)
        },
    );

    let manifest_using_component = create_manifest_with_middle(
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
                    "accept_non_empty_bucket",
                    manifest_args!(full_bucket),
                )
                .call_method(component_address, "accept_proof", manifest_args!(proof))
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
fn cannot_pass_bucket_for_proof_argument() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = create_manifest_with_middle(
        component_address,
        |builder, component_address, empty_bucket, full_bucket, proof| {
            builder
                .return_to_worktop(empty_bucket)
                .call_method(
                    component_address,
                    "accept_proof",
                    manifest_args!(full_bucket),
                )
                .drop_proof(proof)
        },
    );

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("DataValidation"))
}

#[test]
fn cannot_pass_proof_for_bucket_argument() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = create_manifest_with_middle(
        component_address,
        |builder, component_address, empty_bucket, full_bucket, proof| {
            builder
                .return_to_worktop(empty_bucket)
                .return_to_worktop(full_bucket)
                .call_method(
                    component_address,
                    "accept_empty_bucket",
                    manifest_args!(proof),
                )
        },
    );

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("DataValidation"))
}

#[test]
fn cannot_return_proof_for_bucket() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "return_proof_for_bucket",
            manifest_args!(),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("PayloadValidationError"))
}

#[test]
fn cannot_return_bucket_for_proof() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "return_bucket_for_proof",
            manifest_args!(),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("PayloadValidationError"))
}

#[test]
fn cannot_create_object_with_mismatching_data() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/data_validation");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "DataValidation",
            "create_object_with_illegal_data",
            manifest_args!(),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("DataValidation"))
}

#[test]
fn cannot_update_substate_with_mismatching_data() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "update_state_with_illegal_data",
            manifest_args!(),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("DataValidation"))
}

/// Note that payload validation after pushing call frame.
#[test]
fn pass_own_as_reference_trigger_move_error_rather_than_payload_validation_error() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "can_pass_own_as_reference",
            manifest_args!(),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::PassMessageError(PassMessageError::StableRefNotFound(_))
            ))
        )
    });
}

#[test]
fn test_receive_reference_of_specific_blueprint() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "accept_custom_reference",
            manifest_args!(XRD),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn test_receive_reference_not_of_specific_blueprint() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let component_address = setup_component(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "accept_custom_reference",
            manifest_args!(PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("DataValidation"))
}

#[test]
fn vec_of_u8_underflow_should_not_cause_panic() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("tests/blueprints/data_validation");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "VecOfU8Underflow",
            "write_vec_u8_underflow_to_key_value_store",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::InvalidSubstateWrite(e)) 
            if e.eq("TraversalError(DecodeError(BufferUnderflow { required: 99999993, remaining: 1048569 })) occurred at byte offset 7-7 and value path Array->[ERROR] DecodeError(BufferUnderflow { required: 99999993, remaining: 1048569 })") => true,
        _ => false,
    })
}
