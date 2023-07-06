use radix_engine::blueprints::resource::ResourceNativePackage;
use radix_engine::errors::{NativeRuntimeError, RuntimeError, SystemError, VmError};
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::*;
use radix_engine::vm::{NativeVm, NativeVmV1, NativeVmV1Instance, VmInvoke};
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, PackageDefinition,
    PackagePublishNativeInput, PackagePublishNativeManifestInput, PACKAGE_BLUEPRINT,
    RESOURCE_CODE_ID,
};
use sbor::basic_well_known_types::ANY_ID;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;
use transaction::prelude::DynamicPackageAddress;
use transaction::validation::ManifestIdAllocator;

#[derive(Clone)]
pub struct TestNativeVm<C: VmInvoke + Clone> {
    vm: NativeVmV1,
    custom_invoke: C,
}

impl<C: VmInvoke + Clone> TestNativeVm<C> {
    pub fn new(custom_invoke: C) -> Self {
        Self {
            vm: NativeVmV1,
            custom_invoke,
        }
    }
}

impl<C: VmInvoke + Clone> NativeVm for TestNativeVm<C> {
    type Instance = TestNativeVmInstance<C>;

    fn create_instance(
        &self,
        package_address: &PackageAddress,
        code: &[u8],
    ) -> Result<TestNativeVmInstance<C>, RuntimeError> {
        let native_package_code_id = {
            let code: [u8; 8] = match code.clone().try_into() {
                Ok(code) => code,
                Err(..) => {
                    return Err(RuntimeError::VmError(VmError::Native(
                        NativeRuntimeError::InvalidCodeId,
                    )));
                }
            };
            u64::from_be_bytes(code)
        };

        if native_package_code_id < 1024u64 {
            let instance = self.vm.create_instance(package_address, code)?;
            Ok(TestNativeVmInstance::Normal(instance))
        } else {
            Ok(TestNativeVmInstance::Other(self.custom_invoke.clone()))
        }
    }
}

pub enum TestNativeVmInstance<C: VmInvoke> {
    Normal(NativeVmV1Instance),
    Other(C),
}

impl<C: VmInvoke + Clone> VmInvoke for TestNativeVmInstance<C> {
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match self {
            TestNativeVmInstance::Normal(instance) => instance.invoke(export_name, input, api),
            TestNativeVmInstance::Other(custom_invoke) => {
                custom_invoke.invoke(export_name, input, api)
            }
        }
    }
}

#[test]
fn global_address_access_from_frame_owned_object_should_not_succeed() {
    // Arrange
    #[derive(Clone)]
    struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y>(
            &mut self,
            export_name: &str,
            _input: &IndexedScryptoValue,
            api: &mut Y,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        {
            match export_name {
                "test" => {
                    let node_id = api.new_simple_object("my_blueprint", vec![])?;
                    let _ = api.call_method(&node_id, "get_global_address", scrypto_args!())?;
                    let _ = api.drop_object(&node_id)?;
                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                "get_global_address" => {
                    api.actor_get_global_address()
                        .expect_err("Should not have global address");
                    Ok(IndexedScryptoValue::from_typed(&()))
                }
                _ => Ok(IndexedScryptoValue::from_typed(&())),
            }
        }
    }
    let mut test_runner =
        TestRunnerBuilder::new().build_with_native_vm(TestNativeVm::new(TestInvoke));
    let package_address = PackageAddress::new_or_panic([
        13, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
    ]);
    let mut id_allocator = ManifestIdAllocator::new();
    let receipt = test_runner.execute_system_transaction_with_preallocated_addresses(
        vec![InstructionV1::CallFunction {
            package_address: DynamicPackageAddress::Static(PACKAGE_PACKAGE),
            blueprint_name: "Package".to_string(),
            function_name: "publish_native".to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                definition: PackageDefinition::new_test_definition(
                    "my_blueprint",
                    vec![("test", false), ("get_global_address", true)]
                ),
                native_package_code_id: 1024u64,
                metadata: MetadataInit::default(),
                package_address: Some(id_allocator.new_address_reservation_id()),
            }),
        }],
        vec![(
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(package_address),
        )
            .into()],
        btreeset!(AuthAddresses::system_role()),
    );
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .call_function(package_address, "my_blueprint", "test", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn global_address_access_from_direct_access_methods_should_fail_even_with_borrowed_reference() {
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
