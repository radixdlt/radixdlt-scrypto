use crate::blueprints::resource::AuthZone;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::kernel_modules::auth::*;
use crate::types::*;
use native_sdk::resource::SysProof;
use radix_engine_interface::api::{ClientObjectApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::ops::Fn;

pub struct Authentication;

impl Authentication {
    fn proof_matches<Y: KernelSubstateApi + ClientObjectApi<RuntimeError>>(
        resource_rule: &HardResourceOrNonFungible,
        proof: &Proof,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match resource_rule {
            HardResourceOrNonFungible::NonFungible(non_fungible_global_id) => {
                let proof_resource_address = proof.sys_resource_address(api)?;
                Ok(
                    proof_resource_address == non_fungible_global_id.resource_address()
                        && proof
                            .sys_non_fungible_local_ids(api)?
                            .contains(non_fungible_global_id.local_id()),
                )
            }
            HardResourceOrNonFungible::Resource(resource_address) => {
                let proof_resource_address = proof.sys_resource_address(api)?;
                Ok(proof_resource_address == *resource_address)
            }
            // TODO: I believe team wants to propagate these error codes?
            HardResourceOrNonFungible::InvalidPath
            | HardResourceOrNonFungible::NotResourceAddress
            | HardResourceOrNonFungible::NotResourceAddressOrNonFungibleGlobalId => Ok(false),
        }
    }

    fn auth_zone_stack_matches<P, Y>(
        barrier_crossings_required: u32,
        barrier_crossings_allowed: u32,
        auth_zone_id: NodeId,
        api: &mut Y,
        check: P,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelSubstateApi + ClientObjectApi<RuntimeError>,
        P: Fn(&AuthZone, usize, &mut Y) -> Result<bool, RuntimeError>,
    {
        let mut remaining_barrier_crossings_required = barrier_crossings_required;
        let mut remaining_barrier_crossings_allowed = barrier_crossings_allowed;
        let mut current_auth_zone_id = auth_zone_id;
        let mut rev_index = 0;
        let mut handles = Vec::new();
        let mut pass = false;
        loop {
            // Load auth zone
            let handle = api.kernel_lock_substate(
                &current_auth_zone_id,
                TypedModuleId::ObjectState,
                &AuthZoneOffset::AuthZone.into(),
                LockFlags::read_only(),
            )?;
            let auth_zone: AuthZone = api.kernel_read_substate(handle)?.as_typed().unwrap();
            let auth_zone = auth_zone.clone();
            handles.push(handle);

            if remaining_barrier_crossings_required == 0 {
                // Check
                if check(&auth_zone, rev_index, api)? {
                    pass = true;
                    break;
                }
                rev_index += 1;
            }

            // Progress
            if auth_zone.is_barrier {
                if remaining_barrier_crossings_allowed == 0 {
                    break;
                }
                remaining_barrier_crossings_allowed -= 1;

                if remaining_barrier_crossings_required > 0 {
                    remaining_barrier_crossings_required -= 1;
                }
            }

            if let Some(id) = auth_zone.parent {
                current_auth_zone_id = id.into();
            } else {
                break;
            }
        }

        for handle in handles {
            api.kernel_drop_lock(handle)?;
        }

        Ok(pass)
    }

    fn auth_zone_stack_has_amount<Y: KernelSubstateApi + ClientObjectApi<RuntimeError>>(
        barrier_crossings_required: u32,
        barrier_crossings_allowed: u32,
        auth_zone_id: NodeId,
        resource_rule: &HardResourceOrNonFungible,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(
            barrier_crossings_required,
            barrier_crossings_allowed,
            auth_zone_id,
            api,
            |auth_zone, _, api| {
                // FIXME: Need to check the composite max amount rather than just each proof individually
                for p in auth_zone.proofs() {
                    if Self::proof_matches(resource_rule, p, api)? && p.sys_amount(api)? >= amount {
                        return Ok(true);
                    }
                }

                Ok(false)
            },
        )
    }

    fn auth_zone_stack_matches_rule<Y: KernelSubstateApi + ClientObjectApi<RuntimeError>>(
        barrier_crossings_required: u32,
        barrier_crossings_allowed: u32,
        auth_zone_id: NodeId,
        resource_rule: &HardResourceOrNonFungible,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(
            barrier_crossings_required,
            barrier_crossings_allowed,
            auth_zone_id,
            api,
            |auth_zone, rev_index, api| {
                if let HardResourceOrNonFungible::NonFungible(non_fungible_global_id) =
                    resource_rule
                {
                    if rev_index == 0 {
                        if auth_zone
                            .virtual_non_fungibles_non_extending()
                            .contains(&non_fungible_global_id)
                        {
                            return Ok(true);
                        }
                    }

                    if auth_zone
                        .virtual_non_fungibles()
                        .contains(&non_fungible_global_id)
                    {
                        return Ok(true);
                    }
                    if auth_zone
                        .virtual_resources()
                        .contains(&non_fungible_global_id.resource_address())
                    {
                        return Ok(true);
                    }
                }

                for p in auth_zone.proofs() {
                    if Self::proof_matches(resource_rule, p, api)? {
                        return Ok(true);
                    }
                }

                Ok(false)
            },
        )
    }

    pub fn verify_proof_rule<Y: KernelSubstateApi + ClientObjectApi<RuntimeError>>(
        barrier_crossings_required: u32,
        barrier_crossings_allowed: u32,
        auth_zone_id: NodeId,
        proof_rule: &HardProofRule,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match proof_rule {
            HardProofRule::Require(resource) => {
                if Self::auth_zone_stack_matches_rule(
                    barrier_crossings_required,
                    barrier_crossings_allowed,
                    auth_zone_id,
                    resource,
                    api,
                )? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            HardProofRule::AmountOf(HardDecimal::Amount(amount), resource) => {
                if Self::auth_zone_stack_has_amount(
                    barrier_crossings_required,
                    barrier_crossings_allowed,
                    auth_zone_id,
                    resource,
                    *amount,
                    api,
                )? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            HardProofRule::AllOf(HardProofRuleResourceList::List(resources)) => {
                for resource in resources {
                    if !Self::auth_zone_stack_matches_rule(
                        barrier_crossings_required,
                        barrier_crossings_allowed,
                        auth_zone_id,
                        resource,
                        api,
                    )? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            HardProofRule::AnyOf(HardProofRuleResourceList::List(resources)) => {
                for resource in resources {
                    if Self::auth_zone_stack_matches_rule(
                        barrier_crossings_required,
                        barrier_crossings_allowed,
                        auth_zone_id,
                        resource,
                        api,
                    )? {
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            HardProofRule::CountOf(
                HardCount::Count(count),
                HardProofRuleResourceList::List(resources),
            ) => {
                let mut left = count.clone();
                for resource in resources {
                    if Self::auth_zone_stack_matches_rule(
                        barrier_crossings_required,
                        barrier_crossings_allowed,
                        auth_zone_id,
                        resource,
                        api,
                    )? {
                        left -= 1;
                        if left == 0 {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    pub fn verify_auth_rule<Y: KernelSubstateApi + ClientObjectApi<RuntimeError>>(
        barrier_crossings_required: u32,
        barrier_crossings_allowed: u32,
        auth_zone_id: NodeId,
        auth_rule: &HardAuthRule,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match auth_rule {
            HardAuthRule::ProofRule(rule) => Self::verify_proof_rule(
                barrier_crossings_required,
                barrier_crossings_allowed,
                auth_zone_id,
                rule,
                api,
            ),
            HardAuthRule::AnyOf(rules) => {
                for r in rules {
                    if Self::verify_auth_rule(
                        barrier_crossings_required,
                        barrier_crossings_allowed,
                        auth_zone_id,
                        r,
                        api,
                    )? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            HardAuthRule::AllOf(rules) => {
                for r in rules {
                    if !Self::verify_auth_rule(
                        barrier_crossings_required,
                        barrier_crossings_allowed,
                        auth_zone_id,
                        r,
                        api,
                    )? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
        }
    }

    pub fn verify_method_auth<Y: KernelSubstateApi + ClientObjectApi<RuntimeError>>(
        barrier_crossings_required: u32,
        barrier_crossings_allowed: u32,
        auth_zone_id: NodeId,
        method_auth: &MethodAuthorization,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match method_auth {
            MethodAuthorization::Protected(rule) => Self::verify_auth_rule(
                barrier_crossings_required,
                barrier_crossings_allowed,
                auth_zone_id,
                rule,
                api,
            ),
            MethodAuthorization::AllowAll => Ok(true),
            MethodAuthorization::DenyAll => Ok(false),
        }
    }
}
