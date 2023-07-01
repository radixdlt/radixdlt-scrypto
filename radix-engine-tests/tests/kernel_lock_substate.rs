use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::OpenSubstateError;
use radix_engine::kernel::id_allocator::IdAllocator;
use radix_engine::kernel::kernel::KernelBoot;
use radix_engine::kernel::kernel_api::KernelSubstateApi;
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::system::system_callback::{SystemConfig, SystemLockData};
use radix_engine::system::system_modules::costing::{FeeTable, SystemLoanFeeReserve};
use radix_engine::system::system_modules::SystemModuleMixer;
use radix_engine::track::Track;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::{ScryptoVm, Vm};
use radix_engine_interface::api::LockFlags;
use radix_engine_queries::typed_substate_layout::{
    BlueprintVersionKey, PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET,
};
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::builder::ManifestBuilder;
use transaction::prelude::TestTransaction;

#[test]
pub fn test_open_substate_of_invisible_package_address() {
    // Create dummy transaction
    let transaction = TestTransaction::new_from_nonce(
        ManifestBuilder::new()
            .lock_fee(FAUCET, 500u32.into())
            .build(),
        1,
    )
    .prepare()
    .unwrap();
    let executable = transaction.get_executable(btreeset![]);
    let execution_config = ExecutionConfig::for_test_transaction();

    // Create database and bootstrap
    let mut database = InMemorySubstateDatabase::standard();
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    Bootstrapper::new(&mut database, &scrypto_vm, false);

    // Create kernel
    let mut id_allocator = IdAllocator::new(executable.intent_hash().to_hash());
    let mut system = SystemConfig {
        blueprint_cache: NonIterMap::new(),
        auth_cache: NonIterMap::new(),
        schema_cache: NonIterMap::new(),
        callback_obj: Vm {
            scrypto_vm: &scrypto_vm,
        },
        modules: SystemModuleMixer::new(
            execution_config.enabled_modules,
            executable.intent_hash().to_hash(),
            executable.auth_zone_params().clone(),
            SystemLoanFeeReserve::default(),
            FeeTable::new(),
            executable.payload_size(),
            executable.auth_zone_params().initial_proofs.len(),
            &execution_config,
        ),
    };
    let mut track = Track::<InMemorySubstateDatabase, SpreadPrefixKeyMapper>::new(&database);
    let mut kernel_boot = KernelBoot {
        id_allocator: &mut id_allocator,
        callback: &mut system,
        store: &mut track,
    };
    let mut kernel = kernel_boot.create_kernel_for_test_only();

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
            CallFrameError::OpenSubstateError(OpenSubstateError::TrackError(_))
        )))
    ));
}
