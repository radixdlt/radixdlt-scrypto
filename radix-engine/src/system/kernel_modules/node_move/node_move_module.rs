use crate::blueprints::resource::ProofInfoSubstate;
use crate::errors::{ModuleError, RuntimeError};
use crate::kernel::actor::{Actor, ActorIdentifier};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::types::*;
use radix_engine_interface::api::{ClientApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NodeMoveError {
    CantMoveDownstream(RENodeId),
    CantMoveUpstream(RENodeId),
}

#[derive(Debug, Clone)]
pub struct NodeMoveModule {}

impl NodeMoveModule {
    fn prepare_move_downstream<Y: KernelModuleApi<RuntimeError> + ClientApi<RuntimeError>>(
        node_id: RENodeId,
        actor: &Option<Actor>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Object(..) => {
                let (package_address, blueprint) = api.get_object_type_info(node_id)?;
                match (package_address, blueprint.as_str()) {
                    (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => {
                        if let Some(Actor {
                            identifier:
                                ActorIdentifier::Function(FnIdentifier {
                                    package_address: RESOURCE_MANAGER_PACKAGE,
                                    ..
                                }),
                            ..
                        }) = actor
                        {
                            return Ok(());
                        }

                        let handle = api.kernel_lock_substate(
                            node_id,
                            NodeModuleId::SELF,
                            SubstateOffset::Proof(ProofOffset::Info),
                            LockFlags::MUTABLE,
                        )?;
                        let proof: &mut ProofInfoSubstate =
                            api.kernel_get_substate_ref_mut(handle)?;

                        if proof.restricted {
                            return Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                                NodeMoveError::CantMoveDownstream(node_id),
                            )));
                        }

                        // Change to restricted unless it's for auth zone.
                        if !matches!(
                            actor,
                            Some(Actor {
                                identifier: ActorIdentifier::Method(MethodIdentifier(
                                    RENodeId::AuthZoneStack,
                                    ..
                                )),
                                ..
                            })
                        ) {
                            proof.change_to_restricted();
                        }

                        api.kernel_drop_lock(handle)?;
                    }
                    _ => {}
                }
                Ok(())
            }

            RENodeId::AuthZoneStack
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::GlobalObject(..) => Err(RuntimeError::ModuleError(
                ModuleError::NodeMoveError(NodeMoveError::CantMoveDownstream(node_id)),
            )),
        }
    }

    fn prepare_move_upstream<Y: KernelModuleApi<RuntimeError>>(
        node_id: RENodeId,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Object(..) => Ok(()),

            RENodeId::AuthZoneStack
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::GlobalObject(..) => Err(RuntimeError::ModuleError(
                ModuleError::NodeMoveError(NodeMoveError::CantMoveUpstream(node_id)),
            )),
        }
    }
}

impl KernelModule for NodeMoveModule {
    fn before_push_frame<Y: KernelModuleApi<RuntimeError> + ClientApi<RuntimeError>>(
        api: &mut Y,
        actor: &Option<Actor>,
        call_frame_update: &mut CallFrameUpdate,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        for node_id in &call_frame_update.nodes_to_move {
            // TODO: Move into system layer
            Self::prepare_move_downstream(*node_id, actor, api)?;
        }

        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _caller: &Option<Actor>,
        call_frame_update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        for node_id in &call_frame_update.nodes_to_move {
            Self::prepare_move_upstream(*node_id, api)?;
        }

        Ok(())
    }
}
