use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::package::{
    PackageDefinition,
};
use radix_engine_interface::*;
use radix_engine_interface::api::*;
use sbor::rust::prelude::*;
use crate::blueprints::access_controller::AccessControllerBlueprint;

pub struct AccessControllerNativePackage;

impl AccessControllerNativePackage {
    pub fn definition() -> PackageDefinition {
        let blueprints = btreemap!(
            ACCESS_CONTROLLER_BLUEPRINT.to_string() => AccessControllerBlueprint::definition()
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        match export_name {
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT => AccessControllerBlueprint::create_global(input, api),
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => AccessControllerBlueprint::create_proof(input, api),
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => {
                AccessControllerBlueprint::initiate_recovery_as_primary(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => {
                AccessControllerBlueprint::initiate_recovery_as_recovery(input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                AccessControllerBlueprint::quick_confirm_primary_role_recovery_proposal(&receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                AccessControllerBlueprint::quick_confirm_recovery_role_recovery_proposal(&receiver, input, api)
            }
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                AccessControllerBlueprint::timed_confirm_recovery(&receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                AccessControllerBlueprint::cancel_primary_role_recovery_proposal(input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                AccessControllerBlueprint::cancel_recovery_role_recovery_proposal(input, api)
            }
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT => AccessControllerBlueprint::lock_primary_role(input, api),
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT => AccessControllerBlueprint::unlock_primary_role(input, api),
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => AccessControllerBlueprint::stop_timed_recovery(input, api),
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT => {
                AccessControllerBlueprint::initiate_badge_withdraw_attempt_as_primary(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT => {
                AccessControllerBlueprint::initiate_badge_withdraw_attempt_as_recovery(input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                AccessControllerBlueprint::quick_confirm_primary_role_badge_withdraw_attempt(&receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                AccessControllerBlueprint::quick_confirm_recovery_role_badge_withdraw_attempt(&receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                AccessControllerBlueprint::cancel_primary_role_badge_withdraw_attempt(input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                AccessControllerBlueprint::cancel_recovery_role_badge_withdraw_attempt(input, api)
            }
            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT => AccessControllerBlueprint::mint_recovery_badges(input, api),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
