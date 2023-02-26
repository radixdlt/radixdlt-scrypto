pub mod actor_api;
pub mod component;
pub mod component_api;
pub mod events_api;
pub mod node_api;
pub mod node_modules;
pub mod package;
pub mod package_api;
pub mod substate_api;
pub mod types;
pub mod unsafe_api;

// Re-exports
pub use actor_api::ClientActorApi;
pub use component_api::ClientComponentApi;
pub use events_api::ClientEventsApi;
pub use node_api::ClientNodeApi;
pub use package_api::ClientPackageApi;
pub use substate_api::ClientSubstateApi;
pub use substate_api::LockFlags;
pub use unsafe_api::ClientUnsafeApi;

/// Interface of the system, for blueprints and Node modules.
///
/// For WASM blueprints, only a subset of the API is exposed at the moment.
pub trait ClientApi<E: sbor::rust::fmt::Debug>:
    ClientActorApi<E>
    + ClientComponentApi<E>
    + ClientPackageApi<E>
    + ClientNodeApi<E>
    + ClientSubstateApi<E>
    + ClientUnsafeApi<E>
    + ClientEventsApi<E>
{
}
