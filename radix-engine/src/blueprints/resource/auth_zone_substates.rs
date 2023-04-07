use crate::types::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, ScryptoSbor, Default)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,

    // Virtualized resources, note that one cannot create proofs with virtual resources but only be used for AuthZone checks
    pub virtual_resources: BTreeSet<ResourceAddress>,
    pub virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    pub virtual_non_fungibles_non_extending: BTreeSet<NonFungibleGlobalId>,

    pub is_barrier: bool,
    pub parent: Option<Reference>,
}

impl Clone for AuthZone {
    fn clone(&self) -> Self {
        Self {
            proofs: self.proofs.iter().map(|p| Proof(p.0)).collect(),
            virtual_resources: self.virtual_resources.clone(),
            virtual_non_fungibles: self.virtual_non_fungibles.clone(),
            virtual_non_fungibles_non_extending: self.virtual_non_fungibles_non_extending.clone(),
            is_barrier: self.is_barrier,
            parent: self.parent.clone(),
        }
    }
}

impl AuthZone {
    pub fn new(
        proofs: Vec<Proof>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
        virtual_non_fungibles_non_extending: BTreeSet<NonFungibleGlobalId>,
        is_barrier: bool,
        parent: Option<Reference>,
    ) -> Self {
        Self {
            proofs,
            virtual_resources,
            virtual_non_fungibles,
            virtual_non_fungibles_non_extending,
            is_barrier,
            parent,
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

    pub fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn pop(&mut self) -> Option<Proof> {
        self.proofs.pop()
    }

    pub fn drain(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
    }

    pub fn clear_signature_proofs(&mut self) {
        self.virtual_resources
            .retain(|x| x != &ECDSA_SECP256K1_TOKEN && x != &EDDSA_ED25519_TOKEN);
        self.virtual_non_fungibles.retain(|x| {
            x.resource_address() != ECDSA_SECP256K1_TOKEN
                && x.resource_address() != EDDSA_ED25519_TOKEN
        });
        self.virtual_non_fungibles_non_extending.retain(|x| {
            x.resource_address() != ECDSA_SECP256K1_TOKEN
                && x.resource_address() != EDDSA_ED25519_TOKEN
        });
    }
}
