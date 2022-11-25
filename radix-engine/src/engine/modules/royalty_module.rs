use crate::engine::*;
use crate::fee::{FeeReserve, RoyaltyCollector};
use radix_engine_interface::api::types::{
    ComponentOffset, GlobalAddress, PackageOffset, RENodeId, SubstateId, SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;
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

impl RoyaltyModule {
    pub fn new() -> Self {
        Self {}
    }
}

impl<R: FeeReserve> Module<R> for RoyaltyModule {
    fn pre_execute_invocation(
        &mut self,
        actor: &REActor,
        _input: &IndexedScryptoValue,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        // Identify the function, and optional component address
        let (package_address, blueprint_name, fn_ident, optional_component_id) = match actor {
            REActor::Function(function) => match function {
                ResolvedFunction::Scrypto {
                    package_address,
                    blueprint_name,
                    ident,
                    ..
                } => (package_address, blueprint_name, ident, None),
                ResolvedFunction::Native(_) => {
                    return Ok(());
                }
            },
            REActor::Method(method, receiver) => match method {
                ResolvedMethod::Scrypto {
                    package_address,
                    blueprint_name,
                    ident,
                    ..
                } => {
                    if let RENodeId::Component(component_id) = receiver.receiver {
                        (package_address, blueprint_name, ident, Some(component_id))
                    } else {
                        (package_address, blueprint_name, ident, None)
                    }
                }
                ResolvedMethod::Native(_) => {
                    return Ok(());
                }
            },
        };

        // Apply package royalty config
        let node_id = RENodeId::Global(GlobalAddress::Package(*package_address));
        let offset = SubstateOffset::Package(PackageOffset::RoyaltyConfig);
        // TODO: deref
        track
            .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
            .map_err(RoyaltyError::from)?;
        let royalty_config = track
            .get_substate(node_id, &offset)
            .package_royalty_config();
        track
            .release_lock(SubstateId(node_id, offset.clone()), false)
            .map_err(RoyaltyError::from)?;
        // TODO: apply royalty

        // Apply component royalty config
        if let Some(component_id) = optional_component_id {
            let node_id = RENodeId::Component(component_id);
            let offset = SubstateOffset::Component(ComponentOffset::RoyaltyConfig);
            track
                .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
                .map_err(RoyaltyError::from)?;
            let substate = track.get_substate(node_id, &offset);
            let royalty = substate
                .component_royalty_config()
                .royalty_config
                .get_rule(fn_ident)
                .clone();
            track
                .fee_reserve
                .consume_royalty(RoyaltyCollector::Component(component_id), royalty)
                .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            track
                .release_lock(SubstateId(node_id, offset.clone()), false)
                .map_err(RoyaltyError::from)?;
        }

        Ok(())
    }
}
