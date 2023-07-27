use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::{TestRunner, TestRunnerBuilder};
use std::iter;
use transaction::prelude::*;

#[test]
pub fn should_be_able_to_withdraw_from_maximum_vault_size_access_controller() {
    // Arrange
    let (mut test_runner, access_controller) = arrange_access_controller_big_vault();

    let (key, _, account) = test_runner.new_account(false);
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_method(
                access_controller,
                ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT,
                manifest_args!(),
            )
            .call_method(
                access_controller,
                ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
                manifest_args!(),
            )
            .deposit_batch(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&key)],
    );

    receipt.expect_commit_success();
}

#[test]
pub fn should_be_able_to_create_proof_from_maximum_vault_access_controller() {
    // Arrange
    let (mut test_runner, access_controller) = arrange_access_controller_big_vault();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_method(
                access_controller,
                ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
                manifest_args!(),
            )
            .build(),
        vec![],
    );

    receipt.expect_commit_success();
}

const BLUEPRINT_NAME: &str = "MyBlueprint";
const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;
#[derive(Clone)]
struct TestInvoke;
impl VmInvoke for TestInvoke {
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match export_name {
            "new" => {
                let size: (usize,) = input.as_typed().unwrap();
                let entries = iter::repeat((ScryptoValue::Tuple { fields: vec![] },))
                    .take(size.0)
                    .collect();
                let result = api.call_function(
                    RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT,
                    scrypto_encode(
                        &NonFungibleResourceManagerCreateRuidWithInitialSupplyInput {
                            entries,
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                )?;
                let result: NonFungibleResourceManagerCreateRuidWithInitialSupplyOutput =
                    scrypto_decode(&result).unwrap();
                let bucket = result.1;

                api.call_function(
                    ACCESS_CONTROLLER_PACKAGE,
                    ACCESS_CONTROLLER_BLUEPRINT,
                    ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
                    scrypto_encode(&AccessControllerCreateGlobalInput {
                        controlled_asset: bucket,
                        rule_set: RuleSet {
                            primary_role: AccessRule::AllowAll,
                            recovery_role: AccessRule::AllowAll,
                            confirmation_role: AccessRule::AllowAll,
                        },
                        timed_recovery_delay_in_minutes: None,
                    })
                    .unwrap(),
                )?;
            }
            _ => {}
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}

fn arrange_access_controller_big_vault() -> (
    TestRunner<OverridePackageCode<TestInvoke>, InMemorySubstateDatabase>,
    ComponentAddress,
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = test_runner.publish_native_package(
        CUSTOM_PACKAGE_CODE_ID,
        PackageDefinition::new_functions_only_test_definition(
            BLUEPRINT_NAME,
            vec![("new", "new", false)],
        ),
    );

    // Create the largest access controller non fungible vault possible
    {
        // Start by incrementing non fungible bucket size by 100 until failure
        let mut bucket_size = 100usize;
        loop {
            let receipt = test_runner.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee(test_runner.faucet_component(), 500u32)
                    .call_function(
                        package_address,
                        BLUEPRINT_NAME,
                        "new",
                        manifest_args!(bucket_size),
                    )
                    .build(),
                vec![],
            );
            let commit = receipt.expect_commit_ignore_result();
            if !commit.outcome.is_success() {
                bucket_size -= 1;
                break;
            }
            bucket_size += 100;
        }

        // Decrement failure bucket size by 1 until success
        loop {
            let receipt = test_runner.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee(test_runner.faucet_component(), 500u32)
                    .call_function(
                        package_address,
                        BLUEPRINT_NAME,
                        "new",
                        manifest_args!(bucket_size),
                    )
                    .build(),
                vec![],
            );
            let commit = receipt.expect_commit_ignore_result();
            if commit.outcome.is_success() {
                let access_controller =
                    receipt.expect_commit_success().new_component_addresses()[0];
                return (test_runner, access_controller);
            }
            bucket_size -= 1;
        }
    }
}
