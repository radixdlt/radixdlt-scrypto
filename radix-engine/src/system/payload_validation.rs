use crate::kernel::kernel_api::KernelApi;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT,
    NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use radix_engine_interface::constants::*;
use sbor::rust::prelude::*;

use super::node_modules::type_info::TypeInfoSubstate;
use super::system::SystemService;
use super::system_callback::SystemConfig;
use super::system_callback_api::SystemCallbackObject;

//==================
// ADAPTERS
//==================

pub struct SystemServiceTypeInfoLookup<
    's,
    'a,
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
> {
    system_service: RefCell<&'s mut SystemService<'a, Y, V>>,
}

impl<'s, 'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>
    SystemServiceTypeInfoLookup<'s, 'a, Y, V>
{
    pub fn new(system_service: &'s mut SystemService<'a, Y, V>) -> Self {
        Self {
            system_service: system_service.into(),
        }
    }
}

impl<'s, 'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject> TypeInfoLookup
    for SystemServiceTypeInfoLookup<'s, 'a, Y, V>
{
    fn get_node_type_info(&self, node_id: &NodeId) -> Option<TypeInfoForValidation> {
        let type_info = self
            .system_service
            .borrow_mut()
            .get_node_type_info(&node_id)?;
        let mapped = match type_info {
            TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => {
                TypeInfoForValidation::Object {
                    package: blueprint.package_address,
                    blueprint: blueprint.blueprint_name,
                }
            }
            TypeInfoSubstate::KeyValueStore(_) => TypeInfoForValidation::KeyValueStore,
            TypeInfoSubstate::Index => TypeInfoForValidation::Index,
            TypeInfoSubstate::SortedIndex => TypeInfoForValidation::SortedIndex,
        };
        Some(mapped)
    }

    fn is_bucket(&self, type_info: &TypeInfoForValidation) -> bool {
        matches!(
            type_info,
            TypeInfoForValidation::Object { package, blueprint }
                if package == &RESOURCE_PACKAGE
                && matches!(blueprint.as_str(), FUNGIBLE_BUCKET_BLUEPRINT | NON_FUNGIBLE_BUCKET_BLUEPRINT)
        )
    }

    fn is_proof(&self, type_info: &TypeInfoForValidation) -> bool {
        matches!(
            type_info,
            TypeInfoForValidation::Object { package, blueprint }
                if package == &RESOURCE_PACKAGE
                && matches!(blueprint.as_str(), FUNGIBLE_PROOF_BLUEPRINT | NON_FUNGIBLE_PROOF_BLUEPRINT)
        )
    }
}
