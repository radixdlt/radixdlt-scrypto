use crate::blueprints::resource::AuthZone;
use crate::errors::RuntimeError;
use crate::kernel::actor::AuthActorInfo;
use crate::kernel::kernel_api::{KernelApi, KernelSubstateApi};
use crate::system::node_modules::role_assignment::OwnerRoleSubstate;
use crate::system::system::{FieldSubstate, KeyValueEntrySubstate, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::{
    AuthorityListAuthorizationResult, AuthorizationCheckResult,
};
use crate::types::*;
use native_sdk::resource::{NativeNonFungibleProof, NativeProof};
use radix_engine_interface::api::{LockFlags, ObjectModuleId};
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::ops::Fn;

pub struct Authorization;

impl Authorization {
    fn proof_matches<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        resource_rule: &ResourceOrNonFungible,
        proof: &Proof,
        api: &mut SystemService<Y, V>,
    ) -> Result<bool, RuntimeError> {
        match resource_rule {
            ResourceOrNonFungible::NonFungible(non_fungible_global_id) => {
                let proof_resource_address = proof.resource_address(api)?;
                Ok(
                    proof_resource_address == non_fungible_global_id.resource_address()
                        && proof
                            .non_fungible_local_ids(api)?
                            .contains(non_fungible_global_id.local_id()),
                )
            }
            ResourceOrNonFungible::Resource(resource_address) => {
                let proof_resource_address = proof.resource_address(api)?;
                Ok(proof_resource_address == *resource_address)
            }
        }
    }

    fn global_auth_zone_matches<Y, V, P>(
        api: &mut SystemService<Y, V>,
        auth_zone_id: &NodeId,
        check: &P,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
        P: Fn(
            &[Proof],
            &BTreeSet<ResourceAddress>,
            BTreeSet<&NonFungibleGlobalId>,
            &mut SystemService<Y, V>,
        ) -> Result<bool, RuntimeError>,
    {
        let mut pass = false;
        let mut current_auth_zone_id = *auth_zone_id;
        let mut handles = Vec::new();
        loop {
            // Load auth zone
            let handle = api.kernel_open_substate(
                &current_auth_zone_id,
                MAIN_BASE_PARTITION,
                &AuthZoneField::AuthZone.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            let auth_zone: FieldSubstate<AuthZone> =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let auth_zone = auth_zone.value.0.clone();
            handles.push(handle);

            {
                let mut virtual_non_fungible_global_ids = BTreeSet::new();
                let virtual_resources = auth_zone.virtual_resources();

                virtual_non_fungible_global_ids.extend(auth_zone.virtual_non_fungibles());

                let proofs = auth_zone.proofs();

                // Check
                if check(
                    proofs,
                    virtual_resources,
                    virtual_non_fungible_global_ids,
                    api,
                )? {
                    pass = true;
                    break;
                }
            }

            if let Some(id) = auth_zone.parent {
                current_auth_zone_id = id.into();
            } else {
                break;
            }
        }

        for handle in handles {
            api.kernel_close_substate(handle)?;
        }

        Ok(pass)
    }

    fn auth_zone_stack_matches<P, Y, V>(
        auth_zone: &NodeId,
        api: &mut SystemService<Y, V>,
        check: P,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
        P: Fn(
            &[Proof],
            &BTreeSet<ResourceAddress>,
            BTreeSet<&NonFungibleGlobalId>,
            &mut SystemService<Y, V>,
        ) -> Result<bool, RuntimeError>,
    {
        {
            let handle = api.kernel_open_substate(
                &auth_zone,
                MAIN_BASE_PARTITION,
                &AuthZoneField::AuthZone.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            let auth_zone: FieldSubstate<AuthZone> = api.kernel_read_substate(handle)?.as_typed().unwrap();

            // TODO: Combine these two

            // Test Local Caller package address
            if let Some(local_package_address) = auth_zone.value.0.local_caller_package_address {
                let non_fungible_global_id = NonFungibleGlobalId::package_of_direct_caller_badge(
                    local_package_address,
                );
                let local_call_frame_proofs = btreeset!(&non_fungible_global_id);
                if check(&[], &btreeset!(), local_call_frame_proofs, api)? {
                    api.kernel_close_substate(handle)?;
                    return Ok(true);
                }
            }

            // Test Global Caller
            if let Some((global_caller, global_caller_reference)) = &auth_zone.value.0.global_caller {
                let non_fungible_global_id =
                    NonFungibleGlobalId::global_caller_badge(global_caller.clone());
                let global_call_frame_proofs = btreeset!(&non_fungible_global_id);
                if check(&[], &btreeset!(), global_call_frame_proofs, api)? {
                    api.kernel_close_substate(handle)?;
                    return Ok(true);
                }

                if Self::global_auth_zone_matches(api, &global_caller_reference.0, &check)? {
                    api.kernel_close_substate(handle)?;
                    return Ok(true);
                }
            }

            api.kernel_close_substate(handle)?;
        }

        if Self::global_auth_zone_matches(api, auth_zone, &check)? {
            return Ok(true);
        }

        Ok(false)
    }

    fn auth_zone_stack_has_amount<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone: &NodeId,
        resource: &ResourceAddress,
        amount: Decimal,
        api: &mut SystemService<Y, V>,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(auth_zone, api, |proofs, _, _, api| {
            // TODO: revisit this and decide if we need to check the composite max amount rather than just each proof individually
            for p in proofs {
                if Self::proof_matches(&ResourceOrNonFungible::Resource(*resource), p, api)?
                    && p.amount(api)? >= amount
                {
                    return Ok(true);
                }
            }

            Ok(false)
        })
    }

    fn auth_zone_stack_matches_rule<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone: &NodeId,
        resource_rule: &ResourceOrNonFungible,
        api: &mut SystemService<Y, V>,
    ) -> Result<bool, RuntimeError> {
        Self::auth_zone_stack_matches(
            auth_zone,
            api,
            |proofs, virtual_resources, virtual_non_fungibles, api| {
                if let ResourceOrNonFungible::NonFungible(non_fungible_global_id) = resource_rule {
                    if virtual_non_fungibles.contains(non_fungible_global_id) {
                        return Ok(true);
                    }

                    if virtual_resources.contains(&non_fungible_global_id.resource_address()) {
                        return Ok(true);
                    }
                }

                for p in proofs {
                    if Self::proof_matches(resource_rule, p, api)? {
                        return Ok(true);
                    }
                }

                Ok(false)
            },
        )
    }

    pub fn verify_proof_rule<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone: &NodeId,
        proof_rule: &ProofRule,
        api: &mut SystemService<Y, V>,
    ) -> Result<bool, RuntimeError> {
        match proof_rule {
            ProofRule::Require(resource) => {
                if Self::auth_zone_stack_matches_rule(auth_zone, resource, api)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ProofRule::AmountOf(amount, resource) => {
                if Self::auth_zone_stack_has_amount(auth_zone, resource, *amount, api)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ProofRule::AllOf(resources) => {
                for resource in resources {
                    if !Self::auth_zone_stack_matches_rule(auth_zone, resource, api)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            ProofRule::AnyOf(resources) => {
                for resource in resources {
                    if Self::auth_zone_stack_matches_rule(auth_zone, resource, api)? {
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            ProofRule::CountOf(count, resources) => {
                let mut left = count.clone();
                for resource in resources {
                    if Self::auth_zone_stack_matches_rule(auth_zone, resource, api)? {
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

    pub fn verify_auth_rule<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone: &NodeId,
        auth_rule: &AccessRuleNode,
        api: &mut SystemService<Y, V>,
    ) -> Result<AuthorizationCheckResult, RuntimeError> {
        match auth_rule {
            AccessRuleNode::ProofRule(rule) => {
                if Self::verify_proof_rule(auth_zone, rule, api)? {
                    Ok(AuthorizationCheckResult::Authorized)
                } else {
                    Ok(AuthorizationCheckResult::Failed(vec![]))
                }
            }
            AccessRuleNode::AnyOf(rules) => {
                for r in rules {
                    let rtn = Self::verify_auth_rule(auth_zone, r, api)?;
                    if matches!(rtn, AuthorizationCheckResult::Authorized) {
                        return Ok(rtn);
                    }
                }
                Ok(AuthorizationCheckResult::Failed(vec![]))
            }
            AccessRuleNode::AllOf(rules) => {
                for r in rules {
                    let rtn = Self::verify_auth_rule(auth_zone, r, api)?;
                    if matches!(rtn, AuthorizationCheckResult::Failed(..)) {
                        return Ok(rtn);
                    }
                }

                return Ok(AuthorizationCheckResult::Authorized);
            }
        }
    }

    pub fn check_authorization_against_role_key_internal<
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    >(
        auth_zone: &NodeId,
        role_assignment_of: &NodeId,
        key: &ModuleRoleKey,
        api: &mut SystemService<Y, V>,
    ) -> Result<AuthorizationCheckResult, RuntimeError> {
        let access_rule = if key.key.key.eq(SELF_ROLE) {
            // FIXME: Prevent panics of node id, this may be triggered by vaults and auth zone
            rule!(require(global_caller(GlobalAddress::new_or_panic(
                role_assignment_of.0
            ))))
        } else {
            let handle = api.kernel_open_substate_with_default(
                role_assignment_of,
                ROLE_ASSIGNMENT_BASE_PARTITION
                    .at_offset(ROLE_ASSIGNMENT_ROLE_DEF_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&key).unwrap()),
                LockFlags::read_only(),
                Some(|| {
                    let kv_entry = KeyValueEntrySubstate::<()>::default();
                    IndexedScryptoValue::from_typed(&kv_entry)
                }),
                SystemLockData::default(),
            )?;
            let substate: KeyValueEntrySubstate<AccessRule> =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            api.kernel_close_substate(handle)?;

            match substate.value {
                Some(access_rule) => access_rule,
                None => {
                    let handle = api.kernel_open_substate(
                        role_assignment_of,
                        ROLE_ASSIGNMENT_BASE_PARTITION
                            .at_offset(ROLE_ASSIGNMENT_FIELDS_PARTITION_OFFSET)
                            .unwrap(),
                        &SubstateKey::Field(0u8),
                        LockFlags::read_only(),
                        SystemLockData::default(),
                    )?;

                    let owner_role_substate: FieldSubstate<OwnerRoleSubstate> =
                        api.kernel_read_substate(handle)?.as_typed().unwrap();
                    api.kernel_close_substate(handle)?;
                    owner_role_substate.value.0.owner_role_entry.rule
                }
            }
        };

        Self::check_authorization_against_access_rule(api, auth_zone, &access_rule)
    }

    pub fn check_authorization_against_access_rule<
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    >(
        api: &mut SystemService<Y, V>,
        auth_zone: &NodeId,
        rule: &AccessRule,
    ) -> Result<AuthorizationCheckResult, RuntimeError> {
        match rule {
            AccessRule::Protected(rule_node) => {
                let mut rtn = Self::verify_auth_rule(auth_zone, rule_node, api)?;
                match &mut rtn {
                    AuthorizationCheckResult::Authorized => {}
                    AuthorizationCheckResult::Failed(stack) => {
                        stack.push(rule.clone());
                    }
                }
                Ok(rtn)
            }
            AccessRule::AllowAll => Ok(AuthorizationCheckResult::Authorized),
            AccessRule::DenyAll => Ok(AuthorizationCheckResult::Failed(vec![rule.clone()])),
        }
    }

    pub fn check_authorization_against_role_list<
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    >(
        auth_zone: &NodeId,
        role_assignment_of: &NodeId,
        module: ObjectModuleId,
        role_list: &RoleList,
        api: &mut SystemService<Y, V>,
    ) -> Result<AuthorityListAuthorizationResult, RuntimeError> {
        let mut failed = Vec::new();

        for key in &role_list.list {
            let module_role_key = ModuleRoleKey::new(module, key.key.as_str());
            let result = Self::check_authorization_against_role_key_internal(
                &auth_zone,
                role_assignment_of,
                &module_role_key,
                api,
            )?;
            match result {
                AuthorizationCheckResult::Authorized => {
                    return Ok(AuthorityListAuthorizationResult::Authorized)
                }
                AuthorizationCheckResult::Failed(stack) => {
                    failed.push((key.clone(), stack));
                }
            }
        }

        Ok(AuthorityListAuthorizationResult::Failed(failed))
    }
}
