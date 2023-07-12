use crate::types::*;
use radix_engine_interface::blueprints::resource::AUTH_ZONE_BLUEPRINT;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use radix_engine_interface::{api::ObjectModuleId, blueprints::resource::GlobalCaller};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InstanceContext {
    pub outer_object: GlobalAddress,
    pub outer_blueprint: String,
    // TODO: Add module id?
    // TODO: Add features
}

/// No method acting here!
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct MethodActor {
    /// Global Address exists if: (1) NOT a direct access and (2) the object has been stored
    pub global_address: Option<GlobalAddress>,
    pub node_id: NodeId,
    pub module_id: ObjectModuleId,
    pub ident: String,
    pub is_direct_access: bool,

    // Cached info
    pub node_object_info: NodeObjectInfo,
}

impl MethodActor {
    pub fn get_blueprint_id(&self) -> BlueprintId {
        match self.module_id {
            ObjectModuleId::Main => self
                .node_object_info
                .main_blueprint_info
                .blueprint_id
                .clone(),
            _ => self.module_id.static_blueprint().unwrap(),
        }
    }

    pub fn get_blueprint_info(&self) -> BlueprintObjectType {
        match self.module_id {
            ObjectModuleId::Main => self
                .node_object_info
                .main_blueprint_info
                .blueprint_type
                .clone(),
            _ => BlueprintObjectType::Outer,
        }
    }

    pub fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier {
            blueprint_id: self.get_blueprint_id(),
            ident: FnIdent::Application(self.ident.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub enum Actor {
    Root,
    Method(MethodActor),
    Function {
        blueprint_id: BlueprintId,
        ident: String,
    },
    VirtualLazyLoad {
        blueprint_id: BlueprintId,
        ident: u8,
    },
}

impl Actor {
    pub fn len(&self) -> usize {
        match self {
            Actor::Root => 1,
            Actor::Method(MethodActor { node_id, ident, .. }) => {
                node_id.as_ref().len() + ident.len()
            }
            Actor::Function {
                blueprint_id: blueprint,
                ident,
            } => {
                blueprint.package_address.as_ref().len()
                    + blueprint.blueprint_name.len()
                    + ident.len()
            }
            Actor::VirtualLazyLoad {
                blueprint_id: blueprint,
                ..
            } => blueprint.package_address.as_ref().len() + blueprint.blueprint_name.len() + 1,
        }
    }

    pub fn is_auth_zone(&self) -> bool {
        match self {
            Actor::Method(MethodActor {
                node_object_info: object_info,
                ..
            }) => {
                object_info
                    .main_blueprint_info
                    .blueprint_id
                    .package_address
                    .eq(&RESOURCE_PACKAGE)
                    && object_info
                        .main_blueprint_info
                        .blueprint_id
                        .blueprint_name
                        .eq(AUTH_ZONE_BLUEPRINT)
            }
            Actor::Function { .. } => false,
            Actor::VirtualLazyLoad { .. } => false,
            Actor::Root { .. } => false,
        }
    }

    pub fn is_barrier(&self) -> bool {
        match self {
            Actor::Method(MethodActor {
                node_object_info: object_info,
                ..
            }) => object_info.global,
            Actor::Function { .. } => true,
            Actor::VirtualLazyLoad { .. } => false,
            Actor::Root { .. } => false,
        }
    }

    pub fn fn_identifier(&self) -> FnIdentifier {
        match self {
            Actor::Root => panic!("Should never be called"),
            Actor::Method(method_actor) => method_actor.fn_identifier(),
            Actor::Function {
                blueprint_id: blueprint,
                ident,
            } => FnIdentifier {
                blueprint_id: blueprint.clone(),
                ident: FnIdent::Application(ident.to_string()),
            },
            Actor::VirtualLazyLoad {
                blueprint_id: blueprint,
                ident,
            } => FnIdentifier {
                blueprint_id: blueprint.clone(),
                ident: FnIdent::System(*ident),
            },
        }
    }

    pub fn is_transaction_processor_blueprint(&self) -> bool {
        match self {
            Actor::Root => false,
            Actor::Method(MethodActor {
                node_object_info:
                    NodeObjectInfo {
                        main_blueprint_info:
                            BlueprintObjectInfo {
                                blueprint_id: blueprint,
                                ..
                            },
                        ..
                    },
                ..
            })
            | Actor::Function {
                blueprint_id: blueprint,
                ..
            }
            | Actor::VirtualLazyLoad {
                blueprint_id: blueprint,
                ..
            } => blueprint.eq(&BlueprintId::new(
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

    pub fn global_address(&self) -> Option<GlobalAddress> {
        match self {
            Actor::Method(MethodActor { global_address, .. }) => global_address.clone(),
            _ => None,
        }
    }

    pub fn as_global_caller(&self) -> Option<GlobalCaller> {
        match self {
            Actor::Method(actor) => actor.global_address.map(|address| address.into()),
            Actor::Function {
                blueprint_id: blueprint,
                ..
            } => Some(blueprint.clone().into()),
            _ => None,
        }
    }

    pub fn blueprint_id(&self) -> BlueprintId {
        match self {
            Actor::Method(actor) => actor.get_blueprint_id(),
            Actor::Function { blueprint_id, .. } | Actor::VirtualLazyLoad { blueprint_id, .. } => {
                blueprint_id.clone()
            }
            Actor::Root => panic!("Unexpected call"), // FIXME: have the right interface
        }
    }

    /// Proofs which exist only on the local call frame
    /// FIXME: Update abstractions such that it is based on local call frame
    pub fn get_virtual_non_extending_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        btreeset!(NonFungibleGlobalId::package_of_direct_caller_badge(
            self.blueprint_id().package_address
        ))
    }

    pub fn get_virtual_non_extending_barrier_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        if let Some(global_caller) = self.as_global_caller() {
            btreeset!(NonFungibleGlobalId::global_caller_badge(global_caller))
        } else {
            btreeset!()
        }
    }

    pub fn method(
        global_address: Option<GlobalAddress>,
        node_id: NodeId,
        module_id: ObjectModuleId,
        ident: String,
        object_info: NodeObjectInfo,
        is_direct_access: bool,
    ) -> Self {
        Self::Method(MethodActor {
            global_address,
            node_id,
            module_id,
            ident,
            node_object_info: object_info,
            is_direct_access,
        })
    }

    pub fn function(blueprint: BlueprintId, ident: String) -> Self {
        Self::Function {
            blueprint_id: blueprint,
            ident,
        }
    }

    pub fn virtual_lazy_load(blueprint: BlueprintId, ident: u8) -> Self {
        Self::VirtualLazyLoad {
            blueprint_id: blueprint,
            ident,
        }
    }
}
