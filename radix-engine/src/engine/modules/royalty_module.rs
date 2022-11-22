use crate::engine::*;
use crate::fee::FeeReserve;
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
        let (package_address, blueprint_name, fn_ident, optional_component_address) = match actor {
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
                    // TODO: does it make sense to apply royalty on local components?
                    if let Some((
                        RENodeId::Global(GlobalAddress::Component(component_address)),
                        _,
                    )) = receiver.derefed_from
                    {
                        (
                            package_address,
                            blueprint_name,
                            ident,
                            Some(component_address),
                        )
                    } else {
                        (package_address, blueprint_name, ident, None)
                    }
                }
                ResolvedMethod::Native(_) => {
                    return Ok(());
                }
            },
        };

        // Load package royalty config
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

        // Load component royalty config
        if let Some(component_address) = optional_component_address {
            let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
            let offset = SubstateOffset::Component(ComponentOffset::RoyaltyConfig);
            // TODO: deref
            track
                .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
                .map_err(RoyaltyError::from)?;
            let royalty_config = track
                .get_substate(node_id, &offset)
                .component_royalty_config();
            track
                .release_lock(SubstateId(node_id, offset.clone()), false)
                .map_err(RoyaltyError::from)?;

            // TODO: apply royalty
        }

        Ok(())
    }
}
