pub mod actor_api;
pub mod actor_index_api;
pub mod actor_key_value_entry_api;
pub mod actor_sorted_index_api;
pub mod blueprint_api;
pub mod costing_api;
pub mod crypto_utils_api;
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
pub use costing_api::ClientCostingApi;
pub use crypto_utils_api::ClientCryptoUtilsApi;
pub use execution_trace_api::ClientExecutionTraceApi;
pub use field_api::*;
pub use key_value_entry_api::*;
pub use key_value_store_api::*;
pub use object_api::*;
pub use transaction_runtime_api::ClientTransactionRuntimeApi;

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
    + ClientCryptoUtilsApi<E>
{
}
