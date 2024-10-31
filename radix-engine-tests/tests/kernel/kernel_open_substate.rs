use radix_engine::system::system_modules::kernel_trace::KernelTraceModule;
use radix_engine::system::system_modules::limits::LimitsModule;
use radix_engine::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use scrypto_test::prelude::*;

#[test]
pub fn test_open_substate_of_invisible_package_address() {
    // Create dummy transaction
    let executable = TestTransaction::new_v1_from_nonce(
        ManifestBuilder::new().lock_fee_from_faucet().build(),
        1,
        btreeset![],
    )
    .into_executable_unwrap();

    // Create database and bootstrap
    let mut database = InMemorySubstateDatabase::standard();
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    ProtocolBuilder::for_simulator()
        .from_bootstrap_to_latest()
        .commit_each_protocol_update(&mut database);

    // Create kernel
    let mut system = System::new(
        SystemVersion::latest(),
        Vm {
            scrypto_vm: &scrypto_vm,
            native_vm,
            vm_boot: VmBoot::latest(),
        },
        SystemModuleMixer::new(
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
                fee_table: FeeTable::latest(),
                tx_payload_len: executable.payload_size(),
                tx_num_of_signature_validations: executable.num_of_signature_validations(),
                config: CostingModuleConfig::babylon_genesis(),
                cost_breakdown: None,
                detailed_cost_breakdown: None,
                on_apply_cost: Default::default(),
            },
            ExecutionTraceModule::new(MAX_EXECUTION_TRACE_DEPTH),
        ),
        SystemFinalization::no_nullifications(),
    );
    let mut track = Track::new(&database);
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
    assert_matches!(
        result,
        Err(RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::OpenSubstateError(OpenSubstateError::SubstateFault)
        )))
    );
}
