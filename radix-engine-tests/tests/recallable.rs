use radix_engine::blueprints::resource::ResourceNativePackage;
use radix_engine::errors::{KernelError, RejectionError, RuntimeError, SystemModuleError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine::vm::{NativeVm, NativeVmV1, NativeVmV1Instance, VmInvoke};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::RESOURCE_CODE_ID;
use scrypto::prelude::FromPublicKey;
use scrypto_unit::*;
use std::ops::Sub;
use transaction::builder::ManifestBuilder;

#[test]
fn non_existing_vault_should_cause_error() {
    // Arrange
    let mut test_runner =
        TestRunnerBuilder::new().build_with_native_vm(CheckedGlobalAddressNativeVm::new());
    let (_, _, account) = test_runner.new_allocated_account();

    let non_existing_address = local_address(EntityType::InternalFungibleVault, 5);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .recall(non_existing_address, Decimal::one())
        .call_method(
            account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_rejection(|e| {
        e.eq(&RejectionError::ErrorBeforeFeeLoanRepaid(
            RuntimeError::KernelError(KernelError::InvalidReference(
                non_existing_address.as_node_id().clone(),
            )),
        ))
    });
}

#[test]
fn cannot_take_on_non_recallable_vault() {
    // Arrange
    let mut test_runner =
        TestRunnerBuilder::new().build_with_native_vm(CheckedGlobalAddressNativeVm::new());
    let (_, _, account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_fungible_resource(10u32.into(), 0u8, account);
    let vaults = test_runner.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .recall(
            InternalAddress::new_or_panic(vault_id.into()),
            Decimal::one(),
        )
        .call_method(
            account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                AuthError::Unauthorized { .. },
            ))
        )
    });
}

#[test]
fn can_take_on_recallable_vault() {
    // Arrange
    let mut test_runner =
        TestRunnerBuilder::new().build_with_native_vm(CheckedGlobalAddressNativeVm::new());
    let (_, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let recallable_token = test_runner.create_recallable_token(account);
    let vaults = test_runner.get_component_vaults(account, recallable_token);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .recall(
            InternalAddress::new_or_panic(vault_id.into()),
            Decimal::one(),
        )
        .call_method(
            other_account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    let original_account_amount = test_runner
        .get_component_resources(account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    let mut expected_amount: Decimal = 5u32.into();
    expected_amount = expected_amount.sub(Decimal::one());
    assert_eq!(expected_amount, original_account_amount);

    let other_amount = test_runner
        .get_component_resources(other_account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    assert_eq!(other_amount, Decimal::one());
}

#[test]
fn test_recall_on_internal_vault() {
    // Basic setup
    let mut test_runner =
        TestRunnerBuilder::new().build_with_native_vm(CheckedGlobalAddressNativeVm::new());
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/recall");

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_function(package_address, "RecallTest", "new", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let (component_address, _): (ComponentAddress, ResourceAddress) =
        receipt.expect_commit(true).output(1);

    // Recall
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_method(
                component_address,
                "recall_on_internal_vault",
                manifest_args!(),
            )
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidInvokeAccess)
        )
    });
}

#[test]
fn test_recall_on_received_direct_access_reference() {
    // Arrange
    let mut test_runner =
        TestRunnerBuilder::new().build_with_native_vm(CheckedGlobalAddressNativeVm::new());
    let (public_key, _, account) = test_runner.new_allocated_account();
    let recallable_token_address = test_runner.create_recallable_token(account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/recall");
    let vault_id = test_runner.get_component_vaults(account, recallable_token_address)[0];

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_function(
                package_address,
                "RecallTest",
                "recall_on_direct_access_ref",
                manifest_args!(InternalAddress::new_or_panic(vault_id.into())),
            )
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_recall_on_received_direct_access_reference_which_is_same_as_self() {
    // Arrange
    let mut test_runner =
        TestRunnerBuilder::new().build_with_native_vm(CheckedGlobalAddressNativeVm::new());
    let (public_key, _, account) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish("./tests/blueprints/recall");
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .call_function(package_address, "RecallTest", "new", manifest_args!())
            .build(),
        vec![],
    );
    let (component_address, recallable): (ComponentAddress, ResourceAddress) =
        receipt.expect_commit(true).output(1);
    let vault_id = test_runner.get_component_vaults(component_address, recallable)[0];

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .call_method(
                component_address,
                "recall_on_direct_access_ref_method",
                manifest_args!(InternalAddress::new_or_panic(vault_id.into())),
            )
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

/// Native VM which adds global address invariant checking on direct access methods
#[derive(Clone)]
pub struct CheckedGlobalAddressNativeVm {
    vm: NativeVmV1,
    resource_direct_access_methods: HashSet<String>,
}

impl CheckedGlobalAddressNativeVm {
    pub fn new() -> Self {
        let resource_direct_access_methods: HashSet<String> = ResourceNativePackage::definition()
            .blueprints
            .into_iter()
            .flat_map(|(_, def)| def.schema.functions.functions.into_iter())
            .filter_map(|(_, def)| {
                def.receiver.and_then(|i| {
                    if matches!(i.ref_types, RefTypes::DIRECT_ACCESS) {
                        Some(def.export)
                    } else {
                        None
                    }
                })
            })
            .collect();

        Self {
            vm: NativeVmV1,
            resource_direct_access_methods,
        }
    }
}

impl NativeVm for CheckedGlobalAddressNativeVm {
    type Instance = CheckInvariantsNativeVmInstance;

    fn create_instance(
        &self,
        package_address: &PackageAddress,
        code: &[u8],
    ) -> Result<CheckInvariantsNativeVmInstance, RuntimeError> {
        let instance = self.vm.create_instance(package_address, code)?;
        Ok(CheckInvariantsNativeVmInstance {
            instance,
            resource_direct_access_methods: self.resource_direct_access_methods.clone(),
        })
    }
}

pub struct CheckInvariantsNativeVmInstance {
    instance: NativeVmV1Instance,
    resource_direct_access_methods: HashSet<String>,
}

impl VmInvoke for CheckInvariantsNativeVmInstance {
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match self.instance.native_package_code_id {
            RESOURCE_CODE_ID => {
                if self.resource_direct_access_methods.contains(export_name) {
                    api.actor_get_global_address()
                        .expect_err("Direct method calls should never have global address");
                }
            }
            _ => {}
        }
        self.instance.invoke(export_name, input, api)
    }
}
