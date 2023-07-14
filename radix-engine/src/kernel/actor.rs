use crate::types::*;
use radix_engine_interface::blueprints::package::PackageExport;
use radix_engine_interface::blueprints::resource::AUTH_ZONE_BLUEPRINT;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use radix_engine_interface::{api::ObjectModuleId, blueprints::resource::GlobalCaller};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InstanceContext {
    pub outer_object: GlobalAddress,
    pub outer_blueprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct MethodActor {
    pub node_id: NodeId,
    pub module_id: ObjectModuleId,
    pub is_direct_access: bool,
    pub ident: String,

    pub object_info: ObjectInfo,
    pub global_address: Option<GlobalAddress>,
    pub instance_context: Option<InstanceContext>,
}

impl MethodActor {
    pub fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier {
            blueprint_id: self.object_info.main_blueprint_id.clone(),
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
    pub blueprint_id: BlueprintId,
    pub hook: BlueprintHook,
    pub export: PackageExport,

    pub node_id: Option<NodeId>,
    pub module_id: Option<ObjectModuleId>,
    pub object_info: Option<ObjectInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum Actor {
    Root,
    Method(MethodActor),
    Function(FunctionActor),
    BlueprintHook(BlueprintHookActor),
}

impl Actor {
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
                    .main_blueprint_id
                    .package_address
                    .eq(&RESOURCE_PACKAGE)
                    && object_info
                        .main_blueprint_id
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
                        main_blueprint_id: blueprint_id,
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
            Actor::BlueprintHook(BlueprintHookActor { node_id, .. }) => node_id.clone(),
            _ => None,
        }
    }

    pub fn module_id(&self) -> Option<ObjectModuleId> {
        match self {
            Actor::Method(MethodActor { module_id, .. }) => Some(*module_id),
            Actor::BlueprintHook(BlueprintHookActor { module_id, .. }) => module_id.clone(),
            _ => None,
        }
    }

    pub fn object_info(&self) -> Option<ObjectInfo> {
        match self {
            Actor::Method(MethodActor { object_info, .. }) => Some(object_info.clone()),
            Actor::BlueprintHook(BlueprintHookActor { object_info, .. }) => object_info.clone(),
            _ => None,
        }
    }

    pub fn is_direct_access(&self) -> bool {
        match self {
            Actor::Method(MethodActor {
                is_direct_access, ..
            }) => *is_direct_access,
            _ => false,
        }
    }

    pub fn global_address(&self) -> Option<GlobalAddress> {
        match self {
            Actor::Method(MethodActor { global_address, .. }) => global_address.clone(),
            _ => None,
        }
    }

    pub fn as_global_caller(&self) -> Option<GlobalCaller> {
        match self {
            Actor::Method(actor) => actor.global_address.map(|address| address.into()),
            Actor::Function(FunctionActor { blueprint_id, .. }) => {
                Some(blueprint_id.clone().into())
            }
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

    pub fn blueprint_id(&self) -> Option<&BlueprintId> {
        match self {
            Actor::Method(MethodActor {
                object_info:
                    ObjectInfo {
                        main_blueprint_id: blueprint_id,
                        ..
                    },
                ..
            })
            | Actor::Function(FunctionActor { blueprint_id, .. })
            | Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => Some(blueprint_id),
            Actor::Root => None,
        }
    }

    /// Proofs which exist only on the local call frame
    /// FIXME: Update abstractions such that it is based on local call frame
    pub fn get_virtual_non_extending_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        if let Some(package_address) = self.package_address() {
            btreeset!(NonFungibleGlobalId::package_of_direct_caller_badge(
                *package_address
            ))
        } else {
            btreeset!()
        }
    }

    pub fn get_virtual_non_extending_barrier_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        if let Some(global_caller) = self.as_global_caller() {
            btreeset!(NonFungibleGlobalId::global_caller_badge(global_caller))
        } else {
            btreeset!()
        }
    }

    pub fn package_address(&self) -> Option<&PackageAddress> {
        match &self {
            Actor::Method(MethodActor {
                object_info:
                    ObjectInfo {
                        main_blueprint_id: blueprint_id,
                        ..
                    },
                ..
            })
            | Actor::Function(FunctionActor { blueprint_id, .. })
            | Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => {
                Some(&blueprint_id.package_address)
            }
            Actor::Root => None,
        }
    }

    pub fn blueprint_name(&self) -> Option<&str> {
        match &self {
            Actor::Method(MethodActor {
                object_info:
                    ObjectInfo {
                        main_blueprint_id: blueprint_id,
                        ..
                    },
                ..
            })
            | Actor::Function(FunctionActor { blueprint_id, .. })
            | Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => {
                Some(blueprint_id.blueprint_name.as_str())
            }
            Actor::Root => None,
        }
    }

    pub fn method(
        global_address: Option<GlobalAddress>,
        node_id: NodeId,
        module_id: ObjectModuleId,
        ident: String,
        object_info: ObjectInfo,
        instance_context: Option<InstanceContext>,
        is_direct_access: bool,
    ) -> Self {
        Self::Method(MethodActor {
            global_address,
            node_id,
            module_id,
            ident,
            object_info,
            instance_context,
            is_direct_access,
        })
    }

    pub fn function(blueprint_id: BlueprintId, ident: String) -> Self {
        Self::Function(FunctionActor {
            blueprint_id,
            ident,
        })
    }
}
