use super::AuthVerification;
use crate::errors::RuntimeError;
use crate::system::kernel_modules::auth::*;
use crate::types::*;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct AuthZoneStackSubstate {
    pub(super) auth_zones: Vec<AuthZone>,
}

impl AuthZoneStackSubstate {
    pub fn new() -> Self {
        Self { auth_zones: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.auth_zones.is_empty()
    }

    pub fn check_auth<Y: ClientObjectApi<RuntimeError>>(
        &self,
        is_barrier: bool,
        method_auth: &MethodAuthorization,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        AuthVerification::verify_method_auth(
            if is_barrier { 0 } else { 1 },
            method_auth,
            &self,
            api,
        )
    }

    pub fn push_auth_zone(&mut self, auth_zone: AuthZone) {
        self.auth_zones.push(auth_zone);
    }

    pub fn pop_auth_zone(&mut self) -> Option<AuthZone> {
        self.auth_zones.pop()
    }

    pub fn cur_auth_zone_mut(&mut self) -> &mut AuthZone {
        self.auth_zones.last_mut().unwrap()
    }

    pub fn cur_auth_zone(&self) -> &AuthZone {
        self.auth_zones.last().unwrap()
    }

    pub fn all_proofs(&self) -> Vec<Proof> {
        let mut proofs = Vec::new();
        for auth_zone in &self.auth_zones {
            for p in &auth_zone.proofs {
                proofs.push(Proof(p.0));
            }
        }
        proofs
    }
}

#[derive(Debug, ScryptoSbor)]
pub struct AuthZone {
    pub(super) proofs: Vec<Proof>,
    // Virtualized resources, note that one cannot create proofs with virtual resources but only be used for AuthZone checks
    pub(super) virtual_resources: BTreeSet<ResourceAddress>,
    pub(super) virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    pub(super) virtual_non_fungibles_non_extending: BTreeSet<NonFungibleGlobalId>,
    pub(super) barrier: bool,
}

impl Clone for AuthZone {
    fn clone(&self) -> Self {
        Self {
            proofs: self.proofs.iter().map(|p| Proof(p.0)).collect(),
            virtual_resources: self.virtual_resources.clone(),
            virtual_non_fungibles: self.virtual_non_fungibles.clone(),
            virtual_non_fungibles_non_extending: self.virtual_non_fungibles_non_extending.clone(),
            barrier: self.barrier.clone(),
        }
    }
}

impl AuthZone {
    pub fn new(
        proofs: Vec<Proof>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
        virtual_non_fungibles_non_extending: BTreeSet<NonFungibleGlobalId>,
        barrier: bool,
    ) -> Self {
        Self {
            proofs,
            virtual_resources,
            virtual_non_fungibles,
            virtual_non_fungibles_non_extending,
            barrier,
        }
    }

    pub fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn pop(&mut self) -> Option<Proof> {
        self.proofs.pop()
    }

    pub fn drain(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
    }

    pub fn clear_virtual_proofs(&mut self) {
        self.virtual_resources.clear();
        self.virtual_non_fungibles.clear();
        self.virtual_non_fungibles_non_extending.clear();
    }
}
