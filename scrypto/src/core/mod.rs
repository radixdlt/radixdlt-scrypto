mod account;
mod blueprint;
mod call;
mod component;
mod context;
mod lazy_map;
mod logger;
mod package;

pub use account::Account;
pub use blueprint::Blueprint;
pub use call::{call_function, call_method};
pub use component::Component;
pub use context::Context;
pub use lazy_map::LazyMap;
pub use logger::Logger;
pub use package::Package;

/// Represents the state of a component.
pub trait State: sbor::Encode + sbor::Decode {
    fn name() -> &'static str;

    fn instantiate(self) -> Component;
}
