use radix_common::prelude::*;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::OpenSubstateError;
use radix_engine::kernel::id_allocator::IdAllocator;
use radix_engine::kernel::kernel::Kernel;
use radix_engine::kernel::kernel_api::KernelSubstateApi;
use radix_engine::system::system_callback::{System, SystemLockData, VersionedSystemLogic};
use radix_engine::system::system_modules::auth::AuthModule;
use radix_engine::system::system_modules::costing::{
    CostingModule, CostingModuleConfig, FeeTable, SystemLoanFeeReserve,
};
use radix_engine::system::system_modules::execution_trace::ExecutionTraceModule;
use radix_engine::system::system_modules::kernel_trace::KernelTraceModule;
use radix_engine::system::system_modules::limits::LimitsModule;
use radix_engine::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use radix_engine::system::system_modules::{EnabledModules, SystemModuleMixer};
use radix_engine::track::Track;
use radix_engine::updates::ProtocolBuilder;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_substate_store_queries::typed_substate_layout::{
    BlueprintVersionKey, PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET,
};
use radix_transactions::prelude::*;
use scrypto_test::prelude::UniqueTransaction;

#[test]
pub fn test_open_substate_of_invisible_package_address() {
    // Create dummy transaction
    let transaction = TestTransaction::new_v1_from_nonce(
        ManifestBuilder::new().lock_fee_from_faucet().build(),
        1,
        btreeset![],
    )
    .prepare_with_latest_settings()
    .unwrap();
    let executable = transaction.get_executable();

    // Create database and bootstrap
    let mut database = InMemorySubstateDatabase::standard();
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    ProtocolBuilder::for_simulator()
        .from_bootstrap_to_latest()
        .commit_each_protocol_update(&mut database);

    // Create kernel
    let mut system = System {
        versioned_system_logic: VersionedSystemLogic::V1,
        blueprint_cache: NonIterMap::new(),
        auth_cache: NonIterMap::new(),
        schema_cache: NonIterMap::new(),
        callback: Vm {
            scrypto_vm: &scrypto_vm,
            native_vm,
            vm_boot: VmBoot::latest(),
        },
        modules: SystemModuleMixer::new(
            EnabledModules::for_test_transaction(),
            KernelTraceModule,
            TransactionRuntimeModule::new(
                NetworkDefinition::simulator(),
                *executable.unique_hash(),
            ),
            AuthModule::new(),
            LimitsModule::babylon_genesis(),
            CostingModule {
                current_depth: 0,
                fee_reserve: SystemLoanFeeReserve::default(),
                fee_table: FeeTable::new(),
                tx_payload_len: executable.payload_size(),
                tx_num_of_signature_validations: executable.num_of_signature_validations(),
                config: CostingModuleConfig::babylon_genesis(),
                cost_breakdown: None,
                detailed_cost_breakdown: None,
                on_apply_cost: Default::default(),
            },
            ExecutionTraceModule::new(MAX_EXECUTION_TRACE_DEPTH),
        ),
        finalization: Default::default(),
    };
    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&database);
    let mut id_allocator = IdAllocator::new(executable.unique_seed_for_id_allocator());
    let mut kernel = Kernel::new_no_refs(&mut track, &mut id_allocator, &mut system);

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
