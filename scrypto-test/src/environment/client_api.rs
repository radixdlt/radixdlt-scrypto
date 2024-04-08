//! This module has the implementation of the [`ClientApi`] for the [`TestEnvironment`] in order not
//! to clutter up the other modules.
//!
//! [`ClientApi`]: crate::prelude::ClientApi
//! [`TestEnvironment`]: crate::prelude::TestEnvironment

use super::*;
use radix_common::crypto::*;
use radix_common::data::scrypto::*;
use radix_common::math::*;
use radix_common::types::*;
use radix_common::*;
use radix_engine::errors::*;
use radix_engine::system::system::*;
use radix_engine_interface::api::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::*;
use radix_substate_store_interface::interface::*;
use sbor::prelude::*;

impl<D> ClientApi<RuntimeError> for TestEnvironment<D> where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static
{
}

impl<D> ClientActorApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn actor_get_blueprint_id(&mut self) -> Result<BlueprintId, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_get_blueprint_id()
            })
        })
    }
    #[inline]
    fn actor_open_field(
        &mut self,
        object_handle: ActorStateHandle,
        field: FieldIndex,
        flags: LockFlags,
    ) -> Result<FieldHandle, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_open_field(object_handle, field, flags)
            })
        })
    }
    #[inline]
    fn actor_is_feature_enabled(
        &mut self,
        object_handle: ActorStateHandle,
        feature: &str,
    ) -> Result<bool, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_is_feature_enabled(object_handle, feature)
            })
        })
    }
    #[inline]
    fn actor_get_node_id(&mut self, ref_handle: ActorRefHandle) -> Result<NodeId, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_get_node_id(ref_handle)
            })
        })
    }
    #[inline]
    fn actor_emit_event(
        &mut self,
        event_name: String,
        event_data: Vec<u8>,
        event_flags: EventFlags,
    ) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_emit_event(event_name, event_data, event_flags)
            })
        })
    }
}
impl<D> ClientActorIndexApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn actor_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_index_insert(object_handle, collection_index, key, buffer)
            })
        })
    }
    #[inline]
    fn actor_index_remove(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_index_remove(object_handle, collection_index, key)
            })
        })
    }
    #[inline]
    fn actor_index_scan_keys(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_index_scan_keys(object_handle, collection_index, limit)
            })
        })
    }
    #[inline]
    fn actor_index_drain(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_index_drain(object_handle, collection_index, limit)
            })
        })
    }
}
impl<D> ClientActorKeyValueEntryApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn actor_open_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_open_key_value_entry(
                    object_handle,
                    collection_index,
                    key,
                    flags,
                )
            })
        })
    }
    #[inline]
    fn actor_remove_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_remove_key_value_entry(object_handle, collection_index, key)
            })
        })
    }
}
impl<D> ClientActorSortedIndexApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn actor_sorted_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_sorted_index_insert(
                    object_handle,
                    collection_index,
                    sorted_key,
                    buffer,
                )
            })
        })
    }
    #[inline]
    fn actor_sorted_index_remove(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_sorted_index_remove(
                    object_handle,
                    collection_index,
                    sorted_key,
                )
            })
        })
    }
    #[inline]
    fn actor_sorted_index_scan(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<(SortedKey, Vec<u8>)>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .actor_sorted_index_scan(object_handle, collection_index, count)
            })
        })
    }
}
impl<D> ClientBlueprintApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .call_function(
                    package_address,
                    blueprint_name,
                    function_name,
                    args,
                )
            })
        })
    }
    #[inline]
    fn resolve_blueprint_type(
        &mut self,
        blueprint_type_id: &BlueprintTypeIdentifier,
    ) -> Result<(Rc<VersionedScryptoSchema>, ScopedTypeId), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .resolve_blueprint_type(blueprint_type_id)
            })
        })
    }
}
impl<D> ClientFieldApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn field_read(&mut self, handle: field_api::FieldHandle) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .field_read(handle)
            })
        })
    }
    #[inline]
    fn field_write(&mut self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .field_write(handle, buffer)
            })
        })
    }
    #[inline]
    fn field_lock(&mut self, handle: FieldHandle) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .field_lock(handle)
            })
        })
    }
    #[inline]
    fn field_close(&mut self, handle: FieldHandle) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .field_close(handle)
            })
        })
    }
}
impl<D> ClientKeyValueEntryApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn key_value_entry_get(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_entry_get(handle)
            })
        })
    }
    #[inline]
    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_entry_set(handle, buffer)
            })
        })
    }
    #[inline]
    fn key_value_entry_remove(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_entry_remove(handle)
            })
        })
    }
    #[inline]
    fn key_value_entry_lock(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_entry_lock(handle)
            })
        })
    }
    #[inline]
    fn key_value_entry_close(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_entry_close(handle)
            })
        })
    }
}
impl<D> ClientKeyValueStoreApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn key_value_store_new(
        &mut self,
        data_schema: KeyValueStoreDataSchema,
    ) -> Result<NodeId, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_store_new(data_schema)
            })
        })
    }
    #[inline]
    fn key_value_store_open_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_store_open_entry(node_id, key, flags)
            })
        })
    }
    #[inline]
    fn key_value_store_remove_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .key_value_store_remove_entry(node_id, key)
            })
        })
    }
}
impl<D> ClientObjectApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        features: Vec<&str>,
        generic_args: GenericArgs,
        fields: IndexMap<FieldIndex, FieldValue>,
        kv_entries: IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .new_object(
                    blueprint_ident,
                    features,
                    generic_args,
                    fields,
                    kv_entries,
                )
            })
        })
    }
    #[inline]
    fn drop_object(&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .drop_object(node_id)
            })
        })
    }
    #[inline]
    fn get_blueprint_id(&mut self, node_id: &NodeId) -> Result<BlueprintId, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .get_blueprint_id(node_id)
            })
        })
    }
    #[inline]
    fn get_outer_object(&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .get_outer_object(node_id)
            })
        })
    }
    #[inline]
    fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<(GlobalAddressReservation, GlobalAddress), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .allocate_global_address(blueprint_id)
            })
        })
    }
    #[inline]
    fn allocate_virtual_global_address(
        &mut self,
        blueprint_id: BlueprintId,
        global_address: GlobalAddress,
    ) -> Result<GlobalAddressReservation, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .allocate_virtual_global_address(blueprint_id, global_address)
            })
        })
    }
    #[inline]
    fn get_reservation_address(&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .get_reservation_address(node_id)
            })
        })
    }
    #[inline]
    fn globalize(
        &mut self,
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .globalize(node_id, modules, address_reservation)
            })
        })
    }
    #[inline]
    fn globalize_with_address_and_create_inner_object_and_emit_event(
        &mut self,
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: IndexMap<u8, FieldValue>,
        event_name: &str,
        event_data: Vec<u8>,
    ) -> Result<(GlobalAddress, NodeId), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .globalize_with_address_and_create_inner_object_and_emit_event(
                    node_id,
                    modules,
                    address_reservation,
                    inner_object_blueprint,
                    inner_object_fields,
                    event_name,
                    event_data,
                )
            })
        })
    }
    #[inline]
    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .call_method(receiver, method_name, args)
            })
        })
    }
    #[inline]
    fn call_direct_access_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .call_direct_access_method(receiver, method_name, args)
            })
        })
    }
    #[inline]
    fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: AttachedModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .call_module_method(receiver, module_id, method_name, args)
            })
        })
    }
}
impl<D> ClientExecutionTraceApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .update_instruction_index(new_index)
            })
        })
    }
}
impl<D> ClientTransactionRuntimeApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn bech32_encode_address(&mut self, address: GlobalAddress) -> Result<String, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .bech32_encode_address(address)
            })
        })
    }
    #[inline]
    fn get_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .get_transaction_hash()
            })
        })
    }
    #[inline]
    fn generate_ruid(&mut self) -> Result<[u8; 32], RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .generate_ruid()
            })
        })
    }
    #[inline]
    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .emit_log(level, message)
            })
        })
    }
    #[inline]
    fn panic(&mut self, message: String) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .panic(message)
            })
        })
    }
}
impl<D> ClientCostingApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn start_lock_fee(&mut self, amount: Decimal) -> Result<bool, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .start_lock_fee(amount)
            })
        })
    }
    #[inline]
    fn lock_fee(&mut self, locked_fee: LiquidFungibleResource, contingent: bool) -> () {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .lock_fee(locked_fee, contingent)
            })
        })
    }
    #[inline]
    fn consume_cost_units(
        &mut self,
        costing_entry: ClientCostingEntry,
    ) -> Result<(), RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .consume_cost_units(costing_entry)
            })
        })
    }
    #[inline]
    fn execution_cost_unit_limit(&mut self) -> Result<u32, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .execution_cost_unit_limit()
            })
        })
    }
    #[inline]
    fn execution_cost_unit_price(&mut self) -> Result<Decimal, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .execution_cost_unit_price()
            })
        })
    }
    #[inline]
    fn finalization_cost_unit_limit(&mut self) -> Result<u32, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .finalization_cost_unit_limit()
            })
        })
    }
    #[inline]
    fn finalization_cost_unit_price(&mut self) -> Result<Decimal, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .finalization_cost_unit_price()
            })
        })
    }
    #[inline]
    fn usd_price(&mut self) -> Result<Decimal, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .usd_price()
            })
        })
    }
    #[inline]
    fn max_per_function_royalty_in_xrd(&mut self) -> Result<Decimal, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .max_per_function_royalty_in_xrd()
            })
        })
    }
    #[inline]
    fn tip_percentage(&mut self) -> Result<u32, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .tip_percentage()
            })
        })
    }
    #[inline]
    fn fee_balance(&mut self) -> Result<Decimal, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .fee_balance()
            })
        })
    }
}
impl<D> ClientCryptoUtilsApi<RuntimeError> for TestEnvironment<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase + 'static,
{
    #[inline]
    fn bls12381_v1_verify(
        &mut self,
        message: &[u8],
        public_key: &Bls12381G1PublicKey,
        signature: &Bls12381G2Signature,
    ) -> Result<u32, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .bls12381_v1_verify(message, public_key, signature)
            })
        })
    }
    #[inline]
    fn bls12381_v1_aggregate_verify(
        &mut self,
        pub_keys_and_msgs: &[(Bls12381G1PublicKey, Vec<u8>)],
        signature: &Bls12381G2Signature,
    ) -> Result<u32, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .bls12381_v1_aggregate_verify(pub_keys_and_msgs, signature)
            })
        })
    }
    #[inline]
    fn bls12381_v1_fast_aggregate_verify(
        &mut self,
        message: &[u8],
        public_keys: &[Bls12381G1PublicKey],
        signature: &Bls12381G2Signature,
    ) -> Result<u32, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .bls12381_v1_fast_aggregate_verify(message, public_keys, signature)
            })
        })
    }
    #[inline]
    fn bls12381_g2_signature_aggregate(
        &mut self,
        signatures: &[Bls12381G2Signature],
    ) -> Result<Bls12381G2Signature, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .bls12381_g2_signature_aggregate(signatures)
            })
        })
    }
    #[inline]
    fn keccak256_hash(&mut self, data: &[u8]) -> Result<Hash, RuntimeError> {
        self.with_log_printing(|this| {
            this.0.with_kernel_mut(|kernel| {
                SystemService {
                    api: kernel,
                    phantom: PhantomData,
                }
                .keccak256_hash(data)
            })
        })
    }
}
