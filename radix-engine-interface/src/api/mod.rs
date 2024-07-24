pub mod actor_api;
pub mod actor_index_api;
pub mod actor_key_value_entry_api;
pub mod actor_sorted_index_api;
pub mod blueprint_api;
pub mod costing_api;
pub mod execution_trace_api;
pub mod field_api;
pub mod key_value_entry_api;
pub mod key_value_store_api;
pub mod object_api;
pub mod transaction_runtime_api;

// Re-exports
pub use actor_api::*;
pub use actor_index_api::*;
pub use actor_key_value_entry_api::*;
pub use actor_sorted_index_api::*;
pub use blueprint_api::*;
pub use costing_api::*;
pub use execution_trace_api::*;
pub use field_api::*;
pub use key_value_entry_api::*;
pub use key_value_store_api::*;
pub use object_api::*;
pub use transaction_runtime_api::*;

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

use radix_common::prelude::*;

pub trait SystemApiError: fmt::Debug + ScryptoCategorize + ScryptoDecode {}

/// Interface of the system, for blueprints and object modules.
///
/// For WASM blueprints, only a subset of the API is exposed at the moment.
pub trait SystemApi<E: SystemApiError>:
    SystemActorApi<E>
    + SystemActorKeyValueEntryApi<E>
    + SystemObjectApi<E>
    + SystemKeyValueStoreApi<E>
    + SystemKeyValueEntryApi<E>
    + SystemActorSortedIndexApi<E>
    + SystemActorIndexApi<E>
    + SystemFieldApi<E>
    + SystemBlueprintApi<E>
    + SystemCostingApi<E>
    + SystemTransactionRuntimeApi<E>
    + SystemExecutionTraceApi<E>
{
}
