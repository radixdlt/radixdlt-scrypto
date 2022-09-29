use sbor::rust::collections::HashMap;
use sbor::rust::string::String;

use crate::buffer::scrypto_encode;
use crate::core::{FnIdent, FunctionIdent, NativeFunctionFnIdent, ResourceManagerFunctionFnIdent};
use crate::engine::{api::*, call_engine};
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
        let input = RadixEngineInput::Invoke(
            FnIdent::Function(FunctionIdent::Native(
                NativeFunctionFnIdent::ResourceManager(ResourceManagerFunctionFnIdent::Create),
            )),
            scrypto_encode(&ResourceManagerCreateInput {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_resource_manager_macro() {
        init_resource_system(ResourceSystem::new());

        let resource_manager = borrow_resource_manager!(ResourceAddress::Normal([0u8; 26]));
        let resource_manager_same_id = borrow_resource_manager!(ResourceAddress::Normal([0u8; 26]));
        let resource_manager_different_id =
            borrow_resource_manager!(ResourceAddress::Normal([1u8; 26]));

        assert_eq!(ResourceAddress::Normal([0u8; 26]), resource_manager.0);
        assert_eq!(
            ResourceAddress::Normal([0u8; 26]),
            resource_manager_same_id.0
        );
        assert_eq!(
            ResourceAddress::Normal([1u8; 26]),
            resource_manager_different_id.0
        );
    }
}
