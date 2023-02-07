use crate::errors::ModuleError;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate;
use crate::system::kernel_modules::fee::{
    CostingError, ExecutionFeeReserve, FeeReserve, RoyaltyReceiver,
};
use radix_engine_interface::api::types::{
    FnIdentifier, GlobalAddress, GlobalOffset, NodeModuleId, RENodeId, RoyaltyOffset, SubstateId,
    SubstateOffset, VaultOffset,
};
use radix_engine_interface::*;
use radix_engine_interface::constants::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum RoyaltyError {
    TrackError(TrackError),
}

pub struct RoyaltyModule {}

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

macro_rules! preload_vault {
    ($track:expr, $royalty_vault:expr) => {
        let vault_node_id = RENodeId::Vault($royalty_vault.vault_id());
        $track
            .acquire_lock(
                SubstateId(
                    vault_node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::Vault),
                ),
                LockFlags::MUTABLE,
            )
            .map_err(RoyaltyError::from)?;
        $track
            .release_lock(
                SubstateId(
                    vault_node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::Vault),
                ),
                false,
            )
            .map_err(RoyaltyError::from)?;
    };
}

impl<R: FeeReserve> BaseModule<R> for RoyaltyModule {
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
            match scrypto_fn_identifier.package_address {
                IDENTITY_PACKAGE | EPOCH_MANAGER_PACKAGE | CLOCK_PACKAGE => {
                    return Ok(());
                }
                _ => {
                }
            }

            let node_id = RENodeId::Global(GlobalAddress::Package(
                scrypto_fn_identifier.package_address,
            ));
            let offset = SubstateOffset::Global(GlobalOffset::Global);
            track
                .acquire_lock(
                    SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                    LockFlags::read_only(),
                )
                .map_err(RoyaltyError::from)?;
            let substate = track.get_substate(node_id, NodeModuleId::SELF, &offset);
            let package_id = match substate.global_address() {
                GlobalAddressSubstate::Package(id) => *id,
                _ => panic!("Unexpected global address substate type"),
            };
            track
                .release_lock(
                    SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                    false,
                )
                .map_err(RoyaltyError::from)?;
            package_id
        };

        let node_id = RENodeId::Package(package_id);
        let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
        track
            .acquire_lock(
                SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
                LockFlags::read_only(),
            )
            .map_err(RoyaltyError::from)?;
        let substate = track.get_substate(node_id, NodeModuleId::PackageRoyalty, &offset);
        let royalty = substate
            .package_royalty_config()
            .royalty_config
            .get(&scrypto_fn_identifier.blueprint_name)
            .map(|x| x.get_rule(&scrypto_fn_identifier.ident).clone())
            .unwrap_or(0);
        track
            .fee_reserve()
            .consume_royalty(
                RoyaltyReceiver::Package(scrypto_fn_identifier.package_address, node_id),
                royalty,
            )
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
        track
            .release_lock(
                SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
                false,
            )
            .map_err(RoyaltyError::from)?;

        // Pre-load accumulator and royalty vault substate to avoid additional substate loading
        // during track finalization.
        // TODO: refactor to defer substate loading to finalization.
        let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
        track
            .acquire_lock(
                SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
                LockFlags::MUTABLE,
            )
            .map_err(RoyaltyError::from)?;
        let royalty_vault = track
            .get_substate(node_id, NodeModuleId::PackageRoyalty, &offset)
            .package_royalty_accumulator()
            .royalty
            .clone();
        preload_vault!(track, royalty_vault);
        track
            .release_lock(
                SubstateId(node_id, NodeModuleId::PackageRoyalty, offset.clone()),
                false,
            )
            .map_err(RoyaltyError::from)?;

        //========================
        // Apply component royalty
        //========================

        if let Some(component_address) = optional_component_address {
            let component_id = {
                let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
                let offset = SubstateOffset::Global(GlobalOffset::Global);
                track
                    .acquire_lock(
                        SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                        LockFlags::read_only(),
                    )
                    .map_err(RoyaltyError::from)?;
                let substate = track.get_substate(node_id, NodeModuleId::SELF, &offset);
                let component_id = match substate.global_address() {
                    GlobalAddressSubstate::Component(id) => *id,
                    _ => panic!("Unexpected global address substate type"),
                };
                track
                    .release_lock(
                        SubstateId(node_id, NodeModuleId::SELF, offset.clone()),
                        false,
                    )
                    .map_err(RoyaltyError::from)?;
                component_id
            };

            let node_id = RENodeId::Component(component_id);
            let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
            track
                .acquire_lock(
                    SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                    LockFlags::read_only(),
                )
                .map_err(RoyaltyError::from)?;
            let substate = track.get_substate(node_id, NodeModuleId::ComponentRoyalty, &offset);
            let royalty = substate
                .component_royalty_config()
                .royalty_config
                .get_rule(&scrypto_fn_identifier.ident)
                .clone();
            track
                .fee_reserve()
                .consume_royalty(
                    RoyaltyReceiver::Component(component_address, node_id),
                    royalty,
                )
                .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
            track
                .release_lock(
                    SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                    false,
                )
                .map_err(RoyaltyError::from)?;

            // Pre-load accumulator and royalty vault substate to avoid additional substate loading
            // during track finalization.
            // TODO: refactor to defer substate loading to finalization.
            let offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
            track
                .acquire_lock(
                    SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                    LockFlags::MUTABLE,
                )
                .map_err(RoyaltyError::from)?;
            let royalty_vault = track
                .get_substate(node_id, NodeModuleId::ComponentRoyalty, &offset)
                .component_royalty_accumulator()
                .royalty
                .clone();
            preload_vault!(track, royalty_vault);
            track
                .release_lock(
                    SubstateId(node_id, NodeModuleId::ComponentRoyalty, offset.clone()),
                    false,
                )
                .map_err(RoyaltyError::from)?;
        }

        Ok(())
    }
}
