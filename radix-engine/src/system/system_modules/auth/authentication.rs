use crate::blueprints::resource::AuthZone;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::system_callback::SystemLockData;
use crate::types::*;
use native_sdk::resource::SysProof;
use radix_engine_interface::api::{ClientApi, ClientObjectApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::ops::Fn;

// TODO: Refactor structure to be able to remove this
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActingLocation {
    AtBarrier,
    AtLocalBarrier,
    InCallFrame,
}

pub struct Authentication;

impl Authentication {
    fn proof_matches<Y: KernelSubstateApi<SystemLockData> + ClientObjectApi<RuntimeError>>(
        resource_rule: &ResourceOrNonFungible,
        proof: &Proof,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match resource_rule {
            ResourceOrNonFungible::NonFungible(non_fungible_global_id) => {
                let proof_resource_address = proof.sys_resource_address(api)?;
                Ok(
                    proof_resource_address == non_fungible_global_id.resource_address()
                        && proof
                            .sys_non_fungible_local_ids(api)?
                            .contains(non_fungible_global_id.local_id()),
                )
            }
            ResourceOrNonFungible::Resource(resource_address) => {
                let proof_resource_address = proof.sys_resource_address(api)?;
                Ok(proof_resource_address == *resource_address)
            }
        }
    }

    fn auth_zone_stack_matches<P, Y>(
        acting_location: ActingLocation,
        auth_zone_id: NodeId,
        api: &mut Y,
        check: P,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData> + ClientObjectApi<RuntimeError>,
        P: Fn(&AuthZone, usize, bool, &mut Y) -> Result<bool, RuntimeError>,
    {
        let (
            mut is_first_barrier,
            mut waiting_for_barrier,
            mut remaining_barrier_crossings_allowed,
            mut skip,
        ) = match acting_location {
            ActingLocation::AtBarrier => (true, 0, 0, 0),
            ActingLocation::AtLocalBarrier => (false, 1, 1, 0),
            ActingLocation::InCallFrame => (false, 1, 1, 1),
        };

        let mut current_auth_zone_id = auth_zone_id;
        let mut rev_index = 0;
        let mut handles = Vec::new();
        let mut pass = false;
        loop {
            // Load auth zone
            let handle = api.kernel_lock_substate(
                &current_auth_zone_id,
                OBJECT_BASE_PARTITION,
                &AuthZoneField::AuthZone.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            let auth_zone: AuthZone = api.kernel_read_substate(handle)?.as_typed().unwrap();
            let auth_zone = auth_zone.clone();
            handles.push(handle);

            if skip > 0 {
                skip -= 1;
            } else {
                // Check
                if check(&auth_zone, rev_index, is_first_barrier, api)? {
                    pass = true;
                    break;
                }
                rev_index += 1;
            }

            // Progress
            is_first_barrier = false;
            if auth_zone.is_barrier {
                if remaining_barrier_crossings_allowed == 0 {
                    break;
                }
                remaining_barrier_crossings_allowed -= 1;

                if waiting_for_barrier > 0 {
                    waiting_for_barrier -= 1;
                    if waiting_for_barrier == 0u32 {
                        is_first_barrier = true;
                    }
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

    fn auth_zone_stack_has_amount<
        Y: KernelSubstateApi<SystemLockData> + ClientObjectApi<RuntimeError>,
    >(
        acting_location: ActingLocation,
        auth_zone_id: NodeId,
        resource: &ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(
            acting_location,
            auth_zone_id,
            api,
            |auth_zone, _, _, api| {
                // FIXME: Need to check the composite max amount rather than just each proof individually
                for p in auth_zone.proofs() {
                    if Self::proof_matches(&ResourceOrNonFungible::Resource(*resource), p, api)?
                        && p.sys_amount(api)? >= amount
                    {
                        return Ok(true);
                    }
                }

                Ok(false)
            },
        )
    }

    fn auth_zone_stack_matches_rule<
        Y: KernelSubstateApi<SystemLockData> + ClientObjectApi<RuntimeError>,
    >(
        acting_location: ActingLocation,
        auth_zone_id: NodeId,
        resource_rule: &ResourceOrNonFungible,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(
            acting_location,
            auth_zone_id,
            api,
            |auth_zone, rev_index, is_first_barrier, api| {
                if let ResourceOrNonFungible::NonFungible(non_fungible_global_id) = resource_rule {
                    if is_first_barrier {
                        if auth_zone
                            .virtual_non_fungibles_non_extending_barrier()
                            .contains(&non_fungible_global_id)
                        {
                            return Ok(true);
                        }
                    }

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

    pub fn verify_proof_rule<Y: KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>>(
        acting_location: ActingLocation,
        auth_zone_id: NodeId,
        proof_rule: &ProofRule,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match proof_rule {
            ProofRule::Require(resource) => {
                if Self::auth_zone_stack_matches_rule(acting_location, auth_zone_id, resource, api)?
                {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ProofRule::AmountOf(amount, resource) => {
                if Self::auth_zone_stack_has_amount(
                    acting_location,
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
            ProofRule::AllOf(resources) => {
                for resource in resources {
                    if !Self::auth_zone_stack_matches_rule(
                        acting_location,
                        auth_zone_id,
                        resource,
                        api,
                    )? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            ProofRule::AnyOf(resources) => {
                for resource in resources {
                    if Self::auth_zone_stack_matches_rule(
                        acting_location,
                        auth_zone_id,
                        resource,
                        api,
                    )? {
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            ProofRule::CountOf(count, resources) => {
                let mut left = count.clone();
                for resource in resources {
                    if Self::auth_zone_stack_matches_rule(
                        acting_location,
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
        }
    }

    pub fn verify_auth_rule<Y: KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>>(
        acting_location: ActingLocation,
        auth_zone_id: NodeId,
        access_rules: &AccessRulesConfig,
        auth_rule: &AccessRuleNode,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match auth_rule {
            AccessRuleNode::Authority(authority) => {
                match access_rules.authorities.get(authority.as_str()) {
                    Some(authority_entry) => {
                        match authority_entry {
                            AuthorityEntry::AccessRule(access_rule) => {
                                // TODO: Make sure we don't have circular entries!
                                Self::verify_method_auth(acting_location, auth_zone_id, access_rules, access_rule, api)
                            }
                        }
                    }
                    None => return Ok(false),
                }
            }
            AccessRuleNode::ProofRule(rule) => {
                Self::verify_proof_rule(acting_location, auth_zone_id, rule, api)
            }
            AccessRuleNode::AnyOf(rules) => {
                for r in rules {
                    if Self::verify_auth_rule(acting_location, auth_zone_id, access_rules, r, api)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            AccessRuleNode::AllOf(rules) => {
                for r in rules {
                    if !Self::verify_auth_rule(acting_location, auth_zone_id, access_rules, r, api)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
        }
    }

    pub fn verify_method_auth<Y: KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>>(
        acting_location: ActingLocation,
        auth_zone_id: NodeId,
        access_rules: &AccessRulesConfig,
        method_auth: &AccessRule,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match method_auth {
            AccessRule::Protected(rule) => {
                Self::verify_auth_rule(acting_location, auth_zone_id, access_rules, rule, api)
            }
            AccessRule::AllowAll => Ok(true),
            AccessRule::DenyAll => Ok(false),
        }
    }
}
