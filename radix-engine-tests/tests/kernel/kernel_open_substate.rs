use radix_common::prelude::*;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::OpenSubstateError;
use radix_engine::kernel::id_allocator::IdAllocator;
use radix_engine::kernel::kernel::BootLoader;
use radix_engine::kernel::kernel_api::KernelSubstateApi;
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::system::system_callback::{System, SystemLockData};
use radix_engine::system::system_modules::costing::{CostingModule, FeeTable, SystemLoanFeeReserve};
use radix_engine::system::system_modules::execution_trace::ExecutionTraceModule;
use radix_engine::system::system_modules::kernel_trace::KernelTraceModule;
use radix_engine::system::system_modules::{EnabledModules, SystemModuleMixer};
use radix_engine::system::system_modules::auth::AuthModule;
use radix_engine::system::system_modules::limits::LimitsModule;
use radix_engine::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use radix_engine::track::Track;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::{DefaultNativeVm, NoExtension, ScryptoVm, Vm, VmInit, VmVersion};
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_substate_store_queries::typed_substate_layout::{
    BlueprintVersionKey, PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET,
};
use radix_transactions::prelude::*;

#[test]
pub fn test_open_substate_of_invisible_package_address() {
    // Create dummy transaction
    let transaction =
        TestTransaction::new_from_nonce(ManifestBuilder::new().lock_fee_from_faucet().build(), 1)
            .prepare()
            .unwrap();
    let executable = transaction.get_executable(btreeset![]);

    // Create database and bootstrap
    let mut database = InMemorySubstateDatabase::standard();
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm_init = VmInit {
        scrypto_vm: &scrypto_vm,
        native_vm_extension: NoExtension,
    };
    Bootstrapper::new(NetworkDefinition::simulator(), &mut database, vm_init, false);

    // Create kernel
    let mut id_allocator = IdAllocator::new(executable.intent_hash().to_hash());
    let mut system = System {
        blueprint_cache: NonIterMap::new(),
        auth_cache: NonIterMap::new(),
        schema_cache: NonIterMap::new(),
        callback: Vm {
            scrypto_vm: &scrypto_vm,
            native_vm,
            vm_version: VmVersion::latest(),
        },
        modules: SystemModuleMixer::new(
            EnabledModules::for_test_transaction(),
            KernelTraceModule,
            TransactionRuntimeModule::new(
                NetworkDefinition::simulator(),
                executable.intent_hash().to_hash(),
            ),
            AuthModule::new(executable.auth_zone_params().clone()),
            LimitsModule::default(),
            CostingModule {
                fee_reserve: SystemLoanFeeReserve::default(),
                fee_table: FeeTable::new(),
                tx_payload_len: executable.payload_size(),
                tx_num_of_signature_validations: executable.auth_zone_params().initial_proofs.len(),
                max_per_function_royalty_in_xrd: Decimal::try_from(MAX_PER_FUNCTION_ROYALTY_IN_XRD).unwrap(),
                cost_breakdown: None,
                on_apply_cost: Default::default(),
            },
            ExecutionTraceModule::new(MAX_EXECUTION_TRACE_DEPTH),
        ),
    };
    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&database);
    let mut boot_loader = BootLoader {
        id_allocator: &mut id_allocator,
        callback: &mut system,
        store: &mut track,
    };
    let mut kernel = boot_loader.boot().unwrap();

    // Lock package substate
    let result = kernel.kernel_open_substate(
        PACKAGE_PACKAGE.as_node_id(),
        MAIN_BASE_PARTITION
            .at_offset(PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET)
            .unwrap(),
        &SubstateKey::Map(scrypto_encode(&BlueprintVersionKey::new_default("Test")).unwrap()),
        LockFlags::read_only(),
        SystemLockData::default(),
    );

    // Verify lock substate
    assert!(matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::OpenSubstateError(OpenSubstateError::SubstateFault)
        )))
    ));
}
