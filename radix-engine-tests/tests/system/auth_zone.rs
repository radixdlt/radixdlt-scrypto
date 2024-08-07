use radix_common::prelude::*;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::CreateFrameError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::{SystemApi, ACTOR_REF_AUTH_ZONE};
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_test::prelude::*;

#[test]
fn should_not_be_able_to_move_auth_zone() {
    // Arrange
    const BLUEPRINT_NAME: &str = "MyBlueprint";
    const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<
            Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
            V: VmApi,
        >(
            &mut self,
            export_name: &str,
            input: &IndexedScryptoValue,
            api: &mut Y,
            _vm_api: &V,
        ) -> Result<IndexedScryptoValue, RuntimeError> {
            match export_name {
                "test" => {
                    let auth_zone_id = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE).unwrap();
                    let self_blueprint_id = api.actor_get_blueprint_id()?;
                    api.call_function(
                        self_blueprint_id.package_address,
                        self_blueprint_id.blueprint_name.as_str(),
                        "hi",
                        scrypto_encode(&Own(auth_zone_id)).unwrap(),
                    )?;
                }
                "hi" => {
                    return Ok(input.clone());
                }
                _ => {}
            }

            Ok(IndexedScryptoValue::from_typed(&()))
        }
    }
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![("test", "test", false), ("hi", "hi", false)],
        ),
    );

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_function(package_address, BLUEPRINT_NAME, "test", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(..))
            ))
        )
    });
}

#[test]
fn test_auth_zone_create_proof_of_all_for_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, 10)
        .create_proof_from_auth_zone_of_all(XRD, "proof")
        .drop_proof("proof")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_auth_zone_create_proof_of_all_for_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_non_fungibles(
            account,
            resource_address,
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
            ],
        )
        .create_proof_from_auth_zone_of_all(resource_address, "proof")
        .drop_proof("proof")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
// Here we are trying to exploit an issue, that was present in Radix Engine.
// Calls to owned components were possesing transaction processor's AuthZone,
// which effectively allowed to withdraw resources from the account that signed the
// transaction.
fn test_auth_zone_try_to_steal_from_account() {
    use radix_engine_tests::common::*;

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("steal"));

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "Steal", "instantiate", manifest_args!())
            .build(),
        vec![],
    );
    let steal_component_address = receipt.expect_commit_success().new_component_addresses()[0];

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                steal_component_address,
                "steal_from_account",
                manifest_args!(account),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
            err,
        ))) => err.fn_identifier.eq(&FnIdentifier {
            blueprint_id: BlueprintId::new(&ACCOUNT_PACKAGE, "Account"),
            ident: "withdraw".to_owned(),
        }),
        _ => false,
    });
}
