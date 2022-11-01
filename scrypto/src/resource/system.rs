use sbor::rust::collections::HashMap;
use sbor::rust::string::String;

use crate::buffer::scrypto_encode;
use crate::engine::{types::*, utils::*};
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

    /// Creates a new resource with the given config.
    ///
    /// A bucket is returned iif an initial supply is provided.
    pub fn new_resource(
        &mut self,
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        mint_params: Option<MintParams>,
    ) -> (ResourceAddress, Option<Bucket>) {
        let input = RadixEngineInput::InvokeNativeFunction(
            NativeFunction::ResourceManager(ResourceManagerFunction::Create),
            scrypto_encode(&ResourceManagerCreateInvocation {
                resource_type,
                metadata,
                access_rules,
                mint_params,
            }),
        );
        call_engine(input)
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
