use crate::blueprints::resource::ProofMoveableSubstate;
use crate::errors::{RuntimeError, SystemModuleError};
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NodeMoveError {
    CantMoveDownstream(NodeId),
    CantMoveUpstream(NodeId),
}

#[derive(Debug, Clone)]
pub struct NodeMoveModule {}

impl NodeMoveModule {
    fn prepare_move_downstream<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        node_id: NodeId,
        callee: &Actor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // TODO: Make this more generic?
        let type_info = TypeInfoBlueprint::get_type(&node_id, api)?;
        match type_info {
            TypeInfoSubstate::Object(info)
                if info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE)
                    && info
                        .blueprint_id
                        .blueprint_name
                        .eq(FUNGIBLE_PROOF_BLUEPRINT) =>
            {
                if matches!(callee, Actor::Method(MethodActor { node_id, .. }) if node_id.eq(info.get_outer_object().as_node_id()))
                {
                    return Ok(());
                }

                if matches!(callee, Actor::Function { .. })
                    && callee.blueprint_id().eq(&info.blueprint_id)
                {
                    return Ok(());
                }

                // Change to restricted unless it's moved to auth zone.
                if callee.is_barrier() {
                    let handle = api.kernel_open_substate(
                        &node_id,
                        MAIN_BASE_PARTITION,
                        &FungibleProofField::Moveable.into(),
                        LockFlags::MUTABLE,
                        SystemLockData::default(),
                    )?;
                    let mut proof: ProofMoveableSubstate =
                        api.kernel_read_substate(handle)?.as_typed().unwrap();

                    if proof.restricted {
                        return Err(RuntimeError::SystemModuleError(
                            SystemModuleError::NodeMoveError(NodeMoveError::CantMoveDownstream(
                                node_id,
                            )),
                        ));
                    }

                    proof.change_to_restricted();
                    api.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&proof))?;
                    api.kernel_close_substate(handle)?;
                } else if callee.is_auth_zone() {
                    let handle = api.kernel_open_substate(
                        &node_id,
                        MAIN_BASE_PARTITION,
                        &FungibleProofField::Moveable.into(),
                        LockFlags::read_only(),
                        SystemLockData::default(),
                    )?;
                    let proof: ProofMoveableSubstate =
                        api.kernel_read_substate(handle)?.as_typed().unwrap();

                    if proof.restricted {
                        return Err(RuntimeError::SystemModuleError(
                            SystemModuleError::NodeMoveError(NodeMoveError::CantMoveDownstream(
                                node_id,
                            )),
                        ));
                    }
                    api.kernel_close_substate(handle)?;
                }
            }
            TypeInfoSubstate::Object(info)
                if info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE)
                    && info
                        .blueprint_id
                        .blueprint_name
                        .eq(NON_FUNGIBLE_PROOF_BLUEPRINT) =>
            {
                if matches!(callee, Actor::Method(MethodActor { node_id, .. }) if node_id.eq(info.get_outer_object().as_node_id()))
                {
                    return Ok(());
                }

                if matches!(callee, Actor::Function { .. })
                    && callee.blueprint_id().eq(&info.blueprint_id)
                {
                    return Ok(());
                }

                // Change to restricted unless it's moved to auth zone.
                if callee.is_barrier() {
                    let handle = api.kernel_open_substate(
                        &node_id,
                        MAIN_BASE_PARTITION,
                        &NonFungibleProofField::Moveable.into(),
                        LockFlags::MUTABLE,
                        SystemLockData::default(),
                    )?;
                    let mut proof: ProofMoveableSubstate =
                        api.kernel_read_substate(handle)?.as_typed().unwrap();

                    if proof.restricted {
                        return Err(RuntimeError::SystemModuleError(
                            SystemModuleError::NodeMoveError(NodeMoveError::CantMoveDownstream(
                                node_id,
                            )),
                        ));
                    }

                    proof.change_to_restricted();
                    api.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&proof))?;
                    api.kernel_close_substate(handle)?;
                } else if callee.is_auth_zone() {
                    let handle = api.kernel_open_substate(
                        &node_id,
                        MAIN_BASE_PARTITION,
                        &NonFungibleProofField::Moveable.into(),
                        LockFlags::read_only(),
                        SystemLockData::default(),
                    )?;
                    let proof: ProofMoveableSubstate =
                        api.kernel_read_substate(handle)?.as_typed().unwrap();

                    if proof.restricted {
                        return Err(RuntimeError::SystemModuleError(
                            SystemModuleError::NodeMoveError(NodeMoveError::CantMoveDownstream(
                                node_id,
                            )),
                        ));
                    }
                    api.kernel_close_substate(handle)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn prepare_move_upstream<Y: KernelApi<M>, M: KernelCallbackObject>(
        _node_id: NodeId,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for NodeMoveModule {
    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        message: &mut Message,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        for node_id in &message.move_nodes {
            Self::prepare_move_downstream(*node_id, callee, api)?;
        }

        Ok(())
    }

    fn on_execution_finish<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        message: &Message,
    ) -> Result<(), RuntimeError> {
        for node_id in &message.move_nodes {
            Self::prepare_move_upstream(*node_id, api)?;
        }

        Ok(())
    }
}
