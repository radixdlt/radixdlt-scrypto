use crate::kernel::call_frame::RootNodeType;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_callback_api::CallFrameReferences;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::blueprints::resource::AUTH_ZONE_BLUEPRINT;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use radix_engine_interface::{api::ObjectModuleId, blueprints::resource::GlobalCaller};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceContext {
    pub outer_object: GlobalAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct CallerAuthZone {
    pub global_auth_zone: NodeId,
    pub global: Option<GlobalCaller>,
    pub local_package_address: PackageAddress,
}


#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct MethodActor {
    pub direct_access: bool,
    pub node_id: NodeId,
    pub module_id: ObjectModuleId,
    pub ident: String,

    pub caller_auth_zone: Option<CallerAuthZone>,
    pub self_auth_zone: NodeId,

    // Cached info
    pub object_info: ObjectInfo,
}

impl MethodActor {
    pub fn get_blueprint_id(&self) -> BlueprintId {
        match self.module_id {
            ObjectModuleId::Main => self.object_info.blueprint_info.blueprint_id.clone(),
            _ => self.module_id.static_blueprint().unwrap(),
        }
    }

    pub fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier {
            blueprint_id: self.get_blueprint_id(),
            ident: self.ident.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct FunctionActor {
    pub blueprint_id: BlueprintId,
    pub ident: String,

    pub caller_auth_zone: Option<CallerAuthZone>,
    pub self_auth_zone: NodeId,
}

impl FunctionActor {
    pub fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier {
            blueprint_id: self.blueprint_id.clone(),
            ident: self.ident.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct BlueprintHookActor {
    pub receiver: Option<NodeId>,
    pub hook: BlueprintHook,
    pub blueprint_id: BlueprintId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum Actor {
    Root,
    Method(MethodActor),
    Function(FunctionActor),
    BlueprintHook(BlueprintHookActor),
}

impl CallFrameReferences for Actor {

    fn global_references(&self) -> Vec<GlobalAddress> {
        let mut global_refs = Vec::new();

        if let Some(blueprint_id) = self.blueprint_id() {
            global_refs.push(blueprint_id.package_address.into());
        }

        if let Actor::Method(MethodActor { object_info, .. }) = self {
            if let OuterObjectInfo::Some { outer_object } =
                object_info.blueprint_info.outer_obj_info
            {
                global_refs.push(outer_object.clone());
            }
        }

        global_refs
    }

    fn transient_references(&self) -> Vec<NodeId> {
        let mut references = vec![];
        references.extend(self.self_auth_zone());
        if let Some(caller_auth_zone) = self.caller_authzone() {
            references.push(caller_auth_zone.global_auth_zone.clone());
        }

        if !self.is_direct_access() {
            references.extend(self.node_id());
        }

        references
    }

    fn direct_access_references(&self) -> Vec<NodeId> {
        if self.is_direct_access() {
            self.node_id().into_iter().collect()
        } else {
            vec![]
        }
    }

    fn len(&self) -> usize {
        match self {
            Actor::Root => 1,
            Actor::Method(MethodActor { ident, node_id, .. }) => {
                node_id.as_ref().len() + ident.len()
            }
            Actor::Function(FunctionActor {
                blueprint_id,
                ident,
                ..
            }) => {
                blueprint_id.package_address.as_ref().len()
                    + blueprint_id.blueprint_name.len()
                    + ident.len()
            }
            Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => {
                blueprint_id.package_address.as_ref().len() + blueprint_id.blueprint_name.len() + 1
            }
        }
    }
}

impl Actor {
    pub fn self_auth_zone(&self) -> Option<NodeId> {
        match self {
            Actor::Root | Actor::BlueprintHook(..) => None,
            Actor::Method(method_actor) => Some(method_actor.self_auth_zone),
            Actor::Function(function_actor) => Some(function_actor.self_auth_zone),
        }
    }

    pub fn caller_authzone(&self) -> Option<CallerAuthZone> {
        match self {
            Actor::Root | Actor::BlueprintHook(..) => None,
            Actor::Method(method_actor) => method_actor.caller_auth_zone.clone(),
            Actor::Function(function_actor) => function_actor.caller_auth_zone.clone(),
        }
    }

    pub fn instance_context(&self) -> Option<InstanceContext> {
        let method_actor = match self {
            Actor::Method(method_actor) => method_actor,
            _ => return None,
        };

        match method_actor.module_id {
            ObjectModuleId::Main => {
                if method_actor.object_info.global {
                    Some(InstanceContext {
                        outer_object: GlobalAddress::new_or_panic(method_actor.node_id.0),
                    })
                } else {
                    match &method_actor.object_info.blueprint_info.outer_obj_info {
                        OuterObjectInfo::Some { outer_object } => Some(InstanceContext {
                            outer_object: outer_object.clone(),
                        }),
                        OuterObjectInfo::None { .. } => None,
                    }
                }
            }
            _ => None,
        }
    }

    pub fn get_object_id(self) -> Option<(NodeId, ObjectModuleId)> {
        match self {
            Actor::Method(method_actor) => Some((method_actor.node_id, method_actor.module_id)),
            Actor::BlueprintHook(BlueprintHookActor {
                receiver: Some(node_id),
                ..
            }) => Some((node_id, ObjectModuleId::Main)),
            Actor::BlueprintHook(..) | Actor::Root | Actor::Function(..) => None,
        }
    }

    pub fn is_auth_zone(&self) -> bool {
        match self {
            Actor::Method(MethodActor { object_info, .. }) => {
                object_info
                    .blueprint_info
                    .blueprint_id
                    .package_address
                    .eq(&RESOURCE_PACKAGE)
                    && object_info
                        .blueprint_info
                        .blueprint_id
                        .blueprint_name
                        .eq(AUTH_ZONE_BLUEPRINT)
            }
            Actor::Function { .. } => false,
            Actor::BlueprintHook { .. } => false,
            Actor::Root { .. } => false,
        }
    }

    pub fn is_barrier(&self) -> bool {
        match self {
            Actor::Method(MethodActor { object_info, .. }) => object_info.global,
            Actor::Function { .. } => true,
            Actor::BlueprintHook { .. } => true,
            Actor::Root { .. } => false,
        }
    }

    pub fn fn_identifier(&self) -> Option<FnIdentifier> {
        match self {
            Actor::Method(method_actor) => Some(method_actor.fn_identifier()),
            Actor::Function(function_actor) => Some(function_actor.fn_identifier()),
            _ => None,
        }
    }

    pub fn is_transaction_processor_blueprint(&self) -> bool {
        match self {
            Actor::Root => false,
            Actor::Method(MethodActor {
                object_info:
                    ObjectInfo {
                        blueprint_info: BlueprintInfo { blueprint_id, .. },
                        ..
                    },
                ..
            })
            | Actor::Function(FunctionActor { blueprint_id, .. })
            | Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => {
                blueprint_id.eq(&BlueprintId::new(
                    &TRANSACTION_PROCESSOR_PACKAGE,
                    TRANSACTION_PROCESSOR_BLUEPRINT,
                ))
            }
        }
    }

    pub fn node_id(&self) -> Option<NodeId> {
        match self {
            Actor::Method(MethodActor { node_id, .. }) => Some(*node_id),
            Actor::BlueprintHook(BlueprintHookActor {
                receiver: node_id, ..
            }) => node_id.clone(),
            _ => None,
        }
    }

    pub fn is_direct_access(&self) -> bool {
        match self {
            Actor::Method(MethodActor { direct_access, .. }) => *direct_access,
            _ => false,
        }
    }

    pub fn blueprint_id(&self) -> Option<BlueprintId> {
        match self {
            Actor::Method(actor) => Some(actor.get_blueprint_id()),
            Actor::Function(FunctionActor { blueprint_id, .. })
            | Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => {
                Some(blueprint_id.clone())
            }
            Actor::Root => None,
        }
    }

    pub fn package_address(&self) -> Option<PackageAddress> {
        self.blueprint_id().map(|id| id.package_address)
    }
}
