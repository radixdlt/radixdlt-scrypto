use super::{AuthZone, AuthZoneStackSubstate};
use crate::errors::RuntimeError;
use crate::system::kernel_modules::auth::*;
use crate::types::*;
use native_sdk::resource::SysProof;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::ops::Fn;

pub struct AuthVerification;

impl AuthVerification {
    fn proof_matches<Y: ClientObjectApi<RuntimeError>>(
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
        mut barriers_crossings_allowed: u32,
        auth_zones: &AuthZoneStackSubstate,
        api: &mut Y,
        check: P,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientObjectApi<RuntimeError>,
        P: Fn(&AuthZone, usize, &mut Y) -> Result<bool, RuntimeError>,
    {
        for (rev_index, auth_zone) in auth_zones.auth_zones.iter().rev().enumerate() {
            if check(auth_zone, rev_index, api)? {
                return Ok(true);
            }

            if auth_zone.barrier {
                if barriers_crossings_allowed == 0 {
                    return Ok(false);
                }
                barriers_crossings_allowed -= 1;
            }
        }

        Ok(false)
    }

    fn auth_zone_stack_has_amount<Y: ClientObjectApi<RuntimeError>>(
        barrier_crossings_allowed: u32,
        resource_rule: &HardResourceOrNonFungible,
        amount: Decimal,
        auth_zone: &AuthZoneStackSubstate,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(
            barrier_crossings_allowed,
            auth_zone,
            api,
            |auth_zone, _, api| {
                // FIXME: Need to check the composite max amount rather than just each proof individually
                for p in &auth_zone.proofs {
                    if Self::proof_matches(resource_rule, p, api)? && p.sys_amount(api)? >= amount {
                        return Ok(true);
                    }
                }

                Ok(false)
            },
        )
    }

    fn auth_zone_stack_matches_rule<Y: ClientObjectApi<RuntimeError>>(
        barrier_crossings_allowed: u32,
        resource_rule: &HardResourceOrNonFungible,
        auth_zone: &AuthZoneStackSubstate,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(
            barrier_crossings_allowed,
            auth_zone,
            api,
            |auth_zone, rev_index, api| {
                if let HardResourceOrNonFungible::NonFungible(non_fungible_global_id) =
                    resource_rule
                {
                    if rev_index == 0 {
                        if auth_zone
                            .virtual_non_fungibles_non_extending
                            .contains(&non_fungible_global_id)
                        {
                            return Ok(true);
                        }
                    }

                    if auth_zone
                        .virtual_non_fungibles
                        .contains(&non_fungible_global_id)
                    {
                        return Ok(true);
                    }
                    if auth_zone
                        .virtual_resource_addresses
                        .contains(&non_fungible_global_id.resource_address())
                    {
                        return Ok(true);
                    }
                }

                for p in &auth_zone.proofs {
                    if Self::proof_matches(resource_rule, p, api)? {
                        return Ok(true);
                    }
                }

                Ok(false)
            },
        )
    }

    pub fn verify_proof_rule<Y: ClientObjectApi<RuntimeError>>(
        barrier_crossings_allowed: u32,
        proof_rule: &HardProofRule,
        auth_zone: &AuthZoneStackSubstate,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match proof_rule {
            HardProofRule::Require(resource) => {
                if Self::auth_zone_stack_matches_rule(
                    barrier_crossings_allowed,
                    resource,
                    auth_zone,
                    api,
                )? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            HardProofRule::AmountOf(HardDecimal::Amount(amount), resource) => {
                if Self::auth_zone_stack_has_amount(
                    barrier_crossings_allowed,
                    resource,
                    *amount,
                    auth_zone,
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
                        barrier_crossings_allowed,
                        resource,
                        auth_zone,
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
                        barrier_crossings_allowed,
                        resource,
                        auth_zone,
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
                        barrier_crossings_allowed,
                        resource,
                        auth_zone,
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

    pub fn verify_auth_rule<Y: ClientObjectApi<RuntimeError>>(
        barrier_crossings_allowed: u32,
        auth_rule: &HardAuthRule,
        auth_zone: &AuthZoneStackSubstate,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match auth_rule {
            HardAuthRule::ProofRule(rule) => {
                Self::verify_proof_rule(barrier_crossings_allowed, rule, auth_zone, api)
            }
            HardAuthRule::AnyOf(rules) => {
                for r in rules {
                    if Self::verify_auth_rule(barrier_crossings_allowed, r, auth_zone, api)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            HardAuthRule::AllOf(rules) => {
                for r in rules {
                    if !Self::verify_auth_rule(barrier_crossings_allowed, r, auth_zone, api)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
        }
    }

    pub fn verify_method_auth<Y: ClientObjectApi<RuntimeError>>(
        barrier_crossings_allowed: u32,
        method_auth: &MethodAuthorization,
        auth_zone: &AuthZoneStackSubstate,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match method_auth {
            MethodAuthorization::Protected(rule) => {
                Self::verify_auth_rule(barrier_crossings_allowed, rule, auth_zone, api)
            }
            MethodAuthorization::AllowAll => Ok(true),
            MethodAuthorization::DenyAll => Ok(false),
        }
    }
}
