//! This module defines the test-runtime and its implementations, methods, and functions which is
//! the foundation of the invocation-based model of testing.

use crate::prelude::*;

/// A self-contained instance of the Radix Engine exposed through the [`ClientApi`] and
/// [`KernelApi`].
///
/// Each instance of [`TestRuntime`] has an [`InMemorySubstateDatabase`], [`Track`], and [`Kernel`]
/// which makes it a self-contained instance of the Radix Engine. It implements the [`ClientApi`]
/// and [`KernelApi`] making it a drop-in replacement for `ScryptoEnv` from Scrypto and the
/// [`SystemService`] from native.
pub struct TestRuntime(TestRuntimeInternal);

impl TestRuntime {
    /// Creates a new [`TestRuntime`] with the default configuration.
    ///
    /// By default, the [`TestRuntime`] has the auth and limits kernel modules enabled while all
    /// other kernel modules are disabled, including the costing module. The kernel has is created
    /// with two call-frames: a root call-frame and a function-actor call-frame.
    pub fn new() -> Self {
        Self::default()
    }

    /* Manipulation of Kernel Modules */

    /// Enables the kernel trace kernel module of the Radix Engine.
    pub fn enable_kernel_trace_module(&mut self) {
        self.enable_module(EnabledModules::KERNEL_TRACE)
    }

    /// Enables the limits kernel module of the Radix Engine.
    pub fn enable_limits_module(&mut self) {
        self.enable_module(EnabledModules::LIMITS)
    }

    /// Enables the costing kernel module of the Radix Engine.
    pub fn enable_costing_module(&mut self) {
        self.enable_module(EnabledModules::COSTING)
    }

    /// Enables the auth kernel module of the Radix Engine.
    pub fn enable_auth_module(&mut self) {
        self.enable_module(EnabledModules::AUTH)
    }

    /// Enables the transaction runtime kernel module of the Radix Engine.
    pub fn enable_transaction_runtime_module(&mut self) {
        self.enable_module(EnabledModules::TRANSACTION_RUNTIME)
    }

    /// Enables the execution trace kernel module of the Radix Engine.
    pub fn enable_execution_trace_module(&mut self) {
        self.enable_module(EnabledModules::EXECUTION_TRACE)
    }

    /// Disables the kernel trace kernel module of the Radix Engine.
    pub fn disable_kernel_trace_module(&mut self) {
        self.disable_module(EnabledModules::KERNEL_TRACE)
    }

    /// Disables the limits kernel module of the Radix Engine.
    pub fn disable_limits_module(&mut self) {
        self.disable_module(EnabledModules::LIMITS)
    }

    /// Disables the costing kernel module of the Radix Engine.
    pub fn disable_costing_module(&mut self) {
        self.disable_module(EnabledModules::COSTING)
    }

    /// Disables the auth kernel module of the Radix Engine.
    pub fn disable_auth_module(&mut self) {
        self.disable_module(EnabledModules::AUTH)
    }

    /// Disables the transaction runtime kernel module of the Radix Engine.
    pub fn disable_transaction_runtime_module(&mut self) {
        self.disable_module(EnabledModules::TRANSACTION_RUNTIME)
    }

    /// Disables the execution trace kernel module of the Radix Engine.
    pub fn disable_execution_trace_module(&mut self) {
        self.disable_module(EnabledModules::EXECUTION_TRACE)
    }

    /// Calls the passed `callback` with the kernel trace kernel module enabled.
    pub fn with_kernel_trace_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_kernel_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the limits kernel module enabled.
    pub fn with_limits_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_limits_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the costing kernel module enabled.
    pub fn with_costing_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_costing_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the auth kernel module enabled.
    pub fn with_auth_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_auth_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the transaction runtime kernel module enabled.
    pub fn with_transaction_runtime_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_transaction_runtime_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the execution trace kernel module enabled.
    pub fn with_execution_trace_module_enabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.enable_execution_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the kernel trace kernel module disabled.
    pub fn with_kernel_trace_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_kernel_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the limits kernel module disabled.
    pub fn with_limits_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_limits_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the costing kernel module disabled.
    pub fn with_costing_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_costing_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the auth kernel module disabled.
    pub fn with_auth_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_auth_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the transaction runtime kernel module disabled.
    pub fn with_transaction_runtime_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_transaction_runtime_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Calls the passed `callback` with the execution trace kernel module disabled.
    pub fn with_execution_trace_module_disabled<F, O>(&mut self, callback: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let enabled_modules = self.enabled_modules();
        self.disable_execution_trace_module();
        let rtn = callback(self);
        self.set_enabled_modules(enabled_modules);
        rtn
    }

    /// Returns the bit flags representing the currently enabled kernel modules.
    pub fn enabled_modules(&self) -> EnabledModules {
        self.0
            .with_kernel(|kernel| kernel.kernel_callback().modules.enabled_modules)
    }

    /// Sets the bit flags representing the enabled kernel modules.
    pub fn set_enabled_modules(&mut self, enabled_modules: EnabledModules) {
        self.0.with_kernel_mut(|kernel| {
            kernel.kernel_callback_mut().modules.enabled_modules = enabled_modules
        })
    }

    /// Enables specific kernel module(s).
    pub fn enable_module(&mut self, module: EnabledModules) {
        self.0.with_kernel_mut(|kernel| {
            kernel.kernel_callback_mut().modules.enabled_modules |= module
        })
    }

    /// Disables specific kernel module(s).
    pub fn disable_module(&mut self, module: EnabledModules) {
        self.0.with_kernel_mut(|kernel| {
            kernel.kernel_callback_mut().modules.enabled_modules &= !module
        })
    }
}

impl Default for TestRuntime {
    fn default() -> Self {
        Self(TestRuntimeInternal::new_internal())
    }
}

/// The internal implementation of the [`TestRuntime`].
///
/// This struct defines a self-contained instance of the Radix Engine that has all parts of the
/// engine stack from a substate store, all the way up to a kernel and VMs. The [`ouroboros`] crate
/// is used here to allow for the creation of a self-referencing struct. More specifically, this
/// crate allows for the use of the `'this` lifetime and allows members of the struct to hold
/// references to other members of the struct.
#[ouroboros::self_referencing(no_doc)]
struct TestRuntimeInternal {
    substate_db: InMemorySubstateDatabase,
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    native_vm: NativeVm<NoExtension>,
    id_allocator: IdAllocator,

    #[borrows(substate_db)]
    #[covariant]
    track: TestRuntimeTrack<'this>,

    #[borrows(scrypto_vm)]
    #[covariant]
    system_config: TestRuntimeSystemConfig<'this>,

    #[borrows(mut system_config, mut track, mut id_allocator)]
    #[not_covariant]
    kernel: TestRuntimeKernel<'this>,
}

impl TestRuntimeInternal {
    const DEFAULT_INTENT_HASH: Hash = Hash([0; 32]);

    fn new_internal() -> Self {
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

        TestRuntimeInternalBuilder {
            substate_db,
            scrypto_vm,
            native_vm: native_vm.clone(),
            id_allocator,
            track_builder: |substate_store| Track::new(substate_store),
            system_config_builder: |scrypto_vm| SystemConfig {
                blueprint_cache: NonIterMap::new(),
                auth_cache: NonIterMap::new(),
                schema_cache: NonIterMap::new(),
                callback_obj: Vm::new(scrypto_vm, native_vm),
                modules: SystemModuleMixer::new(
                    EnabledModules::LIMITS | EnabledModules::AUTH,
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
            },
            kernel_builder: |system, track, id_allocator| {
                // Create the kernel
                let mut kernel = Kernel::kernel_create_kernel_for_testing(
                    SubstateIO {
                        heap: Heap::new(),
                        store: track,
                        non_global_node_refs: NonGlobalNodeRefs::new(),
                        substate_locks: SubstateLocks::new(),
                    },
                    id_allocator,
                    CallFrame::new_root(Actor::Root),
                    vec![],
                    system,
                );

                // Add references to all of the well-known node ids to the root call frame.
                let current_frame = kernel.kernel_current_frame_mut();
                for node_id in GLOBAL_VISIBLE_NODES {
                    let Ok(global_address) = GlobalAddress::try_from(node_id.0) else {
                        continue;
                    };
                    current_frame.add_global_reference(global_address)
                }

                // Create a new call-frame along with an auth-zone for this call-frame
                // TODO: For the time being, the call-frame that we create has a function-actor of
                // the transaction processor. Maybe we should not use this and use another native
                // or non-native blueprints
                let auth_zone = {
                    let mut system_service = SystemService {
                        api: &mut kernel,
                        phantom: PhantomData,
                    };
                    AuthModule::create_mock(
                        &mut system_service,
                        Some((TRANSACTION_PROCESSOR_PACKAGE.as_node_id(), false)),
                        Default::default(),
                        Default::default(),
                    )
                }
                .unwrap();
                let actor = Actor::Function(FunctionActor {
                    blueprint_id: BlueprintId {
                        package_address: TRANSACTION_PROCESSOR_PACKAGE,
                        blueprint_name: TRANSACTION_PROCESSOR_BLUEPRINT.to_owned(),
                    },
                    ident: TRANSACTION_PROCESSOR_RUN_IDENT.to_owned(),
                    auth_zone: auth_zone,
                });

                let message =
                    CallFrameMessage::from_input(&IndexedScryptoValue::from_typed(&()), &actor);
                let current_frame = kernel.kernel_current_frame_mut();
                let new_frame =
                    CallFrame::new_child_from_parent(current_frame, actor, message).unwrap();
                let old = core::mem::replace(current_frame, new_frame);
                kernel.kernel_prev_frame_stack_mut().push(old);

                kernel
            },
        }
        .build()
    }
}

/// Implements the [`ClientApi`] for the [`TestRuntime`] struct.
///
/// This macro exposes a high-level API for specifying the [`ClientApi`] traits to implement for the
/// [`TestRuntime`]. The trait methods are implements through a simple mechanism which creates a
/// [`SystemService`] object from the kernel and calls the trait method on the [`SystemService`]
/// object.
///
/// The syntax supported by this macro is as follows:
///
/// ```no_run
/// implement_client_api! {
///     trait_name: {
///         trait_method1: (args: ArgTypes) -> ReturnTypes,
///         trait_method2: (args: ArgTypes) -> ReturnTypes,
///     }
/// }
/// ```
///
/// This macro is only used internally in this crate for easy implementation of the [`ClientApi`]
/// and is not meant to be used outside or exported.
macro_rules! implement_client_api {
    (
        $(
            $trait: ident: {
                $(
                    $func_ident: ident: (
                        &mut self
                        $(, $input_ident: ident: $input_type: ty)* $(,)?
                    ) -> $outputs: ty
                ),* $(,)?
            }
        ),* $(,)*
    ) => {
        $(
            impl $trait<RuntimeError> for TestRuntime {
                $(
                    #[inline]
                    fn $func_ident(&mut self, $($input_ident: $input_type),*) -> $outputs {
                        self.0.with_kernel_mut(|kernel| {
                            SystemService {
                                api: kernel,
                                phantom: PhantomData,
                            }.$func_ident( $($input_ident),* )
                        })
                    }
                )*
            }
        )*
    };
}
implement_client_api! {
    ClientApi: {},
    ClientActorApi: {
        actor_get_blueprint_id: (&mut self) -> Result<BlueprintId, RuntimeError>,
        actor_open_field: (
            &mut self,
            object_handle: ObjectHandle,
            field: FieldIndex,
            flags: LockFlags,
        ) -> Result<FieldHandle, RuntimeError>,
        actor_is_feature_enabled: (
            &mut self,
            object_handle: ObjectHandle,
            feature: &str,
        ) -> Result<bool, RuntimeError>,
        actor_get_node_id: (&mut self) -> Result<NodeId, RuntimeError>,
        actor_get_outer_object: (&mut self) -> Result<GlobalAddress, RuntimeError>,
        actor_get_global_address: (&mut self) -> Result<GlobalAddress, RuntimeError>,
        actor_call_module: (
            &mut self,
            module_id: ObjectModuleId,
            method_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientActorIndexApi: {
        actor_index_insert: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: Vec<u8>,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        actor_index_remove: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: Vec<u8>,
        ) -> Result<Option<Vec<u8>>, RuntimeError>,
        actor_index_scan_keys: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            limit: u32,
        ) -> Result<Vec<Vec<u8>>, RuntimeError>,
        actor_index_drain: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            limit: u32,
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, RuntimeError>,
    },
    ClientActorKeyValueEntryApi: {
        actor_open_key_value_entry: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: &Vec<u8>,
            flags: LockFlags,
        ) -> Result<KeyValueEntryHandle, RuntimeError>,
        actor_remove_key_value_entry: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: &Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientActorSortedIndexApi: {
        actor_sorted_index_insert: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            sorted_key: SortedKey,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        actor_sorted_index_remove: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            sorted_key: &SortedKey,
        ) -> Result<Option<Vec<u8>>, RuntimeError>,
        actor_sorted_index_scan: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            count: u32,
        ) -> Result<Vec<(SortedKey, Vec<u8>)>, RuntimeError>,
    },
    ClientBlueprintApi: {
        call_function: (
            &mut self,
            package_address: PackageAddress,
            blueprint_name: &str,
            function_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>
    },
    ClientFieldApi: {
        field_read: (&mut self, handle: field_api::FieldHandle) -> Result<Vec<u8>, RuntimeError>,
        field_write: (&mut self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), RuntimeError>,
        field_lock: (&mut self, handle: FieldHandle) -> Result<(), RuntimeError>,
        field_close: (&mut self, handle: FieldHandle) -> Result<(), RuntimeError>
    },
    ClientKeyValueEntryApi: {
        key_value_entry_get: (&mut self, handle: KeyValueEntryHandle) -> Result<Vec<u8>, RuntimeError>,
        key_value_entry_set: (
            &mut self,
            handle: KeyValueEntryHandle,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        key_value_entry_remove: (&mut self, handle: KeyValueEntryHandle) -> Result<Vec<u8>, RuntimeError>,
        key_value_entry_lock: (&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError>,
        key_value_entry_close: (&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError>,
    },
    ClientKeyValueStoreApi: {
        key_value_store_new: (&mut self, generic_args: KeyValueStoreGenericArgs) -> Result<NodeId, RuntimeError>,
        key_value_store_open_entry: (
            &mut self,
            node_id: &NodeId,
            key: &Vec<u8>,
            flags: LockFlags,
        ) -> Result<KeyValueEntryHandle, RuntimeError>,
        key_value_store_remove_entry: (
            &mut self,
            node_id: &NodeId,
            key: &Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientObjectApi: {
        new_object: (
            &mut self,
            blueprint_ident: &str,
            features: Vec<&str>,
            generic_args: GenericArgs,
            fields: Vec<FieldValue>,
            kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
        ) -> Result<NodeId, RuntimeError>,
        drop_object: (&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, RuntimeError>,
        get_blueprint_id: (&mut self, node_id: &NodeId) -> Result<BlueprintId, RuntimeError>,
        get_outer_object: (&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError>,
        allocate_global_address: (
            &mut self,
            blueprint_id: BlueprintId,
        ) -> Result<(GlobalAddressReservation, GlobalAddress), RuntimeError>,
        allocate_virtual_global_address: (
            &mut self,
            blueprint_id: BlueprintId,
            global_address: GlobalAddress,
        ) -> Result<GlobalAddressReservation, RuntimeError>,
        get_reservation_address: (&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError>,
        globalize: (
            &mut self,
            modules: BTreeMap<ObjectModuleId, NodeId>,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Result<GlobalAddress, RuntimeError>,
        globalize_with_address_and_create_inner_object_and_emit_event: (
            &mut self,
            modules: BTreeMap<ObjectModuleId, NodeId>,
            address_reservation: GlobalAddressReservation,
            inner_object_blueprint: &str,
            inner_object_fields: Vec<FieldValue>,
            event_name: String,
            event_data: Vec<u8>,
        ) -> Result<(GlobalAddress, NodeId), RuntimeError>,
        call_method_advanced: (
            &mut self,
            receiver: &NodeId,
            module_id: ObjectModuleId,
            direct_access: bool,
            method_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientAuthApi: {
        get_auth_zone: (&mut self) -> Result<NodeId, RuntimeError>,
    },
    ClientExecutionTraceApi: {
        update_instruction_index: (&mut self, new_index: usize) -> Result<(), RuntimeError>,
    },
    ClientTransactionRuntimeApi: {
        get_transaction_hash: (&mut self) -> Result<Hash, RuntimeError>,
        generate_ruid: (&mut self) -> Result<[u8; 32], RuntimeError>,
        emit_log: (&mut self, level: Level, message: String) -> Result<(), RuntimeError>,
        emit_event: (&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError>,
        panic: (&mut self, message: String) -> Result<(), RuntimeError>,
    },
    ClientCostingApi: {
        consume_cost_units: (&mut self, costing_entry: ClientCostingEntry) -> Result<(), RuntimeError>,
        credit_cost_units: (
            &mut self,
            vault_id: NodeId,
            locked_fee: LiquidFungibleResource,
            contingent: bool,
        ) -> Result<LiquidFungibleResource, RuntimeError>,
        execution_cost_unit_limit: (&mut self) -> Result<u32, RuntimeError>,
        execution_cost_unit_price: (&mut self) -> Result<Decimal, RuntimeError>,
        finalization_cost_unit_limit: (&mut self) -> Result<u32, RuntimeError>,
        finalization_cost_unit_price: (&mut self) -> Result<Decimal, RuntimeError>,
        usd_price: (&mut self) -> Result<Decimal, RuntimeError>,
        max_per_function_royalty_in_xrd: (&mut self) -> Result<Decimal, RuntimeError>,
        tip_percentage: (&mut self) -> Result<u32, RuntimeError>,
        fee_balance: (&mut self) -> Result<Decimal, RuntimeError>,
    }
}
