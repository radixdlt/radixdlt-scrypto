pub mod actor_api;
pub mod component;
pub mod component_api;
pub mod deref_api;
pub mod event_api;
pub mod native_invoke_api;
pub mod node_api;
pub mod node_modules;
pub mod package;
pub mod package_api;
pub mod substate_api;
pub mod types;

// Re-exports
pub use actor_api::ClientActorApi;
pub use component_api::ClientComponentApi;
pub use deref_api::ClientDerefApi;
pub use event_api::ClientEventApi;
pub use native_invoke_api::ClientNativeInvokeApi;
pub use node_api::ClientNodeApi;
pub use package_api::ClientPackageApi;
pub use substate_api::ClientSubstateApi;

/// Interface of the system, for blueprints and Node modules.
///
/// For WASM blueprints, only a subset of the API is exposed at the moment.
pub trait ClientApi<E: sbor::rust::fmt::Debug>:
    ClientActorApi<E>
    + ClientComponentApi<E>
    + ClientPackageApi<E>
    + ClientNativeInvokeApi<E>  // TODO: restrict and protect native invocations
    + ClientNodeApi<E>
    + ClientSubstateApi<E>
    + ClientDerefApi<E>  // TODO: remove
    + ClientEventApi<E>
{
}
