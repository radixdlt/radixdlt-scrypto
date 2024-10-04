//! This module has the implementation of the [`SystemApi`] for the [`TestEnvironment`] in order not
//! to clutter up the other modules.
//!
//! [`SystemApi`]: crate::prelude::SystemApi
//! [`TestEnvironment`]: crate::prelude::TestEnvironment

use crate::prelude::*;

/// Implements the [`SystemApi`] for the [`TestEnvironment`] struct.
///
/// This macro exposes a high-level API for specifying the [`SystemApi`] traits to implement for the
/// [`TestEnvironment`]. The trait methods are implements through a simple mechanism which creates a
/// [`SystemService`] object from the kernel and calls the trait method on the [`SystemService`]
/// object.
///
/// The syntax supported by this macro is as follows:
///
/// ```rust,ignore
/// # // Ignoring because the macro isn't exported, so this test can't use it
/// implement_system_api! {
///     trait_name: {
///         trait_method1: (args: ArgTypes) -> ReturnTypes,
///         trait_method2: (args: ArgTypes) -> ReturnTypes,
///     }
/// }
/// ```
///
/// This macro is only used internally in this crate for easy implementation of the [`SystemApi`]
/// and is not meant to be used outside or exported.
///
/// [`SystemApi`]: crate::prelude::SystemApi
/// [`TestEnvironment`]: crate::prelude::TestEnvironment
/// [`SystemService`]: crate::prelude::SystemService
macro_rules! implement_system_api {
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
            impl<D> $trait<RuntimeError> for TestEnvironment<D>
            where
                D: SubstateDatabase + CommittableSubstateDatabase + 'static
            {
                $(
                    #[inline]
                    fn $func_ident(&mut self, $($input_ident: $input_type),*) -> $outputs {
                        let logs_before = self.0.with_kernel_mut(|kernel| {
                            kernel
                                .kernel_get_system_state()
                                .system
                                .modules
                                .transaction_runtime()
                                .map(|runtime| runtime.logs.len())
                                .unwrap_or(0)
                        });

                        let rtn = self.0.with_kernel_mut(|kernel| {
                            SystemService::new(kernel).$func_ident( $($input_ident),* )
                        });

                        self.0.with_kernel_mut(|kernel| {
                            let logs_after = kernel
                                .kernel_get_system_state()
                                .system
                                .modules
                                .transaction_runtime()
                                .map(|runtime| runtime.logs.len())
                                .unwrap_or(0);

                            if logs_before != logs_after {
                                for (level, message) in kernel
                                    .kernel_get_system_state()
                                    .system
                                    .modules
                                    .transaction_runtime()
                                    .map(|module| module.logs.iter())
                                    .unwrap_or_default()
                                    .into_iter()
                                    .skip(logs_before)
                                {
                                    println!("[{}]: {}", level, message)
                                }
                            }
                        });

                        rtn
                    }
                )*
            }
        )*
    };
}
implement_system_api! {
    SystemApi: {},
    SystemActorApi: {
        actor_get_blueprint_id: (&mut self) -> Result<BlueprintId, RuntimeError>,
        actor_open_field: (
            &mut self,
            object_handle: ActorStateHandle,
            field: FieldIndex,
            flags: LockFlags,
        ) -> Result<FieldHandle, RuntimeError>,
        actor_is_feature_enabled: (
            &mut self,
            object_handle: ActorStateHandle,
            feature: &str,
        ) -> Result<bool, RuntimeError>,
        actor_get_node_id: (&mut self, ref_handle: ActorRefHandle) -> Result<NodeId, RuntimeError>,
        actor_emit_event: (
            &mut self,
            event_name: String,
            event_data: Vec<u8>,
            event_flags: EventFlags,
        ) -> Result<(), RuntimeError>
    },
    SystemActorIndexApi: {
        actor_index_insert: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            key: Vec<u8>,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        actor_index_remove: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            key: Vec<u8>,
        ) -> Result<Option<Vec<u8>>, RuntimeError>,
        actor_index_scan_keys: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            limit: u32,
        ) -> Result<Vec<Vec<u8>>, RuntimeError>,
        actor_index_drain: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            limit: u32,
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, RuntimeError>,
    },
    SystemActorKeyValueEntryApi: {
        actor_open_key_value_entry: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            key: &Vec<u8>,
            flags: LockFlags,
        ) -> Result<KeyValueEntryHandle, RuntimeError>,
        actor_remove_key_value_entry: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            key: &Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    SystemActorSortedIndexApi: {
        actor_sorted_index_insert: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            sorted_key: SortedKey,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        actor_sorted_index_remove: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            sorted_key: &SortedKey,
        ) -> Result<Option<Vec<u8>>, RuntimeError>,
        actor_sorted_index_scan: (
            &mut self,
            object_handle: ActorStateHandle,
            collection_index: CollectionIndex,
            count: u32,
        ) -> Result<Vec<(SortedKey, Vec<u8>)>, RuntimeError>,
    },
    SystemBlueprintApi: {
        call_function: (
            &mut self,
            package_address: PackageAddress,
            blueprint_name: &str,
            function_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
        resolve_blueprint_type: (
            &mut self,
            blueprint_type_id: &BlueprintTypeIdentifier,
        ) -> Result<(Rc<VersionedScryptoSchema>, ScopedTypeId), RuntimeError>
    },
    SystemFieldApi: {
        field_read: (&mut self, handle: field_api::FieldHandle) -> Result<Vec<u8>, RuntimeError>,
        field_write: (&mut self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), RuntimeError>,
        field_lock: (&mut self, handle: FieldHandle) -> Result<(), RuntimeError>,
        field_close: (&mut self, handle: FieldHandle) -> Result<(), RuntimeError>
    },
    SystemKeyValueEntryApi: {
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
    SystemKeyValueStoreApi: {
        key_value_store_new: (&mut self, data_schema: KeyValueStoreDataSchema) -> Result<NodeId, RuntimeError>,
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
    SystemObjectApi: {
        new_object: (
            &mut self,
            blueprint_ident: &str,
            features: Vec<&str>,
            generic_args: GenericArgs,
            fields: IndexMap<FieldIndex, FieldValue>,
            kv_entries: IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
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
            node_id: NodeId,
            modules: IndexMap<AttachedModuleId, NodeId>,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Result<GlobalAddress, RuntimeError>,
        globalize_with_address_and_create_inner_object_and_emit_event: (
            &mut self,
            node_id: NodeId,
            modules: IndexMap<AttachedModuleId, NodeId>,
            address_reservation: GlobalAddressReservation,
            inner_object_blueprint: &str,
            inner_object_fields: IndexMap<u8, FieldValue>,
            event_name: &str,
            event_data: Vec<u8>,
        ) -> Result<(GlobalAddress, NodeId), RuntimeError>,
        call_method: (
            &mut self,
            receiver: &NodeId,
            method_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
        call_direct_access_method: (
            &mut self,
            receiver: &NodeId,
            method_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
        call_module_method: (
            &mut self,
            receiver: &NodeId,
            module_id: AttachedModuleId,
            method_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    SystemExecutionTraceApi: {
        update_instruction_index: (&mut self, new_index: usize) -> Result<(), RuntimeError>,
    },
    SystemTransactionRuntimeApi: {
        bech32_encode_address: (&mut self, address: GlobalAddress) -> Result<String, RuntimeError>,
        get_transaction_hash: (&mut self) -> Result<Hash, RuntimeError>,
        generate_ruid: (&mut self) -> Result<[u8; 32], RuntimeError>,
        emit_log: (&mut self, level: Level, message: String) -> Result<(), RuntimeError>,
        panic: (&mut self, message: String) -> Result<(), RuntimeError>,
    },
    SystemCostingApi: {
        start_lock_fee: (&mut self, amount: Decimal, contingent: bool) -> Result<bool, RuntimeError>,
        lock_fee: (
            &mut self,
            locked_fee: LiquidFungibleResource,
            contingent: bool,
        ) -> (),
        consume_cost_units: (&mut self, costing_entry: ClientCostingEntry) -> Result<(), RuntimeError>,
        execution_cost_unit_limit: (&mut self) -> Result<u32, RuntimeError>,
        execution_cost_unit_price: (&mut self) -> Result<Decimal, RuntimeError>,
        finalization_cost_unit_limit: (&mut self) -> Result<u32, RuntimeError>,
        finalization_cost_unit_price: (&mut self) -> Result<Decimal, RuntimeError>,
        usd_price: (&mut self) -> Result<Decimal, RuntimeError>,
        max_per_function_royalty_in_xrd: (&mut self) -> Result<Decimal, RuntimeError>,
        tip_percentage_truncated: (&mut self) -> Result<u32, RuntimeError>,
        fee_balance: (&mut self) -> Result<Decimal, RuntimeError>,
    }
}
