use crate::engine::{api::*, call_engine};
use crate::resource::*;
use crate::rust::collections::HashMap;
use crate::rust::string::String;

/// Represents the Radix Engine resource subsystem.
///
/// Notes:
/// - No mutability semantics are enforced
/// - It's not thread safe
///
/// TODO: research if need to introduce `&` and `&mut` for resource definitions.
/// TODO: add mutex/lock for non-WebAssembly target
pub struct ResourceSystem {
    resource_defs: HashMap<ResourceAddress, ResourceDef>,
}

impl ResourceSystem {
    /// Creates a resource system.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            resource_defs: HashMap::new(),
        }
    }

    /// Returns a reference to a resource definition.
    pub fn get_resource_def(&mut self, resource_address: ResourceAddress) -> &ResourceDef {
        self.resource_defs
            .entry(resource_address)
            .or_insert(ResourceDef(resource_address))
    }

    /// Instantiate a resource definition with the given parameters.
    ///
    /// A bucket is returned iif an initial supply is provided.
    pub fn instantiate_resource_definition(
        &mut self,
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        authorities: HashMap<ResourceAddress, u64>,
        mint_params: Option<MintParams>,
    ) -> (ResourceAddress, Option<Bucket>) {
        let input = CreateResourceInput {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorities,
            mint_params,
        };
        let output: CreateResourceOutput = call_engine(CREATE_RESOURCE, input);

        (
            output.resource_address,
            output.bucket_id.map(|id| Bucket(id)),
        )
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

/// This macro creates a `&ResourceDef` from a `ResourceAddress` via the
/// Radix Engine resource subsystem.
#[macro_export]
macro_rules! resource_def {
    ($id:expr) => {
        resource_system().get_resource_def($id)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_resource_def_macro() {
        init_resource_system(ResourceSystem::new());

        let resource_def = resource_def!(ResourceAddress([0u8; 26]));
        let resource_def_same_id = resource_def!(ResourceAddress([0u8; 26]));
        let resource_def_different_id = resource_def!(ResourceAddress([1u8; 26]));

        assert_eq!(ResourceAddress([0u8; 26]), resource_def.0);
        assert_eq!(ResourceAddress([0u8; 26]), resource_def_same_id.0);
        assert_eq!(ResourceAddress([1u8; 26]), resource_def_different_id.0);
    }
}
