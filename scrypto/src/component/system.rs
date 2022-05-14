use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::*;
use sbor::rust::vec::Vec;

use crate::buffer::*;
use crate::component::*;
use crate::core::SNodeRef;
use crate::engine::{api::*, call_engine};
use crate::resource::AccessRules;

/// Represents the Radix Engine component subsystem.
///
/// Notes:
/// - No mutability semantics are enforced
/// - It's not thread safe
///
/// TODO: research if need to introduce `&` and `&mut` for packages and components.
/// TODO: add mutex/lock for non-WebAssembly target
pub struct ComponentSystem {
    packages: HashMap<PackageAddress, Package>,
    components: HashMap<ComponentAddress, Component>,
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
    pub fn get_package(&mut self, package_address: PackageAddress) -> &Package {
        self.packages
            .entry(package_address)
            .or_insert(Package(package_address))
    }

    /// Returns a reference to a component.
    pub fn get_component(&mut self, component_address: ComponentAddress) -> &Component {
        self.components
            .entry(component_address)
            .or_insert(Component(component_address))
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> PackageAddress {
        let input = RadixEngineInput::InvokeSNode(
            SNodeRef::PackageStatic,
            scrypto_encode(&PackageFunction::Publish(code.to_vec())),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    /// Instantiates a component.
    pub fn instantiate_component<T: ComponentState>(
        &mut self,
        blueprint_name: &str,
        authorization: Vec<AccessRules>,
        state: T,
    ) -> ComponentAddress {
        let input = RadixEngineInput::CreateComponent(
            blueprint_name.to_owned(),
            scrypto_encode(&state),
            authorization,
        );
        let output: ComponentAddress = call_engine(input);

        output
    }

    /// Instantiates a component.
    pub fn to_component_state_with_auth<T: ComponentState>(
        &self,
        blueprint_name: &str,
        state: T,
    ) -> LocalComponent {
        LocalComponent::new(blueprint_name.to_owned(), scrypto_encode(&state))
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

/// This macro creates a `&Package` from a `PackageAddress` via the
/// Radix Engine component subsystem.
#[macro_export]
macro_rules! borrow_package {
    ($id:expr) => {
        component_system().get_package($id)
    };
}

/// This macro converts a `ComponentAddress` into a `&Component` via the
/// Radix Engine component subsystem.
#[macro_export]
macro_rules! borrow_component {
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

        let component = borrow_component!(ComponentAddress([0u8; 26]));
        let component_same_id = borrow_component!(ComponentAddress([0u8; 26]));
        let component_different_id = borrow_component!(ComponentAddress([1u8; 26]));

        assert_eq!(ComponentAddress([0u8; 26]), component.0);
        assert_eq!(ComponentAddress([0u8; 26]), component_same_id.0);
        assert_eq!(ComponentAddress([1u8; 26]), component_different_id.0);
    }

    #[test]
    fn test_package_macro() {
        init_component_system(ComponentSystem::new());

        let package = borrow_package!(PackageAddress([0u8; 26]));
        let package_same_id = borrow_package!(PackageAddress([0u8; 26]));
        let package_different_id = borrow_package!(PackageAddress([1u8; 26]));

        assert_eq!(PackageAddress([0u8; 26]), package.0);
        assert_eq!(PackageAddress([0u8; 26]), package_same_id.0);
        assert_eq!(PackageAddress([1u8; 26]), package_different_id.0);
    }
}
