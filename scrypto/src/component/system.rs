use crate::buffer::*;
use crate::component::*;
use crate::engine::{api::*, call_engine};
use crate::prelude::NonFungibleAddress;
use crate::prelude::String;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::*;

/// Represents the Radix Engine component subsystem.
///
/// Notes:
/// - No mutability semantics are enforced
/// - It's not thread safe
///
/// TODO: research if need to introduce `&` and `&mut` for packages and components.
/// TODO: add mutex/lock for non-WebAssembly target
pub struct ComponentSystem {
    packages: HashMap<PackageId, Package>,
    components: HashMap<ComponentId, Component>,
}

impl ComponentSystem {
    /// Creates a component system.
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            components: HashMap::new(),
        }
    }

    /// Returns a reference to a package.
    pub fn get_package(&mut self, package_id: PackageId) -> &Package {
        self.packages
            .entry(package_id)
            .or_insert(Package(package_id))
    }

    /// Returns a reference to a component.
    pub fn get_component(&mut self, component_id: ComponentId) -> &Component {
        self.components
            .entry(component_id)
            .or_insert(Component(component_id))
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> PackageId {
        let input = PublishPackageInput {
            code: code.to_vec(),
        };
        let output: PublishPackageOutput = call_engine(PUBLISH_PACKAGE, input);

        output.package_id
    }

    /// Instantiates a component.
    pub fn instantiate_component<T: ComponentState>(
        &mut self,
        package_id: PackageId,
        sys_auth: HashMap<String, NonFungibleAddress>,
        state: T,
    ) -> ComponentId {
        let input = CreateComponentInput {
            package_id,
            blueprint_name: T::blueprint_name().to_owned(),
            state: scrypto_encode(&state),
            sys_auth
        };
        let output: CreateComponentOutput = call_engine(CREATE_COMPONENT, input);

        output.component_id
    }
}

static mut COMPONENT_SYSTEM: Option<ComponentSystem> = None;

/// Initializes component subsystem.
pub fn init_component_system(system: ComponentSystem) {
    unsafe { COMPONENT_SYSTEM = Some(system) }
}

/// Returns the component subsystem.
pub fn component_system() -> &'static mut ComponentSystem {
    unsafe { COMPONENT_SYSTEM.as_mut().unwrap() }
}

/// This macro creates a `&Package` from a `PackageId` via the
/// Radix Engine component subsystem.
#[macro_export]
macro_rules! package {
    ($id:expr) => {
        component_system().get_package($id)
    };
}

/// This macro converts a `ComponentId` into a `&Component` via the
/// Radix Engine component subsystem.
#[macro_export]
macro_rules! component {
    ($id:expr) => {
        component_system().get_component($id)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_macro() {
        init_component_system(ComponentSystem::new());

        let component = component!(ComponentId([0u8; 26]));
        let component_same_id = component!(ComponentId([0u8; 26]));
        let component_different_id = component!(ComponentId([1u8; 26]));

        assert_eq!(ComponentId([0u8; 26]), component.0);
        assert_eq!(ComponentId([0u8; 26]), component_same_id.0);
        assert_eq!(ComponentId([1u8; 26]), component_different_id.0);
    }

    #[test]
    fn test_package_macro() {
        init_component_system(ComponentSystem::new());

        let package = package!(PackageId([0u8; 26]));
        let package_same_id = package!(PackageId([0u8; 26]));
        let package_different_id = package!(PackageId([1u8; 26]));

        assert_eq!(PackageId([0u8; 26]), package.0);
        assert_eq!(PackageId([0u8; 26]), package_same_id.0);
        assert_eq!(PackageId([1u8; 26]), package_different_id.0);
    }
}
