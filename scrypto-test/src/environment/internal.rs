//! This module contains the internal representation of the testing environment which is a self
//! contained Radix Engine implemented as a self-referencing struct.

use super::*;
use crate::prelude::*;

/// The implementation of a self-contained Radix Engine.
///
/// This is a self-contained Radix Engine that uses the [`ouroboros`] crate for self-referencing to
/// allow the entire Radix Engine stack to be stored in a single struct where some members reference
/// one another. As an example: the [`Track`] references the substate database stored in the same
/// object as it.
#[ouroboros::self_referencing(no_doc)]
pub(super) struct SelfContainedRadixEngine {
    pub(super) substate_db: InMemorySubstateDatabase,
    pub(super) scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    pub(super) native_vm: NativeVm<NoExtension>,
    pub(super) id_allocator: IdAllocator,

    #[borrows(substate_db)]
    #[covariant]
    pub(super) track: TestTrack<'this>,

    #[borrows(scrypto_vm)]
    #[covariant]
    pub(super) system_config: TestSystemConfig<'this>,

    #[borrows(mut system_config, mut track, mut id_allocator)]
    #[not_covariant]
    pub(super) kernel: TestKernel<'this>,
}

impl SelfContainedRadixEngine {
    const DEFAULT_INTENT_HASH: Hash = Hash([0; 32]);

    pub(super) fn standard() -> Self {
        let mut substate_db = InMemorySubstateDatabase::standard();

        // Create the various VMs we will use
        let native_vm = NativeVm::new();
        let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
        let vm = Vm::new(&scrypto_vm, native_vm.clone());

        // Run genesis against the substate store.
        let mut bootstrapper = Bootstrapper::new(&mut substate_db, vm, false);
        bootstrapper.bootstrap_test_default().unwrap();

        // Create the Id allocator we will be using throughout this test
        let id_allocator = IdAllocator::new(Self::DEFAULT_INTENT_HASH);

        // Create a self-contained engine from everything else created above.
        SelfContainedRadixEngineBuilder {
            substate_db,
            scrypto_vm,
            native_vm: native_vm.clone(),
            id_allocator,
            track_builder: Self::track_builder,
            system_config_builder: |scrypto_vm| Self::system_config_builder(scrypto_vm, native_vm),
            kernel_builder: Self::kernel_builder,
        }
        .build()
    }

    fn track_builder(substate_store: &InMemorySubstateDatabase) -> TestTrack<'_> {
        Track::new(substate_store)
    }

    fn system_config_builder(
        scrypto_vm: &ScryptoVm<DefaultWasmEngine>,
        native_vm: NativeVm<NoExtension>,
    ) -> TestSystemConfig<'_> {
        SystemConfig {
            blueprint_cache: NonIterMap::new(),
            auth_cache: NonIterMap::new(),
            schema_cache: NonIterMap::new(),
            callback_obj: Vm::new(scrypto_vm, native_vm),
            modules: SystemModuleMixer::new(
                EnabledModules::LIMITS | EnabledModules::AUTH | EnabledModules::TRANSACTION_RUNTIME,
                NetworkDefinition::simulator(),
                Self::DEFAULT_INTENT_HASH,
                AuthZoneParams {
                    initial_proofs: Default::default(),
                    virtual_resources: Default::default(),
                },
                SystemLoanFeeReserve::default(),
                FeeTable::new(),
                0,
                0,
                &ExecutionConfig::for_test_transaction().with_kernel_trace(false),
            ),
        }
    }

    fn kernel_builder<'g>(
        system_config: &'g mut TestSystemConfig<'g>,
        track: &'g mut TestTrack<'g>,
        id_allocator: &'g mut IdAllocator,
    ) -> TestKernel<'g> {
        Kernel::kernel_create_kernel_for_testing(
            SubstateIO {
                heap: Heap::new(),
                store: track,
                non_global_node_refs: NonGlobalNodeRefs::new(),
                substate_locks: SubstateLocks::new(),
                heap_transient_substates: TransientSubstates {
                    transient_substates: Default::default(),
                },
                pinned_nodes: Default::default(),
            },
            id_allocator,
            CallFrame::new_root(Actor::Root),
            vec![],
            system_config,
        )
    }
}
