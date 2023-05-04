pub mod actor_api;
pub mod blueprint_api;
pub mod component;
pub mod field_lock_api;
pub mod actor_index_api;
pub mod kernel_modules;
pub mod key_value_entry_api;
pub mod key_value_store_api;
pub mod node_modules;
pub mod object_api;
pub mod actor_sorted_index_api;

// Re-exports
pub use crate::api::actor_sorted_index_api::ClientActorSortedIndexApi;
pub use actor_api::{ClientActorApi, ClientActorKeyValueEntryApi};
pub use blueprint_api::ClientBlueprintApi;
pub use field_lock_api::ClientFieldLockApi;
pub use field_lock_api::LockFlags;
pub use kernel_modules::auth_api::ClientAuthApi;
pub use kernel_modules::costing_api::ClientCostingApi;
pub use kernel_modules::event_api::ClientEventApi;
pub use kernel_modules::execution_trace_api::ClientExecutionTraceApi;
pub use kernel_modules::logger_api::ClientLoggerApi;
pub use kernel_modules::transaction_limits_api::ClientTransactionLimitsApi;
pub use kernel_modules::transaction_runtime_api::ClientTransactionRuntimeApi;
pub use object_api::*;
use radix_engine_interface::api::actor_index_api::ClientActorIndexApi;
use radix_engine_interface::api::key_value_entry_api::ClientKeyValueEntryApi;
use radix_engine_interface::api::key_value_store_api::ClientKeyValueStoreApi;

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
    + ClientEventApi<E>
    + ClientLoggerApi<E>
    + ClientTransactionLimitsApi<E>
    + ClientTransactionRuntimeApi<E>
    + ClientExecutionTraceApi<E>
    + ClientAuthApi<E>
{
}
