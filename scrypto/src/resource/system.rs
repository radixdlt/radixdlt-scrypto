use crate::args;
use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;
use crate::engine::{api::*, call_engine};
use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::rust::string::ToString;

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
    pub fn get_resource_manager(&mut self, resource_address: ResourceAddress) -> &mut ResourceManager {
        self.resource_managers
            .entry(resource_address)
            .or_insert(ResourceManager(resource_address))
    }

    /// Creates a new resource with the given parameters.
    ///
    /// A bucket is returned iif an initial supply is provided.
    pub fn new_resource(
        &mut self,
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        authorization: HashMap<ResourceMethod, (MethodAuth, Mutability)>,
        mint_params: Option<MintParams>,
    ) -> (ResourceAddress, Option<Bucket>) {

        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceStatic,
            function: "main".to_string(),
            args: args![ResourceManagerFunction::Create(resource_type, metadata, authorization, mint_params)],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_resource_manager_macro() {
        init_resource_system(ResourceSystem::new());

        let resource_manager = borrow_resource_manager!(ResourceAddress([0u8; 26]));
        let resource_manager_same_id = borrow_resource_manager!(ResourceAddress([0u8; 26]));
        let resource_manager_different_id = borrow_resource_manager!(ResourceAddress([1u8; 26]));

        assert_eq!(ResourceAddress([0u8; 26]), resource_manager.0);
        assert_eq!(ResourceAddress([0u8; 26]), resource_manager_same_id.0);
        assert_eq!(ResourceAddress([1u8; 26]), resource_manager_different_id.0);
    }
}
