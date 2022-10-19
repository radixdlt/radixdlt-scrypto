use crate::engine::{REActor, ResolvedReceiver, ResolvedReceiverMethod};
use crate::model::MethodAuthorizationError::NotAuthorized;
use crate::model::{
    AuthZoneError, HardAuthRule, HardCount, HardDecimal, HardProofRule, HardProofRuleResourceList,
    HardResourceOrNonFungible, InvokeError,
    MethodAuthorization, MethodAuthorizationError, ProofSubstate,
};
use crate::types::*;
use sbor::rust::ops::Fn;

struct AuthVerification;

impl AuthVerification {
    pub fn proof_matches(resource_rule: &HardResourceOrNonFungible, proof: &ProofSubstate) -> bool {
        match resource_rule {
            HardResourceOrNonFungible::NonFungible(non_fungible_address) => {
                let proof_resource_address = proof.resource_address();
                proof_resource_address == non_fungible_address.resource_address()
                    && match proof.total_ids() {
                        Ok(ids) => ids.contains(&non_fungible_address.non_fungible_id()),
                        Err(_) => false,
                    }
            }
            HardResourceOrNonFungible::Resource(resource_address) => {
                let proof_resource_address = proof.resource_address();
                proof_resource_address == *resource_address
            }
            HardResourceOrNonFungible::SoftResourceNotFound => false,
        }
    }

    pub fn check_auth_zones<P>(
        mut barriers_crossings_allowed: u32,
        auth_zones: &AuthZoneStackSubstate,
        check: P,
    ) -> bool
    where
        P: Fn(&AuthZone) -> bool,
    {
        for auth_zone in auth_zones.auth_zones.iter().rev() {
            if check(auth_zone) {
                return true;
            }

            if auth_zone.barrier {
                if barriers_crossings_allowed == 0 {
                    return false;
                }
                barriers_crossings_allowed -= 1;
            }
        }

        false
    }

    pub fn check_has_amount(
        barrier_crossings_allowed: u32,
        resource_rule: &HardResourceOrNonFungible,
        amount: Decimal,
        auth_zone: &AuthZoneStackSubstate,
    ) -> bool {
        Self::check_auth_zones(barrier_crossings_allowed, auth_zone, |auth_zone| {
            // FIXME: Need to check the composite max amount rather than just each proof individually
            auth_zone
                .proofs
                .iter()
                .any(|p| Self::proof_matches(resource_rule, p) && p.total_amount() >= amount)
        })
    }

    pub fn verify_resource_rule(
        barrier_crossings_allowed: u32,
        resource_rule: &HardResourceOrNonFungible,
        auth_zone: &AuthZoneStackSubstate,
    ) -> bool {
        Self::check_auth_zones(barrier_crossings_allowed, auth_zone, |auth_zone| {
            if let HardResourceOrNonFungible::NonFungible(non_fungible_address) = resource_rule {
                if auth_zone.virtual_resources.contains(&non_fungible_address.resource_address()) {
                    return true;
                }
            }

            if auth_zone
                .proofs
                .iter()
                .any(|p| Self::proof_matches(resource_rule, p))
            {
                return true;
            }

            false
        })
    }

    pub fn verify_proof_rule(
        barrier_crossings_allowed: u32,
        proof_rule: &HardProofRule,
        auth_zone: &AuthZoneStackSubstate,
    ) -> Result<(), MethodAuthorizationError> {
        match proof_rule {
            HardProofRule::Require(resource) => {
                if Self::verify_resource_rule(barrier_crossings_allowed, resource, auth_zone) {
                    Ok(())
                } else {
                    Err(NotAuthorized)
                }
            }
            HardProofRule::AmountOf(HardDecimal::Amount(amount), resource) => {
                if Self::check_has_amount(barrier_crossings_allowed, resource, *amount, auth_zone) {
                    Ok(())
                } else {
                    Err(NotAuthorized)
                }
            }
            HardProofRule::AllOf(HardProofRuleResourceList::List(resources)) => {
                for resource in resources {
                    if !Self::verify_resource_rule(barrier_crossings_allowed, resource, auth_zone) {
                        return Err(NotAuthorized);
                    }
                }

                Ok(())
            }
            HardProofRule::AnyOf(HardProofRuleResourceList::List(resources)) => {
                for resource in resources {
                    if Self::verify_resource_rule(barrier_crossings_allowed, resource, auth_zone) {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            }
            HardProofRule::CountOf(
                HardCount::Count(count),
                HardProofRuleResourceList::List(resources),
            ) => {
                let mut left = count.clone();
                for resource in resources {
                    if Self::verify_resource_rule(barrier_crossings_allowed, resource, auth_zone) {
                        left -= 1;
                        if left == 0 {
                            return Ok(());
                        }
                    }
                }
                Err(NotAuthorized)
            }
            _ => Err(NotAuthorized),
        }
    }

    pub fn verify_auth_rule(
        barrier_crossings_allowed: u32,
        auth_rule: &HardAuthRule,
        auth_zone: &AuthZoneStackSubstate,
    ) -> Result<(), MethodAuthorizationError> {
        match auth_rule {
            HardAuthRule::ProofRule(rule) => {
                Self::verify_proof_rule(barrier_crossings_allowed, rule, auth_zone)
            }
            HardAuthRule::AnyOf(rules) => {
                if !rules.iter().any(|r| {
                    Self::verify_auth_rule(barrier_crossings_allowed, r, auth_zone).is_ok()
                }) {
                    return Err(NotAuthorized);
                }
                Ok(())
            }
            HardAuthRule::AllOf(rules) => {
                if rules.iter().any(|r| {
                    Self::verify_auth_rule(barrier_crossings_allowed, r, auth_zone).is_err()
                }) {
                    return Err(NotAuthorized);
                }
                Ok(())
            }
        }
    }

    pub fn verify_method_auth(
        barrier_crossings_allowed: u32,
        method_auth: &MethodAuthorization,
        auth_zone: &AuthZoneStackSubstate,
    ) -> Result<(), MethodAuthorizationError> {
        match method_auth {
            MethodAuthorization::Protected(rule) => {
                Self::verify_auth_rule(barrier_crossings_allowed, rule, auth_zone)
            }
            MethodAuthorization::AllowAll => Ok(()),
            MethodAuthorization::DenyAll => Err(NotAuthorized),
            MethodAuthorization::Unsupported => Err(MethodAuthorizationError::UnsupportedMethod),
        }
    }
}

/// A transient resource container.
#[derive(Debug)]
pub struct AuthZoneStackSubstate {
    auth_zones: Vec<AuthZone>,
}

impl AuthZoneStackSubstate {
    pub fn new(
        proofs: Vec<ProofSubstate>,
        virtual_resources: BTreeSet<ResourceAddress>,
    ) -> Self {
        Self {
            auth_zones: vec![AuthZone::new_with_virtual_proofs(
                proofs,
                virtual_resources,
                false,
            )],
        }
    }

    fn is_barrier(actor: &REActor) -> bool {
        matches!(
            actor,
            REActor::Method(ResolvedReceiverMethod {
                receiver: ResolvedReceiver {
                    derefed_from: Some(RENodeId::Global(GlobalAddress::Component(..))),
                    ..
                },
                ..
            })
        )
    }

    pub fn check_auth(
        &self,
        to: &REActor,
        method_auths: Vec<MethodAuthorization>,
    ) -> Result<(), (MethodAuthorization, MethodAuthorizationError)> {
        let mut barrier_crossings_allowed = 1u32;
        if Self::is_barrier(to) {
            barrier_crossings_allowed -= 1;
        }

        for method_auth in method_auths {
            AuthVerification::verify_method_auth(barrier_crossings_allowed, &method_auth, &self)
                .map_err(|e| (method_auth, e))?;
        }

        Ok(())
    }

    pub fn new_frame(&mut self, actor: &REActor) {
        let barrier = Self::is_barrier(actor);
        let auth_zone = AuthZone::empty(barrier);
        self.auth_zones.push(auth_zone);
    }

    pub fn pop_frame(&mut self) {
        if let Some(mut auth_zone) = self.auth_zones.pop() {
            auth_zone.clear()
        }
    }

    pub fn clear_all(&mut self) {
        for auth_zone in &mut self.auth_zones {
            auth_zone.clear()
        }
    }

    pub fn cur_auth_zone_mut(&mut self) -> &mut AuthZone {
        self.auth_zones.last_mut().unwrap()
    }

    pub fn cur_auth_zone(&self) -> &AuthZone {
        self.auth_zones.last().unwrap()
    }
}

#[derive(Debug)]
pub struct AuthZone {
    proofs: Vec<ProofSubstate>,
    // Virtualized resources, note that one cannot create proofs with virtual resources but only be used for AuthZone checks
    virtual_resources: BTreeSet<ResourceAddress>,
    barrier: bool,
}

impl AuthZone {
    fn empty(barrier: bool) -> Self {
        Self {
            proofs: vec![],
            virtual_resources: BTreeSet::new(),
            barrier,
        }
    }

    fn new_with_virtual_proofs(
        proofs: Vec<ProofSubstate>,
        virtual_resources: BTreeSet<ResourceAddress>,
        barrier: bool,
    ) -> Self {
        Self {
            proofs,
            virtual_resources,
            barrier,
        }
    }

    pub fn pop(&mut self) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        if self.proofs.is_empty() {
            return Err(InvokeError::Error(AuthZoneError::EmptyAuthZone));
        }

        Ok(self.proofs.remove(self.proofs.len() - 1))
    }

    pub fn push(&mut self, proof: ProofSubstate) {
        self.proofs.push(proof);
    }

    pub fn drain(&mut self) -> Vec<ProofSubstate> {
        self.proofs.drain(0..).collect()
    }

    pub fn clear(&mut self) {
        loop {
            if let Some(proof) = self.proofs.pop() {
                proof.drop();
            } else {
                break;
            }
        }
    }

    pub fn create_proof(
        &self,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        ProofSubstate::compose(&self.proofs, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    pub fn create_proof_by_amount(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        ProofSubstate::compose_by_amount(&self.proofs, amount, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    pub fn create_proof_by_ids(
        &self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        let maybe_existing_proof =
            ProofSubstate::compose_by_ids(&self.proofs, ids, resource_address, resource_type)
                .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)));

        let proof = match maybe_existing_proof {
            Ok(proof) => proof,
            Err(e) => Err(e)?,
        };

        Ok(proof)
    }
}
