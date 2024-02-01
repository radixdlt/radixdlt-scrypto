#![cfg(feature = "std")]

use radix_engine::errors::*;
use radix_engine::kernel::id_allocator::*;
use radix_engine::kernel::kernel::*;
use radix_engine::kernel::kernel_api::*;
use radix_engine::system::bootstrap::*;
use radix_engine::system::system::*;
use radix_engine::system::system_callback::*;
use radix_engine::system::system_modules::costing::*;
use radix_engine::system::system_modules::*;
use radix_engine::track::*;
use radix_engine::transaction::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::test_utils::invocations::*;
use radix_engine_interface::prelude::*;
use radix_engine_store_interface::db_key_mapper::*;
use radix_engine_stores::memory_db::*;
use scrypto_test::prelude::TestRunnerBuilder;
use transaction::prelude::*;

#[test]
fn panics_in_native_blueprints_can_be_caught_by_the_native_vm() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let manifest = ManifestBuilder::new()
        .call_function(
            TEST_UTILS_PACKAGE,
            TEST_UTILS_BLUEPRINT,
            TEST_UTILS_PANIC_IDENT,
            TestUtilsPanicInput("I'm panicking!".to_owned()),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::VmError(VmError::Native(NativeRuntimeError::Trap { .. }))
        )
    })
}

#[test]
fn panics_can_be_caught_in_the_native_vm_and_converted_into_results() {
    // Arrange
    let mut substate_db = InMemorySubstateDatabase::standard();

    let _ = Bootstrapper::new(
        NetworkDefinition::simulator(),
        &mut substate_db,
        Vm::new(&ScryptoVm::<DefaultWasmEngine>::default(), NativeVm::new()),
        false,
    )
    .bootstrap_test_default()
    .unwrap();

    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&substate_db);
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = NativeVm::new_with_extension(Extension);
    let vm = Vm::new(&scrypto_vm, native_vm);

    let intent_hash = Hash([0; 32]);
    let mut id_allocator = IdAllocator::new(intent_hash);
    let mut system = SystemConfig {
        blueprint_cache: NonIterMap::new(),
        auth_cache: NonIterMap::new(),
        schema_cache: NonIterMap::new(),
        callback_obj: vm.clone(),
        modules: SystemModuleMixer::new(
            EnabledModules::for_notarized_transaction(),
            NetworkDefinition::simulator(),
            intent_hash,
            AuthZoneParams {
                initial_proofs: Default::default(),
                virtual_resources: Default::default(),
            },
            SystemLoanFeeReserve::default(),
            FeeTable::new(),
            0,
            1,
            &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
        ),
    };

    let mut boot_loader = BootLoader {
        id_allocator: &mut id_allocator,
        callback: &mut system,
        store: &mut track,
    };
    let mut kernel = boot_loader.boot().unwrap();
    let mut api = SystemService {
        api: &mut kernel,
        phantom: Default::default(),
    };

    // Act
    let rtn = api.call_function(
        ACCOUNT_PACKAGE,
        ACCOUNT_BLUEPRINT,
        ACCOUNT_CREATE_ADVANCED_IDENT,
        scrypto_encode(&AccountCreateAdvancedInput {
            address_reservation: None,
            owner_role: OwnerRole::None,
        })
        .unwrap(),
    );

    // Assert
    assert!(matches!(
        rtn,
        Err(RuntimeError::VmError(VmError::Native(
            NativeRuntimeError::Trap { .. }
        )))
    ))
}

#[test]
fn any_panics_can_be_caught_in_the_native_vm_and_converted_into_results() {
    // Arrange
    let mut substate_db = InMemorySubstateDatabase::standard();

    let _ = Bootstrapper::new(
        NetworkDefinition::simulator(),
        &mut substate_db,
        Vm::new(&ScryptoVm::<DefaultWasmEngine>::default(), NativeVm::new()),
        false,
    )
    .bootstrap_test_default()
    .unwrap();

    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&substate_db);
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = NativeVm::new_with_extension(NonStringPanicExtension);
    let vm = Vm::new(&scrypto_vm, native_vm);

    let intent_hash = Hash([0; 32]);
    let mut id_allocator = IdAllocator::new(intent_hash);
    let mut system = SystemConfig {
        blueprint_cache: NonIterMap::new(),
        auth_cache: NonIterMap::new(),
        schema_cache: NonIterMap::new(),
        callback_obj: vm.clone(),
        modules: SystemModuleMixer::new(
            EnabledModules::for_notarized_transaction(),
            NetworkDefinition::simulator(),
            intent_hash,
            AuthZoneParams {
                initial_proofs: Default::default(),
                virtual_resources: Default::default(),
            },
            SystemLoanFeeReserve::default(),
            FeeTable::new(),
            0,
            1,
            &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
        ),
    };

    let mut boot_loader = BootLoader {
        id_allocator: &mut id_allocator,
        callback: &mut system,
        store: &mut track,
    };
    let mut kernel = boot_loader.boot().unwrap();
    let mut api = SystemService {
        api: &mut kernel,
        phantom: Default::default(),
    };

    // Act
    let rtn = api.call_function(
        ACCOUNT_PACKAGE,
        ACCOUNT_BLUEPRINT,
        ACCOUNT_CREATE_ADVANCED_IDENT,
        scrypto_encode(&AccountCreateAdvancedInput {
            address_reservation: None,
            owner_role: OwnerRole::None,
        })
        .unwrap(),
    );

    // Assert
    assert!(matches!(
        rtn,
        Err(RuntimeError::VmError(VmError::Native(
            NativeRuntimeError::Trap { .. }
        )))
    ))
}

#[derive(Clone)]
pub struct Extension;

impl NativeVmExtension for Extension {
    type Instance = NullVmInvoke;

    fn try_create_instance(&self, _: &[u8]) -> Option<Self::Instance> {
        Some(NullVmInvoke)
    }
}

#[derive(Clone)]
pub struct NonStringPanicExtension;

impl NativeVmExtension for NonStringPanicExtension {
    type Instance = NonStringPanicExtensionInstance;

    fn try_create_instance(&self, _: &[u8]) -> Option<Self::Instance> {
        Some(NonStringPanicExtensionInstance)
    }
}

#[derive(Clone)]
pub struct NonStringPanicExtensionInstance;

impl VmInvoke for NonStringPanicExtensionInstance {
    fn invoke<Y, V>(
        &mut self,
        _: &str,
        _: &IndexedScryptoValue,
        _: &mut Y,
        _: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        V: VmApi,
    {
        // A panic with a non-string type. Making sure that our panic infrastructure can catch those
        // panics too even if it can't make any useful messages out of them.
        std::panic::panic_any(1234);
    }
}
