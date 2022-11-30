use crate::abi::BlueprintAbi;
use crate::component::*;
use crate::engine::scrypto_env::ScryptoEnv;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::ScryptoRENode;
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::model::*;
use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::runtime::Runtime;

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
    components: HashMap<ComponentAddress, BorrowedGlobalComponent>,
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
    pub fn get_component(
        &mut self,
        component_address: ComponentAddress,
    ) -> &BorrowedGlobalComponent {
        self.components
            .entry(component_address)
            .or_insert(BorrowedGlobalComponent(component_address))
    }

    /// Publishes a package.
    pub fn publish_package(
        &mut self,
        _code: Vec<u8>,
        _abi: HashMap<String, BlueprintAbi>,
    ) -> PackageAddress {
        todo!("Not supported yet due to lack of dynamic blob creation")
    }

    /// Instantiates a component.
    pub fn create_component<T: ComponentState<C>, C: LocalComponent>(
        &self,
        blueprint_name: &str,
        state: T,
    ) -> Component {
        let mut env = ScryptoEnv;
        let node_id = env
            .sys_create_node(ScryptoRENode::Component(
                Runtime::package_address(),
                blueprint_name.to_string(),
                RoyaltyConfig::default(),
                scrypto_encode(&state).unwrap(),
            ))
            .unwrap();
        Component(node_id.into())
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
