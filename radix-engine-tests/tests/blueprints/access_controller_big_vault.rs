use radix_common::prelude::*;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmApi, VmInvoke};
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::prelude::*;
use scrypto_test::prelude::{LedgerSimulator, LedgerSimulatorBuilder};
use std::iter;

#[test]
pub fn should_be_able_to_withdraw_from_maximum_vault_size_access_controller() {
    // Arrange
    let (mut ledger, access_controller) = arrange_access_controller_big_vault();

    // Act
    let (key, _, account) = ledger.new_account(false);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
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
            .deposit_entire_worktop(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
pub fn should_be_able_to_create_proof_from_maximum_vault_access_controller() {
    // Arrange
    let (mut ledger, access_controller) = arrange_access_controller_big_vault();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(ledger.faucet_component(), 500u32)
            .call_method(
                access_controller,
                ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
                manifest_args!(),
            )
            .build(),
        vec![],
    );

    // Asert
    receipt.expect_commit_success();
}

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
                            owner_role: Default::default(),
                            track_total_supply: Default::default(),
                            non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(),
                            resource_roles: Default::default(),
                            metadata: Default::default(),
                            address_reservation: Default::default(),
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
                    ACCESS_CONTROLLER_CREATE_IDENT,
                    scrypto_encode(&AccessControllerCreateInput {
                        controlled_asset: bucket,
                        rule_set: RuleSet {
                            primary_role: AccessRule::AllowAll,
                            recovery_role: AccessRule::AllowAll,
                            confirmation_role: AccessRule::AllowAll,
                        },
                        timed_recovery_delay_in_minutes: None,
                        address_reservation: None,
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
    LedgerSimulator<OverridePackageCode<TestInvoke>, InMemorySubstateDatabase>,
    ComponentAddress,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
        .build();
    let package_address = ledger.publish_native_package(
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
            let receipt = ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee(ledger.faucet_component(), 500u32)
                    .call_function(
                        package_address,
                        BLUEPRINT_NAME,
                        "new",
                        manifest_args!(bucket_size),
                    )
                    .build(),
                vec![],
            );
            let commit = receipt.expect_commit_ignore_outcome();
            if !commit.outcome.is_success() {
                bucket_size -= 1;
                break;
            }
            bucket_size += 100;
        }

        // Decrement failure bucket size by 1 until success
        loop {
            let receipt = ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee(ledger.faucet_component(), 500u32)
                    .call_function(
                        package_address,
                        BLUEPRINT_NAME,
                        "new",
                        manifest_args!(bucket_size),
                    )
                    .build(),
                vec![],
            );
            let commit = receipt.expect_commit_ignore_outcome();
            if commit.outcome.is_success() {
                let access_controller =
                    receipt.expect_commit_success().new_component_addresses()[0];
                return (ledger, access_controller);
            }
            bucket_size -= 1;
        }
    }
}
