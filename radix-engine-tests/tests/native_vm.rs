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
use radix_engine_interface::prelude::*;
use radix_engine_store_interface::db_key_mapper::*;
use radix_engine_stores::memory_db::*;
use scrypto::prelude::*;
use transaction::prelude::*;

#[cfg(feature = "std")]
#[test]
fn panics_can_be_caught_in_the_native_vm_and_converted_into_results() {
    // Arrange
    let mut substate_db = InMemorySubstateDatabase::standard();

    let _ = Bootstrapper::new(
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
            &ExecutionConfig::for_notarized_transaction(),
        ),
    };

    let mut kernel_boot = KernelBoot {
        id_allocator: &mut id_allocator,
        callback: &mut system,
        store: &mut track,
    };
    let mut kernel = kernel_boot.create_kernel_for_test_only();
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
        Err(RuntimeError::ApplicationError(
            ApplicationError::NativePanic { .. }
        ))
    ))
}

#[derive(Clone)]
pub struct Extension;

#[derive(Clone)]
pub struct ExtensionInstance;

impl NativeVmExtension for Extension {
    type Instance = ExtensionInstance;

    fn try_create_instance(&self, _: &[u8]) -> Option<Self::Instance> {
        Some(ExtensionInstance)
    }
}

impl VmInvoke for ExtensionInstance {
    fn invoke<Y>(
        &mut self,
        _: &str,
        _: &IndexedScryptoValue,
        _: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        panic!("This VM extension does nothing but panic. We're testing to see if the native VM code can recover from panics.")
    }
}
