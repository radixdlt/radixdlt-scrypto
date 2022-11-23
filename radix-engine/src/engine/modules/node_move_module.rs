use crate::engine::{CallFrameUpdate, LockFlags, ModuleError, REActor, RuntimeError, SystemApi};
use crate::types::*;
use radix_engine_interface::api::types::{BucketOffset, ProofOffset, RENodeId, SubstateOffset};

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NodeMoveError {
    CantMoveDownstream(RENodeId),
    CantMoveUpstream(RENodeId),
}

pub struct NodeMoveModule;

impl NodeMoveModule {
    fn prepare_move_downstream<Y: SystemApi>(
        node_id: RENodeId,
        to: &REActor,
        system_api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Bucket(..) => {
                let handle = system_api.lock_substate(
                    node_id,
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                    LockFlags::read_only(),
                )?;
                let substate_ref = system_api.get_ref(handle)?;
                let bucket = substate_ref.bucket();
                let locked = bucket.is_locked();
                system_api.drop_lock(handle)?;
                if locked {
                    Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                        NodeMoveError::CantMoveDownstream(node_id),
                    )))
                } else {
                    Ok(())
                }
            }
            RENodeId::Proof(..) => {
                let from = system_api.get_actor();

                if from.is_scrypto_or_transaction() || to.is_scrypto_or_transaction() {
                    let handle = system_api.lock_substate(
                        node_id,
                        SubstateOffset::Proof(ProofOffset::Proof),
                        LockFlags::MUTABLE,
                    )?;
                    let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                    let proof = substate_ref_mut.proof();

                    let rtn = if proof.is_restricted() {
                        Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                            NodeMoveError::CantMoveDownstream(node_id),
                        )))
                    } else {
                        proof.change_to_restricted();
                        Ok(())
                    };

                    system_api.drop_lock(handle)?;

                    rtn
                } else {
                    Ok(())
                }
            }
            RENodeId::Component(..) => Ok(()),
            RENodeId::AuthZoneStack(..)
            | RENodeId::RoyaltyReserve(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::Vault(..)
            | RENodeId::Package(..)
            | RENodeId::Worktop
            | RENodeId::EpochManager(..)
            | RENodeId::Global(..) => Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                NodeMoveError::CantMoveDownstream(node_id),
            ))),
        }
    }

    fn prepare_move_upstream<Y: SystemApi>(
        node_id: RENodeId,
        system_api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Bucket(..) => {
                let handle = system_api.lock_substate(
                    node_id,
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                    LockFlags::read_only(),
                )?;
                let substate_ref = system_api.get_ref(handle)?;
                let bucket = substate_ref.bucket();
                let locked = bucket.is_locked();
                system_api.drop_lock(handle)?;
                if locked {
                    Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                        NodeMoveError::CantMoveUpstream(node_id),
                    )))
                } else {
                    Ok(())
                }
            }
            RENodeId::Proof(..) | RENodeId::Component(..) | RENodeId::Vault(..) => Ok(()),

            RENodeId::AuthZoneStack(..)
            | RENodeId::RoyaltyReserve(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::Package(..)
            | RENodeId::Worktop
            | RENodeId::EpochManager(..)
            | RENodeId::Global(..) => Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                NodeMoveError::CantMoveUpstream(node_id),
            ))),
        }
    }

    pub fn on_call_frame_enter<Y: SystemApi>(
        call_frame_update: &mut CallFrameUpdate,
        actor: &REActor,
        system_api: &mut Y,
    ) -> Result<(), RuntimeError> {
        for node_id in &call_frame_update.nodes_to_move {
            Self::prepare_move_downstream(*node_id, actor, system_api)?;
        }

        Ok(())
    }

    pub fn on_call_frame_exit<Y: SystemApi>(
        call_frame_update: &CallFrameUpdate,
        system_api: &mut Y,
    ) -> Result<(), RuntimeError> {
        for node_id in &call_frame_update.nodes_to_move {
            Self::prepare_move_upstream(*node_id, system_api)?;
        }

        Ok(())
    }
}
