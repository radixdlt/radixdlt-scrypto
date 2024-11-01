#![cfg(feature = "std")]

use radix_common::prelude::*;
use radix_engine::errors::*;
use radix_engine::kernel::id_allocator::*;
use radix_engine::kernel::kernel::*;
use radix_engine::kernel::kernel_api::*;
use radix_engine::system::system::*;
use radix_engine::system::system_callback::*;
use radix_engine::system::system_modules::auth::AuthModule;
use radix_engine::system::system_modules::costing::*;
use radix_engine::system::system_modules::execution_trace::ExecutionTraceModule;
use radix_engine::system::system_modules::kernel_trace::KernelTraceModule;
use radix_engine::system::system_modules::limits::LimitsModule;
use radix_engine::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use radix_engine::system::system_modules::*;
use radix_engine::track::*;
use radix_engine::updates::ProtocolBuilder;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::test_utils::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::*;
use radix_transactions::prelude::*;
use scrypto_test::prelude::LedgerSimulatorBuilder;

#[test]
fn panics_in_native_blueprints_can_be_caught_by_the_native_vm() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            TEST_UTILS_PACKAGE,
            TEST_UTILS_BLUEPRINT,
            TEST_UTILS_PANIC_IDENT,
            TestUtilsPanicInput("I'm panicking!".to_owned()),
        )
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
    ProtocolBuilder::for_simulator()
        .from_bootstrap_to_latest()
        .commit_each_protocol_update(&mut substate_db);

    let mut track = Track::new(&substate_db);
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = NativeVm::new_with_extension(Extension);

    let intent_hash = Hash([0; 32]);
    let mut system = System::new(
        SystemVersion::latest(),
        Vm {
            scrypto_vm: &scrypto_vm,
            native_vm,
            vm_boot: VmBoot::latest(),
        },
        SystemModuleMixer::new(
            EnabledModules::for_notarized_transaction(),
            KernelTraceModule,
            TransactionRuntimeModule::new(NetworkDefinition::simulator(), intent_hash),
            AuthModule::new(),
            LimitsModule::babylon_genesis(),
            CostingModule {
                current_depth: 0,
                fee_reserve: SystemLoanFeeReserve::default(),
                fee_table: FeeTable::latest(),
                tx_payload_len: 0,
                tx_num_of_signature_validations: 1,
                config: CostingModuleConfig::babylon_genesis(),
                cost_breakdown: None,
                detailed_cost_breakdown: None,
                on_apply_cost: Default::default(),
            },
            ExecutionTraceModule::new(MAX_EXECUTION_TRACE_DEPTH),
        ),
        SystemFinalization::no_nullifications(),
    );

    let mut id_allocator = IdAllocator::new(intent_hash);
    let mut kernel = Kernel::new_no_refs(&mut track, &mut id_allocator, &mut system);

    let mut api = SystemService::new(&mut kernel);

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
    assert_matches!(
        rtn,
        Err(RuntimeError::VmError(VmError::Native(
            NativeRuntimeError::Trap { .. }
        )))
    )
}

#[test]
fn any_panics_can_be_caught_in_the_native_vm_and_converted_into_results() {
    // Arrange
    let mut substate_db = InMemorySubstateDatabase::standard();
    ProtocolBuilder::for_simulator()
        .from_bootstrap_to_latest()
        .commit_each_protocol_update(&mut substate_db);

    let mut track = Track::new(&substate_db);
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = NativeVm::new_with_extension(NonStringPanicExtension);

    let intent_hash = Hash([0; 32]);
    let mut system = System::new(
        SystemVersion::latest(),
        Vm {
            scrypto_vm: &scrypto_vm,
            native_vm,
            vm_boot: VmBoot::latest(),
        },
        SystemModuleMixer::new(
            EnabledModules::for_notarized_transaction(),
            KernelTraceModule,
            TransactionRuntimeModule::new(NetworkDefinition::simulator(), intent_hash),
            AuthModule::new(),
            LimitsModule::babylon_genesis(),
            CostingModule {
                current_depth: 0,
                fee_reserve: SystemLoanFeeReserve::default(),
                fee_table: FeeTable::latest(),
                tx_payload_len: 0,
                tx_num_of_signature_validations: 1,
                config: CostingModuleConfig::babylon_genesis(),
                cost_breakdown: None,
                detailed_cost_breakdown: None,
                on_apply_cost: Default::default(),
            },
            ExecutionTraceModule::new(MAX_EXECUTION_TRACE_DEPTH),
        ),
        SystemFinalization::no_nullifications(),
    );

    let mut id_allocator = IdAllocator::new(intent_hash);
    let mut kernel = Kernel::new_no_refs(&mut track, &mut id_allocator, &mut system);
    let mut api = SystemService::new(&mut kernel);

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
    assert_matches!(
        rtn,
        Err(RuntimeError::VmError(VmError::Native(
            NativeRuntimeError::Trap { .. }
        )))
    )
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
    fn invoke<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        V: VmApi,
    >(
        &mut self,
        _: &str,
        _: &IndexedScryptoValue,
        _: &mut Y,
        _: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // A panic with a non-string type. Making sure that our panic infrastructure can catch those
        // panics too even if it can't make any useful messages out of them.
        std::panic::panic_any(1234);
    }
}
