use crate::types::*;
use radix_engine_interface::blueprints::resource::*;

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

    pub fn proofs(&self) -> &[Proof] {
        &self.proofs
    }

    pub fn virtual_resources(&self) -> &BTreeSet<ResourceAddress> {
        &self.virtual_resources
    }

    pub fn virtual_non_fungibles(&self) -> &BTreeSet<NonFungibleGlobalId> {
        &self.virtual_non_fungibles
    }

    pub fn virtual_non_fungibles_non_extending(&self) -> &BTreeSet<NonFungibleGlobalId> {
        &self.virtual_non_fungibles_non_extending
    }

    pub fn is_barrier(&self) -> bool {
        self.barrier
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
