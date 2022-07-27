use crate::buffer::*;
use crate::component::package::Package;
use crate::component::*;
use crate::core::{DataAddress, DataRef, SNodeRef};
use crate::engine::{api::*, call_engine};
use crate::prelude::DataRefMut;
use sbor::rust::borrow::ToOwned;
use sbor::rust::cell::{Ref, RefCell, RefMut};
use sbor::rust::collections::*;
use sbor::rust::string::ToString;
use sbor::{Decode, Encode};

pub struct DataValue<V: 'static + Encode + Decode> {
    address: DataAddress,
    value: RefCell<Option<V>>,
}

impl<V: 'static + Encode + Decode> DataValue<V> {
    pub fn new(address: DataAddress) -> Self {
        Self {
            address,
            value: RefCell::new(Option::None),
        }
    }

    /// Returns a reference to component data
    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let value = RefMut::map(self.value.borrow_mut(), |v| {
            v.get_or_insert_with(|| {
                let input =
                    ::scrypto::engine::api::RadixEngineInput::ReadData(self.address.clone());
                let value: V = call_engine(input);
                value
            })
        });
        DataRefMut {
            address: self.address.clone(),
            value,
        }
    }

    pub fn get(&self) -> DataRef<V> {
        self.value.borrow_mut().get_or_insert_with(|| {
            let input = ::scrypto::engine::api::RadixEngineInput::ReadData(self.address.clone());
            let value: V = call_engine(input);
            value
        });

        let value = Ref::map(self.value.borrow(), |v| v.as_ref().unwrap());

        DataRef { value }
    }
}

/// Represents the Radix Engine component subsystem.
///
/// Notes:
/// - No mutability semantics are enforced
/// - It's not thread safe
///
/// TODO: research if need to introduce `&` and `&mut` for packages and components.
/// TODO: add mutex/lock for non-WebAssembly target
pub struct ComponentSystem {
    packages: HashMap<PackageAddress, BorrowedPackage>,
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
    pub fn get_package(&mut self, package_address: PackageAddress) -> &BorrowedPackage {
        self.packages
            .entry(package_address)
            .or_insert(BorrowedPackage(package_address))
    }

    /// Returns a reference to a component.
    pub fn get_component(&mut self, component_address: ComponentAddress) -> &Component {
        self.components
            .entry(component_address)
            .or_insert(Component(component_address))
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, package: Package) -> PackageAddress {
        let input = RadixEngineInput::InvokeSNode(
            SNodeRef::PackageStatic,
            "publish".to_string(),
            scrypto_encode(&PackagePublishInput { package }),
        );
        call_engine(input)
    }

    /// Instantiates a component.
    pub fn create_component<T: ComponentState<C>, C: LocalComponent>(
        &self,
        blueprint_name: &str,
        state: T,
    ) -> Component {
        let input =
            RadixEngineInput::CreateComponent(blueprint_name.to_owned(), scrypto_encode(&state));
        let component_address: ComponentAddress = call_engine(input);

        Component(component_address)
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

        let component = borrow_component!(ComponentAddress([0u8; 27]));
        let component_same_id = borrow_component!(ComponentAddress([0u8; 27]));
        let component_different_id = borrow_component!(ComponentAddress([1u8; 27]));

        assert_eq!(ComponentAddress([0u8; 27]), component.0);
        assert_eq!(ComponentAddress([0u8; 27]), component_same_id.0);
        assert_eq!(ComponentAddress([1u8; 27]), component_different_id.0);
    }

    #[test]
    fn test_package_macro() {
        init_component_system(ComponentSystem::new());

        let package = borrow_package!(PackageAddress([0u8; 27]));
        let package_same_id = borrow_package!(PackageAddress([0u8; 27]));
        let package_different_id = borrow_package!(PackageAddress([1u8; 27]));

        assert_eq!(PackageAddress([0u8; 27]), package.0);
        assert_eq!(PackageAddress([0u8; 27]), package_same_id.0);
        assert_eq!(PackageAddress([1u8; 27]), package_different_id.0);
    }
}
