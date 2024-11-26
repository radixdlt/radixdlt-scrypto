use crate::internal_prelude::*;
use crate::kernel::kernel_callback_api::CallFrameReferences;
use radix_engine_interface::api::{AttachedModuleId, ModuleId};

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct InstanceContext {
    pub outer_object: GlobalAddress,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum MethodType {
    Main,
    Direct,
    Module(AttachedModuleId),
}

impl MethodType {
    pub fn module_id(&self) -> ModuleId {
        match self {
            MethodType::Module(module_id) => module_id.clone().into(),
            MethodType::Main | MethodType::Direct => ModuleId::Main,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct MethodActor {
    pub method_type: MethodType,
    pub node_id: NodeId,
    pub ident: String,

    pub auth_zone: NodeId,

    // Cached info
    pub object_info: ObjectInfo,
}

impl MethodActor {
    pub fn get_blueprint_id(&self) -> BlueprintId {
        match self.method_type {
            MethodType::Main | MethodType::Direct => {
                self.object_info.blueprint_info.blueprint_id.clone()
            }
            MethodType::Module(module_id) => module_id.static_blueprint(),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct FunctionActor {
    pub blueprint_id: BlueprintId,
    pub ident: String,

    pub auth_zone: NodeId,
}

impl FunctionActor {
    pub fn as_global_caller(&self) -> GlobalCaller {
        GlobalCaller::PackageBlueprint(self.blueprint_id.clone())
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct BlueprintHookActor {
    pub receiver: Option<NodeId>,
    pub hook: BlueprintHook,
    pub blueprint_id: BlueprintId,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum Actor {
    /// In System V1, there was an explicit call to initialize the transaction processor.
    /// This call has to have an actor making the call, which is the Root.
    ///
    /// From V2 onwards, we don't have an explicit function call to initialize the transaction
    /// processor - but we still temporarily set a CallFrameInit with a `Root` actor.
    /// This is used to set up the initial AuthZone in [`MultiThreadIntentProcessor::init`].
    ///
    /// [`MultiThreadIntentProcessor::init`]: crate::system::transaction::multithread_intent_processor::MultiThreadIntentProcessor::init
    Root,
    Method(MethodActor),
    Function(FunctionActor),
    BlueprintHook(BlueprintHookActor),
}

// This is only used by `kernel_create_kernel_for_testing` in the testing framework.
impl Default for Actor {
    fn default() -> Self {
        Self::Root
    }
}

impl CallFrameReferences for Actor {
    fn global_references(&self) -> Vec<NodeId> {
        let mut global_refs = Vec::new();

        if let Some(blueprint_id) = self.blueprint_id() {
            global_refs.push(blueprint_id.package_address.into_node_id());
        }

        if let Actor::Method(MethodActor {
            node_id,
            object_info,
            ..
        }) = self
        {
            if let OuterObjectInfo::Some { outer_object } =
                object_info.blueprint_info.outer_obj_info
            {
                global_refs.push(outer_object.clone().into_node_id());
            }

            if node_id.is_global() {
                global_refs.push(node_id.clone());
            }
        }

        global_refs
    }

    fn direct_access_references(&self) -> Vec<NodeId> {
        if self.is_direct_access() {
            self.node_id().into_iter().collect()
        } else {
            vec![]
        }
    }

    fn stable_transient_references(&self) -> Vec<NodeId> {
        let mut references = vec![];
        references.extend(self.self_auth_zone());

        if !self.is_direct_access() {
            references.extend(self.node_id().filter(|n| !n.is_global()));
        }

        references
    }

    fn len(&self) -> usize {
        match self {
            Actor::Root => 1,
            Actor::Method(MethodActor { ident, node_id, .. }) => {
                node_id.as_bytes().len() + ident.len()
            }
            Actor::Function(FunctionActor {
                blueprint_id,
                ident,
                ..
            }) => {
                blueprint_id.package_address.as_bytes().len()
                    + blueprint_id.blueprint_name.len()
                    + ident.len()
            }
            Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => {
                blueprint_id.package_address.as_bytes().len()
                    + blueprint_id.blueprint_name.len()
                    + 1
            }
        }
    }
}

impl Actor {
    pub fn is_root(&self) -> bool {
        matches!(self, Actor::Root)
    }

    pub fn self_auth_zone(&self) -> Option<NodeId> {
        match self {
            Actor::Root | Actor::BlueprintHook(..) => None,
            Actor::Method(method_actor) => Some(method_actor.auth_zone),
            Actor::Function(function_actor) => Some(function_actor.auth_zone),
        }
    }

    pub fn instance_context(&self) -> Option<InstanceContext> {
        let method_actor = match self {
            Actor::Method(method_actor) => method_actor,
            _ => return None,
        };

        match method_actor.method_type {
            MethodType::Main | MethodType::Direct => {
                if method_actor.object_info.is_global() {
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

    pub fn get_object_id(&self) -> Option<(NodeId, Option<AttachedModuleId>)> {
        match self {
            Actor::Method(method_actor) => Some((
                method_actor.node_id,
                method_actor.method_type.module_id().into(),
            )),
            Actor::BlueprintHook(BlueprintHookActor {
                receiver: Some(node_id),
                ..
            }) => Some((*node_id, None)),
            Actor::BlueprintHook(..) | Actor::Root | Actor::Function(..) => None,
        }
    }

    pub fn is_barrier(&self) -> bool {
        match self {
            Actor::Method(MethodActor { object_info, .. }) => object_info.is_global(),
            Actor::Function { .. } => true,
            Actor::BlueprintHook { .. } => true,
            Actor::Root { .. } => false,
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
            Actor::Method(MethodActor { method_type, .. }) => {
                matches!(method_type, MethodType::Direct)
            }
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
