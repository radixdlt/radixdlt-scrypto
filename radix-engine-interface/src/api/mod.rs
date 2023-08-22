pub mod actor_api;
pub mod actor_index_api;
pub mod actor_key_value_entry_api;
pub mod actor_sorted_index_api;
pub mod blueprint_api;
pub mod field_api;
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
pub use field_api::ClientFieldApi;
pub use field_api::LockFlags;
use key_value_entry_api::ClientKeyValueEntryApi;
use key_value_store_api::ClientKeyValueStoreApi;
pub use object_api::*;
pub use system_modules::costing_api::ClientCostingApi;
pub use system_modules::execution_trace_api::ClientExecutionTraceApi;
pub use system_modules::transaction_runtime_api::ClientTransactionRuntimeApi;

pub type ActorStateHandle = u32;

pub const ACTOR_STATE_SELF: ActorStateHandle = 0u32;
pub const ACTOR_STATE_OUTER_OBJECT: ActorStateHandle = 1u32;

pub type ActorRefHandle = u32;

pub const ACTOR_REF_SELF: ActorRefHandle = 0u32;
pub const ACTOR_REF_OUTER: ActorRefHandle = 1u32;
pub const ACTOR_REF_GLOBAL: ActorRefHandle = 2u32;
pub const ACTOR_REF_AUTH_ZONE: ActorRefHandle = 8u32;

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
    + ClientFieldApi<E>
    + ClientBlueprintApi<E>
    + ClientCostingApi<E>
    + ClientTransactionRuntimeApi<E>
    + ClientExecutionTraceApi<E>
{
}
