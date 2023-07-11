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
pub struct RuntimeReceiverInfo {
    pub node_id: NodeId,
    pub module_id: ObjectModuleId,
    pub is_direct_access: bool,
    pub object_info: ObjectInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct MethodActor {
    pub ident: String,
    pub receiver_info: RuntimeReceiverInfo,
    pub global_address: Option<GlobalAddress>,
    pub instance_context: Option<InstanceContext>,
}

impl MethodActor {
    pub fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier {
            blueprint_id: self.receiver_info.object_info.blueprint_id.clone(),
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
    pub receiver_info: Option<RuntimeReceiverInfo>,
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
            Actor::Method(MethodActor {
                ident,
                receiver_info: RuntimeReceiverInfo { node_id, .. },
                ..
            }) => node_id.as_ref().len() + ident.len(),
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
            Actor::Method(MethodActor {
                receiver_info: RuntimeReceiverInfo { object_info, .. },
                ..
            }) => {
                object_info
                    .blueprint_id
                    .package_address
                    .eq(&RESOURCE_PACKAGE)
                    && object_info
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
            Actor::Method(MethodActor {
                receiver_info: RuntimeReceiverInfo { object_info, .. },
                ..
            }) => object_info.global,
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
                receiver_info:
                    RuntimeReceiverInfo {
                        object_info: ObjectInfo { blueprint_id, .. },
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

    pub fn receiver_info(&self) -> Option<RuntimeReceiverInfo> {
        match self {
            Actor::Method(MethodActor { receiver_info, .. }) => Some(receiver_info.clone()),
            _ => None,
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
                receiver_info:
                    RuntimeReceiverInfo {
                        object_info: ObjectInfo { blueprint_id, .. },
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
                receiver_info:
                    RuntimeReceiverInfo {
                        object_info: ObjectInfo { blueprint_id, .. },
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
                receiver_info:
                    RuntimeReceiverInfo {
                        object_info: ObjectInfo { blueprint_id, .. },
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
        method: MethodIdentifier,
        object_info: ObjectInfo,
        instance_context: Option<InstanceContext>,
        is_direct_access: bool,
    ) -> Self {
        Self::Method(MethodActor {
            global_address,
            ident: method.2,
            receiver_info: RuntimeReceiverInfo {
                node_id: method.0,
                module_id: method.1,
                is_direct_access,
                object_info,
            },
            instance_context,
        })
    }

    pub fn function(blueprint_id: BlueprintId, ident: String) -> Self {
        Self::Function(FunctionActor {
            blueprint_id,
            ident,
        })
    }
}
