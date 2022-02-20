mod component;
mod lazy_map;
mod package;

pub use component::{ComponentId, ComponentState, ParseComponentIdError};
pub use lazy_map::{LazyMap, ParseLazyMapError};
pub use package::{PackageId, ParsePackageIdError};

use crate::buffer::*;
use crate::engine::{api::*, call_engine};
use crate::rust::borrow::ToOwned;

/// Instantiates a component.
pub fn instantiate_component<T: ComponentState>(package_id: PackageId, state: T) -> ComponentId {
    let input = CreateComponentInput {
        package_id,
        blueprint_name: T::blueprint_name().to_owned(),
        state: scrypto_encode(&state),
    };
    let output: CreateComponentOutput = call_engine(CREATE_COMPONENT, input);

    output.component_id
}

/// Publishes a package.
pub fn publish_package(code: &[u8]) -> PackageId {
    let input = PublishPackageInput {
        code: code.to_vec(),
    };
    let output: PublishPackageOutput = call_engine(PUBLISH_PACKAGE, input);

    output.package_id
}
