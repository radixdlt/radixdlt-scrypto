use crate::types::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, ScryptoSbor, Default)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,

    // Virtualized resources, note that one cannot create proofs with virtual resources but only be used for AuthZone checks
    pub virtual_resources: BTreeSet<ResourceAddress>,
    pub virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,

    pub local_caller_package_address: Option<PackageAddress>,
    pub global_caller: Option<(GlobalCaller, Reference)>,

    pub parent: Option<Reference>,
}

impl Clone for AuthZone {
    fn clone(&self) -> Self {
        Self {
            proofs: self.proofs.iter().map(|p| Proof(p.0)).collect(),
            virtual_resources: self.virtual_resources.clone(),
            virtual_non_fungibles: self.virtual_non_fungibles.clone(),
            local_caller_package_address: self.local_caller_package_address.clone(),
            global_caller: self.global_caller.clone(),
            parent: self.parent.clone(),
        }
    }
}

impl AuthZone {
    pub fn new(
        proofs: Vec<Proof>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
        local_caller_package_address: Option<PackageAddress>,
        global_caller: Option<(GlobalCaller, Reference)>,
        parent: Option<Reference>,
    ) -> Self {
        Self {
            proofs,
            virtual_resources,
            virtual_non_fungibles,
            local_caller_package_address,
            global_caller,
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

    pub fn local_virtual_non_fungibles(&self) -> BTreeSet<NonFungibleGlobalId> {
        let mut virtual_proofs = BTreeSet::new();

        // Local Caller package address
        if let Some(local_package_address) = self.local_caller_package_address {
            let non_fungible_global_id =
                NonFungibleGlobalId::package_of_direct_caller_badge(local_package_address);
            virtual_proofs.insert(non_fungible_global_id);
        }

        // Global Caller Actor
        if let Some((global_caller, _global_caller_reference)) = &self.global_caller {
            let non_fungible_global_id =
                NonFungibleGlobalId::global_caller_badge(global_caller.clone());
            virtual_proofs.insert(non_fungible_global_id);
        }

        virtual_proofs
    }

    pub fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn pop(&mut self) -> Option<Proof> {
        self.proofs.pop()
    }

    pub fn remove_signature_proofs(&mut self) {
        self.virtual_resources.retain(|x| {
            x != &SECP256K1_SIGNATURE_VIRTUAL_BADGE && x != &ED25519_SIGNATURE_VIRTUAL_BADGE
        });
        self.virtual_non_fungibles.retain(|x| {
            x.resource_address() != SECP256K1_SIGNATURE_VIRTUAL_BADGE
                && x.resource_address() != ED25519_SIGNATURE_VIRTUAL_BADGE
        });
    }

    pub fn remove_regular_proofs(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
    }
}
