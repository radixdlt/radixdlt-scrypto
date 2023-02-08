use crate::errors::{ModuleError, RuntimeError};
use crate::kernel::kernel_api::{KernelSubstateApi, LockFlags};
use crate::kernel::{CallFrameUpdate, KernelActorApi, KernelNodeApi, ResolvedActor};
use crate::kernel::{KernelModule, KernelModuleId, KernelModuleState};
use crate::types::*;
use radix_engine_interface::api::types::{BucketOffset, ProofOffset, RENodeId, SubstateOffset};
use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum NodeMoveError {
    CantMoveDownstream(RENodeId),
    CantMoveUpstream(RENodeId),
}

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct NodeMoveModule;

impl KernelModuleState for NodeMoveModule {
    const ID: u8 = KernelModuleId::NodeMove as u8;
}

impl NodeMoveModule {
    fn prepare_move_downstream<
        Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>,
    >(
        node_id: RENodeId,
        to: &FnIdentifier,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Bucket(..) => {
                let handle = api.lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let bucket = substate_ref.bucket();
                let locked = bucket.is_locked();
                api.drop_lock(handle)?;
                if locked {
                    Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                        NodeMoveError::CantMoveDownstream(node_id),
                    )))
                } else {
                    Ok(())
                }
            }
            RENodeId::Proof(..) => {
                let from = api.fn_identifier()?;

                if from.is_scrypto_or_transaction() || to.is_scrypto_or_transaction() {
                    let handle = api.lock_substate(
                        node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::Proof(ProofOffset::Proof),
                        LockFlags::MUTABLE,
                    )?;
                    let mut substate_ref_mut = api.get_ref_mut(handle)?;
                    let proof = substate_ref_mut.proof();

                    let rtn = if proof.is_restricted() {
                        Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                            NodeMoveError::CantMoveDownstream(node_id),
                        )))
                    } else {
                        proof.change_to_restricted();
                        Ok(())
                    };

                    api.drop_lock(handle)?;

                    rtn
                } else {
                    Ok(())
                }
            }
            RENodeId::Component(..) => Ok(()),

            RENodeId::TransactionRuntime
            | RENodeId::AuthZoneStack
            | RENodeId::Logger
            | RENodeId::ResourceManager(..)
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::Vault(..)
            | RENodeId::Package(..)
            | RENodeId::Worktop
            | RENodeId::EpochManager(..)
            | RENodeId::Identity(..)
            | RENodeId::Validator(..)
            | RENodeId::Clock(..)
            | RENodeId::Global(..)
            | RENodeId::Account(..)
            | RENodeId::AccessController(..) => Err(RuntimeError::ModuleError(
                ModuleError::NodeMoveError(NodeMoveError::CantMoveDownstream(node_id)),
            )),
        }
    }

    fn prepare_move_upstream<Y: KernelNodeApi + KernelSubstateApi>(
        node_id: RENodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match node_id {
            RENodeId::Bucket(..) => {
                let handle = api.lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let bucket = substate_ref.bucket();
                let locked = bucket.is_locked();
                api.drop_lock(handle)?;
                if locked {
                    Err(RuntimeError::ModuleError(ModuleError::NodeMoveError(
                        NodeMoveError::CantMoveUpstream(node_id),
                    )))
                } else {
                    Ok(())
                }
            }
            RENodeId::Proof(..) | RENodeId::Component(..) | RENodeId::Vault(..) => Ok(()),

            RENodeId::TransactionRuntime
            | RENodeId::AuthZoneStack
            | RENodeId::Logger
            | RENodeId::ResourceManager(..)
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::Package(..)
            | RENodeId::Worktop
            | RENodeId::EpochManager(..)
            | RENodeId::Identity(..)
            | RENodeId::Validator(..)
            | RENodeId::Clock(..)
            | RENodeId::Global(..)
            | RENodeId::Account(..)
            | RENodeId::AccessController(..) => Err(RuntimeError::ModuleError(
                ModuleError::NodeMoveError(NodeMoveError::CantMoveUpstream(node_id)),
            )),
        }
    }
}

impl KernelModule for NodeMoveModule {
    fn on_call_frame_enter<Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>>(
        api: &mut Y,
        call_frame_update: &mut CallFrameUpdate,
        actor: &ResolvedActor,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state::<NodeMoveModule>().is_none() {
            return Ok(());
        }

        for node_id in &call_frame_update.nodes_to_move {
            Self::prepare_move_downstream(*node_id, &actor.identifier, api)?;
        }

        Ok(())
    }

    fn on_call_frame_exit<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        call_frame_update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state::<NodeMoveModule>().is_none() {
            return Ok(());
        }

        for node_id in &call_frame_update.nodes_to_move {
            Self::prepare_move_upstream(*node_id, api)?;
        }

        Ok(())
    }
}
