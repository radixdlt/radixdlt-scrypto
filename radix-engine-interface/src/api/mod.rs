pub mod actor_api;
pub mod actor_index_api;
pub mod actor_key_value_entry_api;
pub mod actor_sorted_index_api;
pub mod blueprint_api;
pub mod field_lock_api;
pub mod key_value_entry_api;
pub mod key_value_store_api;
pub mod node_modules;
pub mod object_api;
pub mod system_modules;

// Re-exports
pub use actor_api::ClientActorApi;
use actor_index_api::ClientActorIndexApi;
pub use actor_key_value_entry_api::ClientActorKeyValueEntryApi;
pub use actor_sorted_index_api::ClientActorSortedIndexApi;
pub use blueprint_api::ClientBlueprintApi;
pub use field_lock_api::ClientFieldLockApi;
pub use field_lock_api::LockFlags;
use key_value_entry_api::ClientKeyValueEntryApi;
use key_value_store_api::ClientKeyValueStoreApi;
pub use object_api::*;
pub use system_modules::auth_api::ClientAuthApi;
pub use system_modules::costing_api::ClientCostingApi;
pub use system_modules::execution_trace_api::ClientExecutionTraceApi;
pub use system_modules::transaction_runtime_api::ClientTransactionRuntimeApi;

pub type ObjectHandle = u32;

pub const OBJECT_HANDLE_SELF: ObjectHandle = 0u32;
pub const OBJECT_HANDLE_OUTER_OBJECT: ObjectHandle = 1u32;

pub type FieldIndex = u8;
pub type CollectionIndex = u8;

/// Interface of the system, for blueprints and Node modules.
///
/// For WASM blueprints, only a subset of the API is exposed at the moment.
pub trait ClientApi<E: sbor::rust::fmt::Debug>:
    ClientActorApi<E>
    + ClientActorKeyValueEntryApi<E>
    + ClientObjectApi<E>
    + ClientKeyValueStoreApi<E>
    + ClientKeyValueEntryApi<E>
    + ClientActorSortedIndexApi<E>
    + ClientActorIndexApi<E>
    + ClientFieldLockApi<E>
    + ClientBlueprintApi<E>
    + ClientCostingApi<E>
    + ClientTransactionRuntimeApi<E>
    + ClientExecutionTraceApi<E>
    + ClientAuthApi<E>
{
}
