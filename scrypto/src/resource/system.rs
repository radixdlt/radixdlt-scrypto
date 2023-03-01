use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::*;
use sbor::rust::collections::HashMap;

use crate::resource::*;

/// Represents the Radix Engine resource subsystem.
///
/// Notes:
/// - No mutability semantics are enforced
/// - It's not thread safe
///
/// TODO: research if need to introduce `&` and `&mut` for resource managers.
/// TODO: add mutex/lock for non-WebAssembly target
pub struct ResourceSystem {
    resource_managers: HashMap<ResourceAddress, ResourceManager>,
}

impl ResourceSystem {
    /// Creates a resource system.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            resource_managers: HashMap::new(),
        }
    }

    /// Returns a reference to a resource manager.
    pub fn get_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> &mut ResourceManager {
        self.resource_managers
            .entry(resource_address)
            .or_insert(ResourceManager(resource_address))
    }
}

static mut RESOURCE_SYSTEM: Option<ResourceSystem> = None;

/// Initializes resource subsystem.
pub fn init_resource_system(system: ResourceSystem) {
    unsafe { RESOURCE_SYSTEM = Some(system) }
}

/// Returns the resource subsystem.
pub fn resource_system() -> &'static mut ResourceSystem {
    unsafe { RESOURCE_SYSTEM.as_mut().unwrap() }
}

/// This macro creates a `&ResourceManager` from a `ResourceAddress` via the
/// Radix Engine resource subsystem.
#[macro_export]
macro_rules! borrow_resource_manager {
    ($id:expr) => {
        resource_system().get_resource_manager($id)
    };
}
