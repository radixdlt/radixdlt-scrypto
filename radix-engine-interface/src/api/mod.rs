pub mod actor_api;
pub mod blueprints;
pub mod component;
pub mod component_api;
pub mod deref_api;
pub mod invoke_api; // TODO: consider removing statically linked invocations.
pub mod kernel_modules; // TODO: remove
pub mod node_modules;
pub mod package;
pub mod package_api;
pub mod substate_api;
pub mod types;

// Re-exports
pub use actor_api::EngineActorApi;
pub use component_api::EngineComponentApi;
pub use deref_api::EngineDerefApi;
pub use invoke_api::{EngineInvokeApi, Invokable};
pub use package_api::EnginePackageApi;
pub use substate_api::EngineSubstateApi;

// Interface of the system, for blueprints and Node modules.
pub trait EngineApi<E: sbor::rust::fmt::Debug>:
    EngineActorApi<E>
    + EngineComponentApi<E>
    + EnginePackageApi
    + EngineInvokeApi<E>
    + EngineSubstateApi<E>
    + EngineDerefApi<E>
{
}
