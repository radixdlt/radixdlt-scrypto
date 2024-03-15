use radix_common::prelude::*;
use radix_engine::blueprints::resource::ProofError;
use radix_engine::errors::{
    ApplicationError, CallFrameError, KernelError, RuntimeError, SystemError,
};
use radix_engine::kernel::call_frame::OpenSubstateError;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn create_payload_of_depth(n: usize) -> Vec<u8> {
    assert!(n >= 1);

    // initial value with depth = 1
    let mut value = ScryptoValue::Array {
        element_value_kind: ScryptoValueKind::Array,
        elements: vec![],
    };
    for _ in 1..n {
        // increase depth by 1
        value = ScryptoValue::Array {
            element_value_kind: ScryptoValueKind::Array,
            elements: vec![value],
        }
    }
    scrypto_encode_with_depth_limit(&value, 128).unwrap()
}

#[test]
fn test_write_kv_store_entry_within_depth_limit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("scrypto_env"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MaxSborDepthTest",
            "write_kv_store_entry_with_depth",
            manifest_args!(create_payload_of_depth(KEY_VALUE_STORE_PAYLOAD_MAX_DEPTH)),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_write_kv_store_entry_exceeding_depth_limit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("scrypto_env"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MaxSborDepthTest",
            "write_kv_store_entry_with_depth",
            manifest_args!(create_payload_of_depth(
                KEY_VALUE_STORE_PAYLOAD_MAX_DEPTH + 1
            )),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| format!("{:?}", e).contains("MaxDepthExceeded"))
}

#[test]
fn test_pop_empty_auth_zone() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("scrypto_env"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "LocalAuthZoneTest",
            "pop_empty_auth_zone",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    assert_eq!(
        receipt.expect_commit_success().output::<Option<Proof>>(1),
        None
    );
}

#[test]
fn test_create_signature_proof() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _priv_key, _account) = ledger.new_account(true);
    let package_address = ledger.publish_package_simple(PackageLoader::get("scrypto_env"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "LocalAuthZoneTest",
            "create_signature_proof",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ProofError(
                ProofError::EmptyProofNotAllowed
            ))
        )
    });
}

#[test]
fn should_not_be_able_to_node_create_with_invalid_blueprint() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("scrypto_env"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ScryptoEnvTest",
            "create_node_with_invalid_blueprint",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::BlueprintDoesNotExist(..)) => true,
        _ => false,
    });
}

#[test]
fn should_not_be_able_to_open_mut_substate_twice_if_object_in_heap() {
    should_not_be_able_to_open_mut_substate_twice(true);
}

#[test]
fn should_not_be_able_to_open_mut_substate_twice_if_object_globalized() {
    should_not_be_able_to_open_mut_substate_twice(false);
}

fn should_not_be_able_to_open_mut_substate_twice(heap: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("scrypto_env"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ScryptoEnvTest",
            "create_and_open_mut_substate_twice",
            manifest_args!(heap),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::OpenSubstateError(OpenSubstateError::SubstateLocked(..)),
        )) => true,
        _ => false,
    });
}

#[test]
fn should_be_able_to_bech32_encode_address() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("scrypto_env"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ScryptoEnvTest",
            "bech32_encode_address",
            manifest_args!(FAUCET),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();
    let _bech32_encoded: String = result.output(1);
}
