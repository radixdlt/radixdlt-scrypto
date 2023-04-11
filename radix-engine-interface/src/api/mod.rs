pub mod actor_api;
pub mod component;
pub mod kernel_modules;
pub mod node_modules;
pub mod object_api;
pub mod substate_api;

// Re-exports
pub use actor_api::ClientActorApi;
pub use kernel_modules::auth_api::ClientAuthApi;
pub use kernel_modules::costing_api::ClientCostingApi;
pub use kernel_modules::event_api::ClientEventApi;
pub use kernel_modules::execution_trace_api::ClientExecutionTraceApi;
pub use kernel_modules::logger_api::ClientLoggerApi;
pub use kernel_modules::transaction_limits_api::ClientTransactionLimitsApi;
pub use kernel_modules::transaction_runtime_api::ClientTransactionRuntimeApi;
pub use object_api::ClientObjectApi;
pub use substate_api::ClientSubstateApi;
pub use substate_api::LockFlags;

/// Interface of the system, for blueprints and Node modules.
///
/// For WASM blueprints, only a subset of the API is exposed at the moment.
pub trait ClientApi<E: sbor::rust::fmt::Debug>:
    ClientActorApi<E>
    + ClientObjectApi<E>
    + ClientSubstateApi<E>
    + ClientCostingApi<E>
    + ClientEventApi<E>
    + ClientLoggerApi<E>
    + ClientTransactionLimitsApi<E>
    + ClientTransactionRuntimeApi<E>
    + ClientExecutionTraceApi<E>
    + ClientAuthApi<E>
{
}
