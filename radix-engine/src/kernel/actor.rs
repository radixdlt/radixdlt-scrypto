use crate::types::*;
use radix_engine_interface::blueprints::resource::AUTH_ZONE_BLUEPRINT;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use radix_engine_interface::{api::ObjectModuleId, blueprints::resource::GlobalCaller};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceContext {
    pub outer_object: GlobalAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ReceiverType {
    DirectAccess,
    OnHeapObject,
    OnStoredObject(GlobalAddress),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct MethodActor {
    pub receiver_type: ReceiverType,
    pub node_id: NodeId,
    pub module_id: ObjectModuleId,
    pub ident: String,

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

impl Actor {
    pub fn global_references(&self) -> Vec<GlobalAddress> {
        let mut global_refs = Vec::new();

        if let Some(blueprint_id) = self.blueprint_id() {
            global_refs.push(blueprint_id.package_address.into());
        }

        if let Actor::Method(MethodActor {
            receiver_type,
            object_info,
            ..
        }) = self
        {
            if let ReceiverType::OnStoredObject(global_address) = receiver_type {
                global_refs.push(global_address.clone());
            }
            if let OuterObjectInfo::Some { outer_object } =
                object_info.blueprint_info.outer_obj_info
            {
                global_refs.push(outer_object.clone());
            }
        }

        global_refs
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

    pub fn len(&self) -> usize {
        match self {
            Actor::Root => 1,
            Actor::Method(MethodActor { ident, node_id, .. }) => {
                node_id.as_ref().len() + ident.len()
            }
            Actor::Function(FunctionActor {
                blueprint_id,
                ident,
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
            Actor::Method(MethodActor {
                receiver_type: ReceiverType::DirectAccess,
                ..
            }) => true,
            _ => false,
        }
    }

    pub fn receiver_type(&self) -> Option<ReceiverType> {
        match self {
            Actor::Method(MethodActor { receiver_type, .. }) => Some(receiver_type.clone()),
            _ => None,
        }
    }

    pub fn as_global_caller(&self) -> Option<GlobalCaller> {
        match self {
            Actor::Method(actor) => match &actor.receiver_type {
                ReceiverType::OnStoredObject(global_address) => Some(global_address.clone().into()),
                _ => None,
            },
            Actor::Function(FunctionActor { blueprint_id, .. }) => {
                Some(blueprint_id.clone().into())
            }
            _ => None,
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

    /// Proofs which exist only on the local call frame
    pub fn get_local_call_frame_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        if let Some(blueprint_id) = self.blueprint_id() {
            btreeset!(NonFungibleGlobalId::package_of_direct_caller_badge(
                blueprint_id.package_address
            ))
        } else {
            btreeset!()
        }
    }

    pub fn get_global_call_frame_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        if let Some(global_caller) = self.as_global_caller() {
            btreeset!(NonFungibleGlobalId::global_caller_badge(global_caller))
        } else {
            btreeset!()
        }
    }

    pub fn method(
        receiver_type: ReceiverType,
        node_id: NodeId,
        module_id: ObjectModuleId,
        ident: String,
        object_info: ObjectInfo,
    ) -> Self {
        Self::Method(MethodActor {
            receiver_type,
            node_id,
            module_id,
            ident,
            object_info,
        })
    }

    pub fn function(blueprint_id: BlueprintId, ident: String) -> Self {
        Self::Function(FunctionActor {
            blueprint_id,
            ident,
        })
    }
}
