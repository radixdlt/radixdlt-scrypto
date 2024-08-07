use super::internal_prelude::*;
use crate::internal_prelude::*;
use radix_engine_interface::api::field_api::*;
use radix_engine_interface::api::object_api::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::*;
use radix_engine_interface::*;
use radix_native_sdk::modules::metadata::*;
use radix_native_sdk::modules::role_assignment::*;
use radix_native_sdk::resource::*;
use radix_native_sdk::runtime::*;
use sbor::rust::prelude::*;

pub struct AccessControllerV1Blueprint;

impl AccessControllerV1Blueprint {
    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        dispatch! {
            IDENT,
            export_name,
            input,
            api,
            AccessController,
            [
                create,
                create_proof,
                initiate_recovery_as_primary,
                initiate_recovery_as_recovery,
                initiate_badge_withdraw_attempt_as_primary,
                initiate_badge_withdraw_attempt_as_recovery,
                quick_confirm_primary_role_recovery_proposal,
                quick_confirm_recovery_role_recovery_proposal,
                quick_confirm_primary_role_badge_withdraw_attempt,
                quick_confirm_recovery_role_badge_withdraw_attempt,
                timed_confirm_recovery,
                cancel_primary_role_recovery_proposal,
                cancel_recovery_role_recovery_proposal,
                cancel_primary_role_badge_withdraw_attempt,
                cancel_recovery_role_badge_withdraw_attempt,
                lock_primary_role,
                unlock_primary_role,
                stop_timed_recovery,
                mint_recovery_badges,
            ]
        }
    }

    pub fn create<Y: SystemApi<RuntimeError>>(
        AccessControllerCreateInput {
            controlled_asset,
            rule_set,
            timed_recovery_delay_in_minutes,
            address_reservation,
        }: AccessControllerCreateInput,
        api: &mut Y,
    ) -> Result<AccessControllerCreateOutput, RuntimeError> {
        // Allocating the address of the access controller - this will be needed for the metadata
        // and access rules of the recovery badge
        let (address_reservation, address) = {
            if let Some(address_reservation) = address_reservation {
                let address = api.get_reservation_address(address_reservation.0.as_node_id())?;
                (address_reservation, address)
            } else {
                api.allocate_global_address(BlueprintId {
                    package_address: ACCESS_CONTROLLER_PACKAGE,
                    blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
                })?
            }
        };

        // Creating a new vault and putting in it the controlled asset
        let vault = {
            let mut vault = controlled_asset
                .resource_address(api)
                .and_then(|resource_address| Vault::create(resource_address, api))?;
            vault.put(controlled_asset, api)?;

            vault
        };

        // Creating a new recovery badge resource
        let recovery_badge_resource = {
            let global_component_caller_badge =
                NonFungibleGlobalId::global_caller_badge(GlobalCaller::GlobalObject(address));

            let resource_address = {
                let non_fungible_schema =
                    NonFungibleDataSchema::new_local_without_self_package_replacement::<()>();

                let result = api.call_function(
                    RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                    scrypto_encode(&NonFungibleResourceManagerCreateInput {
                        owner_role: OwnerRole::Fixed(rule!(require(global_component_caller_badge.clone()))),
                        id_type: NonFungibleIdType::Integer,
                        track_total_supply: true,
                        non_fungible_schema,
                        resource_roles: NonFungibleResourceRoles {
                            mint_roles: mint_roles! {
                                minter => rule!(require(global_component_caller_badge.clone()));
                                minter_updater => rule!(deny_all);
                            },
                            burn_roles: burn_roles! {
                                burner => rule!(allow_all);
                                burner_updater => rule!(allow_all);
                            },
                            withdraw_roles: withdraw_roles! {
                                withdrawer => rule!(deny_all);
                                withdrawer_updater => rule!(deny_all);
                            },
                            ..Default::default()
                        },
                        metadata: metadata! {
                            roles {
                                metadata_setter => AccessRule::DenyAll;
                                metadata_setter_updater => AccessRule::DenyAll;
                                metadata_locker => AccessRule::DenyAll;
                                metadata_locker_updater => AccessRule::DenyAll;
                            },
                            init {
                                "name" => "Recovery Badge".to_owned(), locked;
                                "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-recovery_badge.png"), locked;
                                "access_controller" => address, locked;
                            }
                        },
                        address_reservation: None,
                    })
                        .unwrap(),
                )?;
                scrypto_decode::<ResourceAddress>(result.as_slice()).unwrap()
            };

            resource_address
        };

        let substate = AccessControllerV1Substate::new(
            vault,
            timed_recovery_delay_in_minutes,
            recovery_badge_resource,
        );
        let object_id = api.new_simple_object(
            ACCESS_CONTROLLER_BLUEPRINT,
            indexmap! {
                AccessControllerField::State.field_index() => FieldValue::new(
                    AccessControllerStateFieldPayload::from_content_source(substate)
                ),
            },
        )?;

        let roles = init_roles_from_rule_set(rule_set);
        let roles = indexmap!(ModuleId::Main => roles);
        let role_assignment = RoleAssignment::create(OwnerRole::None, roles, api)?.0;

        let metadata = Metadata::create_with_data(
            metadata_init! {
                "recovery_badge" => GlobalAddress::from(recovery_badge_resource), locked;
            },
            api,
        )?;

        // Creating a global component address for the access controller RENode
        api.globalize(
            object_id,
            indexmap!(
                AttachedModuleId::RoleAssignment => role_assignment.0,
                AttachedModuleId::Metadata => metadata.0,
            ),
            Some(address_reservation),
        )?;

        Ok(Global::new(ComponentAddress::try_from(address).unwrap()))
    }

    pub fn create_proof<Y: SystemApi<RuntimeError>>(
        _: AccessControllerCreateProofInput,
        api: &mut Y,
    ) -> Result<AccessControllerCreateProofOutput, RuntimeError> {
        transition(api, AccessControllerCreateProofStateMachineInput)
    }

    pub fn initiate_recovery_as_primary<Y: SystemApi<RuntimeError>>(
        AccessControllerInitiateRecoveryAsPrimaryInput {
            rule_set,
            timed_recovery_delay_in_minutes,
        }: AccessControllerInitiateRecoveryAsPrimaryInput,
        api: &mut Y,
    ) -> Result<AccessControllerInitiateRecoveryAsPrimaryOutput, RuntimeError> {
        let proposal = RecoveryProposal {
            rule_set,
            timed_recovery_delay_in_minutes,
        };

        transition_mut(
            api,
            AccessControllerInitiateRecoveryAsPrimaryStateMachineInput {
                proposal: proposal.clone(),
            },
        )?;

        Runtime::emit_event(
            api,
            InitiateRecoveryEvent {
                proposal,
                proposer: Proposer::Primary,
            },
        )?;

        Ok(())
    }

    pub fn initiate_recovery_as_recovery<Y: SystemApi<RuntimeError>>(
        AccessControllerInitiateRecoveryAsRecoveryInput {
            rule_set,
            timed_recovery_delay_in_minutes,
        }: AccessControllerInitiateRecoveryAsRecoveryInput,
        api: &mut Y,
    ) -> Result<AccessControllerInitiateRecoveryAsRecoveryOutput, RuntimeError> {
        let proposal = RecoveryProposal {
            rule_set,
            timed_recovery_delay_in_minutes,
        };

        transition_mut(
            api,
            AccessControllerInitiateRecoveryAsRecoveryStateMachineInput {
                proposal: proposal.clone(),
            },
        )?;

        Runtime::emit_event(
            api,
            InitiateRecoveryEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(())
    }

    pub fn initiate_badge_withdraw_attempt_as_primary<Y: SystemApi<RuntimeError>>(
        AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput { .. }: AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput,
        api: &mut Y,
    ) -> Result<AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryOutput, RuntimeError> {
        transition_mut(
            api,
            AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            InitiateBadgeWithdrawAttemptEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(())
    }

    pub fn initiate_badge_withdraw_attempt_as_recovery<Y: SystemApi<RuntimeError>>(
        _: AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput,
        api: &mut Y,
    ) -> Result<AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryOutput, RuntimeError> {
        transition_mut(
            api,
            AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            InitiateBadgeWithdrawAttemptEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(())
    }

    pub fn quick_confirm_primary_role_recovery_proposal<Y: SystemApi<RuntimeError>>(
        AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
            rule_set,
            timed_recovery_delay_in_minutes,
        }: AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput,
        api: &mut Y,
    ) -> Result<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalOutput, RuntimeError> {
        let proposal = RecoveryProposal {
            rule_set,
            timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            api,
            AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        let receiver = Runtime::get_node_id(api)?;
        update_role_assignment(api, &receiver, recovery_proposal.rule_set)?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Primary,
            },
        )?;

        Ok(())
    }

    pub fn quick_confirm_recovery_role_recovery_proposal<Y: SystemApi<RuntimeError>>(
        AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput {
            rule_set,
            timed_recovery_delay_in_minutes,
        }: AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput,
        api: &mut Y,
    ) -> Result<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalOutput, RuntimeError> {
        let proposal = RecoveryProposal {
            rule_set,
            timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            api,
            AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        let receiver = Runtime::get_node_id(api)?;
        update_role_assignment(api, &receiver, recovery_proposal.rule_set)?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(())
    }

    pub fn quick_confirm_primary_role_badge_withdraw_attempt<Y: SystemApi<RuntimeError>>(
        _: AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput,
        api: &mut Y,
    ) -> Result<AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptOutput, RuntimeError>
    {
        let bucket = transition_mut(
            api,
            AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        let receiver = Runtime::get_node_id(api)?;
        update_role_assignment(api, &receiver, locked_role_assignment())?;

        Runtime::emit_event(
            api,
            BadgeWithdrawEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(bucket)
    }

    pub fn quick_confirm_recovery_role_badge_withdraw_attempt<Y: SystemApi<RuntimeError>>(
        _: AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput,
        api: &mut Y,
    ) -> Result<AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptOutput, RuntimeError>
    {
        let bucket = transition_mut(
            api,
            AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        let receiver = Runtime::get_node_id(api)?;
        update_role_assignment(api, &receiver, locked_role_assignment())?;

        Runtime::emit_event(
            api,
            BadgeWithdrawEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(bucket)
    }

    pub fn timed_confirm_recovery<Y: SystemApi<RuntimeError>>(
        AccessControllerTimedConfirmRecoveryInput {
            rule_set,
            timed_recovery_delay_in_minutes,
        }: AccessControllerTimedConfirmRecoveryInput,
        api: &mut Y,
    ) -> Result<AccessControllerTimedConfirmRecoveryOutput, RuntimeError> {
        let proposal = RecoveryProposal {
            rule_set,
            timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            api,
            AccessControllerTimedConfirmRecoveryStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        // Update the access rules
        let receiver = Runtime::get_node_id(api)?;
        update_role_assignment(api, &receiver, recovery_proposal.rule_set)?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(())
    }

    pub fn cancel_primary_role_recovery_proposal<Y: SystemApi<RuntimeError>>(
        AccessControllerCancelPrimaryRoleRecoveryProposalInput { .. }: AccessControllerCancelPrimaryRoleRecoveryProposalInput,
        api: &mut Y,
    ) -> Result<AccessControllerCancelPrimaryRoleRecoveryProposalOutput, RuntimeError> {
        transition_mut(
            api,
            AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelRecoveryProposalEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(())
    }

    pub fn cancel_recovery_role_recovery_proposal<Y: SystemApi<RuntimeError>>(
        AccessControllerCancelRecoveryRoleRecoveryProposalInput { .. }: AccessControllerCancelRecoveryRoleRecoveryProposalInput,
        api: &mut Y,
    ) -> Result<AccessControllerCancelRecoveryRoleRecoveryProposalOutput, RuntimeError> {
        transition_mut(
            api,
            AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelRecoveryProposalEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(())
    }

    pub fn cancel_primary_role_badge_withdraw_attempt<Y: SystemApi<RuntimeError>>(
        AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput { .. }: AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput,
        api: &mut Y,
    ) -> Result<AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptOutput, RuntimeError> {
        transition_mut(
            api,
            AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelBadgeWithdrawAttemptEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(())
    }

    pub fn cancel_recovery_role_badge_withdraw_attempt<Y: SystemApi<RuntimeError>>(
        AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput { .. }: AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput,
        api: &mut Y,
    ) -> Result<AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptOutput, RuntimeError> {
        transition_mut(
            api,
            AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelBadgeWithdrawAttemptEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(())
    }

    pub fn lock_primary_role<Y: SystemApi<RuntimeError>>(
        AccessControllerLockPrimaryRoleInput { .. }: AccessControllerLockPrimaryRoleInput,
        api: &mut Y,
    ) -> Result<AccessControllerLockPrimaryRoleOutput, RuntimeError> {
        transition_mut(api, AccessControllerLockPrimaryRoleStateMachineInput)?;
        Runtime::emit_event(api, LockPrimaryRoleEvent {})?;

        Ok(())
    }

    pub fn unlock_primary_role<Y: SystemApi<RuntimeError>>(
        _: AccessControllerUnlockPrimaryRoleInput,
        api: &mut Y,
    ) -> Result<AccessControllerUnlockPrimaryRoleOutput, RuntimeError> {
        transition_mut(api, AccessControllerUnlockPrimaryRoleStateMachineInput)?;
        Runtime::emit_event(api, UnlockPrimaryRoleEvent {})?;

        Ok(())
    }

    pub fn stop_timed_recovery<Y: SystemApi<RuntimeError>>(
        AccessControllerStopTimedRecoveryInput {
            rule_set,
            timed_recovery_delay_in_minutes,
        }: AccessControllerStopTimedRecoveryInput,
        api: &mut Y,
    ) -> Result<AccessControllerStopTimedRecoveryOutput, RuntimeError> {
        transition_mut(
            api,
            AccessControllerStopTimedRecoveryStateMachineInput {
                proposal: RecoveryProposal {
                    rule_set,
                    timed_recovery_delay_in_minutes,
                },
            },
        )?;
        Runtime::emit_event(api, StopTimedRecoveryEvent)?;

        Ok(())
    }

    pub fn mint_recovery_badges<Y: SystemApi<RuntimeError>>(
        AccessControllerMintRecoveryBadgesInput {
            non_fungible_local_ids,
        }: AccessControllerMintRecoveryBadgesInput,
        api: &mut Y,
    ) -> Result<AccessControllerMintRecoveryBadgesOutput, RuntimeError> {
        let resource_address = {
            let handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                AccessControllerField::State.field_index(),
                LockFlags::read_only(),
            )?;

            let access_controller = {
                let access_controller: AccessControllerStateFieldPayload =
                    api.field_read_typed(handle)?;
                access_controller.fully_update_and_into_latest_version()
            };
            access_controller.recovery_badge
        };

        let non_fungibles: IndexMap<NonFungibleLocalId, (ScryptoValue,)> = non_fungible_local_ids
            .into_iter()
            .map(|local_id| {
                (
                    local_id,
                    (scrypto_decode(&scrypto_encode(&()).unwrap()).unwrap(),),
                )
            })
            .collect();

        let bucket = api
            .call_method(
                resource_address.as_node_id(),
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                scrypto_encode(&NonFungibleResourceManagerMintInput {
                    entries: non_fungibles,
                })
                .unwrap(),
            )
            .map(|buffer| {
                scrypto_decode::<NonFungibleResourceManagerMintOutput>(&buffer).unwrap()
            })?;

        Ok(bucket)
    }
}

//=========
// Helpers
//=========

fn locked_role_assignment() -> RuleSet {
    RuleSet {
        primary_role: AccessRule::DenyAll,
        recovery_role: AccessRule::DenyAll,
        confirmation_role: AccessRule::DenyAll,
    }
}

fn init_roles_from_rule_set(rule_set: RuleSet) -> RoleAssignmentInit {
    roles2! {
        "primary" => rule_set.primary_role, updatable;
        "recovery" => rule_set.recovery_role, updatable;
        "confirmation" => rule_set.confirmation_role, updatable;
    }
}

fn transition<Y: SystemApi<RuntimeError>, I>(
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerV1Substate as Transition<I>>::Output, RuntimeError>
where
    AccessControllerV1Substate: Transition<I>,
{
    let handle = api.actor_open_field(
        ACTOR_STATE_SELF,
        AccessControllerField::State.field_index(),
        LockFlags::read_only(),
    )?;

    let access_controller = {
        let access_controller: AccessControllerStateFieldPayload = api.field_read_typed(handle)?;
        access_controller.fully_update_and_into_latest_version()
    };

    let rtn = access_controller.transition(api, input)?;

    api.field_close(handle)?;

    Ok(rtn)
}

fn transition_mut<Y: SystemApi<RuntimeError>, I>(
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerV1Substate as TransitionMut<I>>::Output, RuntimeError>
where
    AccessControllerV1Substate: TransitionMut<I>,
{
    let handle = api.actor_open_field(
        ACTOR_STATE_SELF,
        AccessControllerField::State.field_index(),
        LockFlags::MUTABLE,
    )?;

    let mut access_controller = {
        let access_controller: AccessControllerStateFieldPayload = api.field_read_typed(handle)?;
        access_controller.fully_update_and_into_latest_version()
    };

    let rtn = access_controller.transition_mut(api, input)?;

    {
        api.field_write_typed(
            handle,
            &AccessControllerStateFieldPayload::from_content_source(access_controller),
        )?;
    }

    api.field_close(handle)?;

    Ok(rtn)
}

fn update_role_assignment<Y: SystemApi<RuntimeError>>(
    api: &mut Y,
    receiver: &NodeId,
    rule_set: RuleSet,
) -> Result<(), RuntimeError> {
    let attached = AttachedRoleAssignment(*receiver);
    attached.set_role(
        ModuleId::Main,
        RoleKey::new("primary"),
        rule_set.primary_role.clone(),
        api,
    )?;
    attached.set_role(
        ModuleId::Main,
        RoleKey::new("recovery"),
        rule_set.recovery_role.clone(),
        api,
    )?;
    attached.set_role(
        ModuleId::Main,
        RoleKey::new("confirmation"),
        rule_set.confirmation_role.clone(),
        api,
    )?;

    Ok(())
}
