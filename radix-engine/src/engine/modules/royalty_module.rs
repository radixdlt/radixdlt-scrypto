use crate::engine::*;
use crate::fee::{FeeReserve, RoyaltyReceiver};
use crate::model::GlobalAddressSubstate;
use radix_engine_interface::api::types::{
    ComponentOffset, FnIdentifier, GlobalAddress, GlobalOffset, PackageOffset, RENodeId,
    SubstateId, SubstateOffset,
};
use radix_engine_interface::scrypto;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RoyaltyError {
    TrackError(TrackError),
}

pub struct RoyaltyModule {}

impl From<RoyaltyError> for ModuleError {
    fn from(error: RoyaltyError) -> Self {
        Self::RoyaltyError(error)
    }
}

impl From<TrackError> for RoyaltyError {
    fn from(error: TrackError) -> Self {
        Self::TrackError(error)
    }
}

impl Default for RoyaltyModule {
    fn default() -> Self {
        Self {}
    }
}

impl<R: FeeReserve> Module<R> for RoyaltyModule {
    fn pre_execute_invocation(
        &mut self,
        actor: &ResolvedActor,
        _update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        // Identify the function, and optional component address
        let (scrypto_fn_identifier, optional_component_address) = match &actor.identifier {
            FnIdentifier::Scrypto(scrypto_fn_identifier) => {
                let maybe_component = match &actor.receiver {
                    Some(ResolvedReceiver {
                        derefed_from:
                            Some((RENodeId::Global(GlobalAddress::Component(component_address)), ..)),
                        ..
                    }) => Some(*component_address),
                    _ => None,
                };

                (scrypto_fn_identifier, maybe_component)
            }
            _ => {
                return Ok(());
            }
        };

        //========================
        // Apply package royalty
        //========================

        let package_id = {
            let node_id = RENodeId::Global(GlobalAddress::Package(
                scrypto_fn_identifier.package_address,
            ));
            let offset = SubstateOffset::Global(GlobalOffset::Global);
            track
                .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
                .map_err(RoyaltyError::from)?;
            let substate = track.get_substate(node_id, &offset);
            let package_id = match substate.global_address() {
                GlobalAddressSubstate::Package(id) => *id,
                _ => panic!("Unexpected global address substate type"),
            };
            track
                .release_lock(SubstateId(node_id, offset.clone()), false)
                .map_err(RoyaltyError::from)?;
            package_id
        };

        let node_id = RENodeId::Package(package_id);
        let offset = SubstateOffset::Package(PackageOffset::RoyaltyConfig);
        track
            .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
            .map_err(RoyaltyError::from)?;
        let substate = track.get_substate(node_id, &offset);
        let royalty = substate
            .package_royalty_config()
            .royalty_config
            .get(&scrypto_fn_identifier.blueprint_name)
            .map(|x| x.get_rule(&scrypto_fn_identifier.ident).clone())
            .unwrap_or(0);
        track
            .fee_reserve
            .consume_royalty(
                RoyaltyReceiver::Package(scrypto_fn_identifier.package_address, node_id),
                royalty,
            )
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
        track
            .release_lock(SubstateId(node_id, offset.clone()), false)
            .map_err(RoyaltyError::from)?;

        let offset = SubstateOffset::Package(PackageOffset::RoyaltyAccumulator);
        track
            .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::MUTABLE)
            .map_err(RoyaltyError::from)?;
        track
            .release_lock(SubstateId(node_id, offset.clone()), false)
            .map_err(RoyaltyError::from)?;

        //========================
        // Apply component royalty
        //========================

        if let Some(component_address) = optional_component_address {
            let component_id = {
                let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
                let offset = SubstateOffset::Global(GlobalOffset::Global);
                track
                    .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
                    .map_err(RoyaltyError::from)?;
                let substate = track.get_substate(node_id, &offset);
                let component_id = match substate.global_address() {
                    GlobalAddressSubstate::Component(id) => *id,
                    _ => panic!("Unexpected global address substate type"),
                };
                track
                    .release_lock(SubstateId(node_id, offset.clone()), false)
                    .map_err(RoyaltyError::from)?;
                component_id
            };

            let node_id = RENodeId::Component(component_id);
            let offset = SubstateOffset::Component(ComponentOffset::RoyaltyConfig);
            track
                .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
                .map_err(RoyaltyError::from)?;
            let substate = track.get_substate(node_id, &offset);
            let royalty = substate
                .component_royalty_config()
                .royalty_config
                .get_rule(&scrypto_fn_identifier.ident)
                .clone();
            track
                .fee_reserve
                .consume_royalty(
                    RoyaltyReceiver::Component(component_address, node_id),
                    royalty,
                )
                .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            track
                .release_lock(SubstateId(node_id, offset.clone()), false)
                .map_err(RoyaltyError::from)?;

            let offset = SubstateOffset::Component(ComponentOffset::RoyaltyAccumulator);
            track
                .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::MUTABLE)
                .map_err(RoyaltyError::from)?;
            track
                .release_lock(SubstateId(node_id, offset.clone()), false)
                .map_err(RoyaltyError::from)?;
        }

        Ok(())
    }
}
