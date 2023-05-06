use crate::types::*;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use radix_engine_interface::{api::ObjectModuleId, blueprints::resource::GlobalCaller};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InstanceContext {
    pub instance: GlobalAddress,
    pub instance_blueprint: String,
}

/// No method acting here!
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct MethodActor {
    pub global_address: Option<GlobalAddress>,
    pub node_id: NodeId,
    pub module_id: ObjectModuleId,
    pub ident: String,
    pub object_info: ObjectInfo,
    pub instance_context: Option<InstanceContext>,
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub enum Actor {
    Root,
    Method(MethodActor),
    Function { blueprint: Blueprint, ident: String },
    VirtualLazyLoad { blueprint: Blueprint, ident: u8 },
}

impl Actor {
    pub fn is_transaction_processor(&self) -> bool {
        match self {
            Actor::Root => false,
            Actor::Method(MethodActor {
                object_info: ObjectInfo { blueprint, .. },
                ..
            })
            | Actor::Function { blueprint, .. }
            | Actor::VirtualLazyLoad { blueprint, .. } => blueprint.eq(&Blueprint::new(
                &TRANSACTION_PROCESSOR_PACKAGE,
                TRANSACTION_PROCESSOR_BLUEPRINT,
            )),
        }
    }

    pub fn try_as_method(&self) -> Option<&MethodActor> {
        match self {
            Actor::Method(actor) => Some(actor),
            _ => None,
        }
    }

    pub fn get_global_ancestor_as_global_caller(&self) -> Option<GlobalCaller> {
        // TODO - technically this isn't right - we don't properly capture the global ancestor of our call frame.
        // EG if the global ancestor of a method was a blueprint function call, we report it as a GlobalObject(package_address)
        match self {
            Actor::Method(actor) => actor.global_address.map(|address| address.into()),
            Actor::Function { blueprint, .. } => Some(blueprint.clone().into()),
            _ => None,
        }
    }

    pub fn instance_context(&self) -> Option<InstanceContext> {
        match self {
            Actor::Method(MethodActor {
                instance_context, ..
            }) => instance_context.clone(),
            _ => None,
        }
    }

    pub fn blueprint(&self) -> &Blueprint {
        match self {
            Actor::Method(MethodActor {
                object_info: ObjectInfo { blueprint, .. },
                ..
            })
            | Actor::Function { blueprint, .. }
            | Actor::VirtualLazyLoad { blueprint, .. } => blueprint,
            Actor::Root => panic!("Unexpected call"), // TODO: Should we just mock this?
        }
    }

    pub fn get_virtual_non_extending_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        btreeset!(NonFungibleGlobalId::package_of_caller_badge(
            *self.package_address()
        ))
    }

    pub fn get_virtual_non_extending_barrier_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        if let Some(global_caller) = self.get_global_ancestor_as_global_caller() {
            btreeset!(NonFungibleGlobalId::global_caller_badge(global_caller))
        } else {
            btreeset!()
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        let blueprint = match &self {
            Actor::Method(MethodActor {
                object_info: ObjectInfo { blueprint, .. },
                ..
            }) => blueprint,
            Actor::Function { blueprint, .. } => blueprint,
            Actor::VirtualLazyLoad { blueprint, .. } => blueprint,
            Actor::Root => return &PACKAGE_PACKAGE, // TODO: Should we mock this with something better?
        };

        &blueprint.package_address
    }

    pub fn blueprint_name(&self) -> &str {
        match &self {
            Actor::Method(MethodActor {
                object_info: ObjectInfo { blueprint, .. },
                ..
            })
            | Actor::Function { blueprint, .. }
            | Actor::VirtualLazyLoad { blueprint, .. } => blueprint.blueprint_name.as_str(),
            Actor::Root => panic!("Unexpected call"), // TODO: Should we just mock this?
        }
    }

    pub fn method(
        global_address: Option<GlobalAddress>,
        method: MethodIdentifier,
        object_info: ObjectInfo,
        instance_context: Option<InstanceContext>,
    ) -> Self {
        Self::Method(MethodActor {
            global_address,
            node_id: method.0,
            module_id: method.1,
            ident: method.2,
            object_info,
            instance_context,
        })
    }

    pub fn function(ident: FunctionIdentifier) -> Self {
        Self::Function {
            blueprint: ident.0,
            ident: ident.1,
        }
    }

    pub fn virtual_lazy_load(blueprint: Blueprint, ident: u8) -> Self {
        Self::VirtualLazyLoad { blueprint, ident }
    }
}
