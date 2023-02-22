use super::{AuthVerification, AuthZoneError};
use crate::errors::{InvokeError, ModuleError, RuntimeError};
use crate::system::kernel_modules::auth::*;
use crate::types::*;
use radix_engine_interface::api::ClientComponentApi;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug)]
pub struct AuthZoneStackSubstate {
    pub(super) auth_zones: Vec<AuthZone>,
}

impl AuthZoneStackSubstate {
    pub fn new(
        proofs: Vec<Proof>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    ) -> Self {
        Self {
            auth_zones: vec![AuthZone::new_with_virtual_proofs(
                proofs,
                virtual_resources,
                virtual_non_fungibles,
                false,
            )],
        }
    }

    pub fn check_auth<Y: ClientComponentApi<RuntimeError>>(
        &self,
        is_barrier: bool,
        method_auths: Vec<MethodAuthorization>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let mut barrier_crossings_allowed = 1u32;
        if is_barrier {
            barrier_crossings_allowed -= 1;
        }

        for method_auth in method_auths {
            if !AuthVerification::verify_method_auth(
                barrier_crossings_allowed,
                &method_auth,
                &self,
                api,
            )? {
                return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                    AuthError::Unauthorized {
                        authorization: method_auth,
                        error: MethodAuthorizationError::NotAuthorized,
                    },
                )));
            }
        }

        Ok(())
    }

    pub fn push_auth_zone(
        &mut self,
        virtual_non_fungibles_non_extending: BTreeSet<NonFungibleGlobalId>,
        barrier: bool,
    ) {
        let auth_zone =
            AuthZone::new_with_virtual_non_fungibles(virtual_non_fungibles_non_extending, barrier);
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
}

#[derive(Debug)]
pub struct AuthZone {
    pub(super) proofs: Vec<Proof>,
    // Virtualized resources, note that one cannot create proofs with virtual resources but only be used for AuthZone checks
    pub(super) virtual_resources: BTreeSet<ResourceAddress>,
    pub(super) virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    pub(super) virtual_non_fungibles_non_extending: BTreeSet<NonFungibleGlobalId>,
    pub(super) barrier: bool,
}

impl AuthZone {
    fn new_with_virtual_non_fungibles(
        virtual_non_fungibles_non_extending: BTreeSet<NonFungibleGlobalId>,
        barrier: bool,
    ) -> Self {
        Self {
            proofs: vec![],
            virtual_resources: BTreeSet::new(),
            virtual_non_fungibles: BTreeSet::new(),
            virtual_non_fungibles_non_extending,
            barrier,
        }
    }

    fn new_with_virtual_proofs(
        proofs: Vec<Proof>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
        barrier: bool,
    ) -> Self {
        Self {
            proofs,
            virtual_resources,
            virtual_non_fungibles,
            virtual_non_fungibles_non_extending: BTreeSet::new(),
            barrier,
        }
    }

    pub fn pop(&mut self) -> Result<Proof, InvokeError<AuthZoneError>> {
        if self.proofs.is_empty() {
            return Err(InvokeError::SelfError(AuthZoneError::EmptyAuthZone));
        }

        Ok(self.proofs.remove(self.proofs.len() - 1))
    }

    pub fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn drain(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
    }
}
