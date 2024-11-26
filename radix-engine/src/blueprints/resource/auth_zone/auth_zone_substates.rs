use crate::internal_prelude::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, ScryptoSbor, Default)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,

    pub simulate_all_proofs_under_resources: BTreeSet<ResourceAddress>,
    pub implicit_non_fungible_proofs: BTreeSet<NonFungibleGlobalId>,

    pub direct_caller_package_address: Option<PackageAddress>,
    pub global_caller: Option<(GlobalCaller, Reference)>,

    pub parent: Option<Reference>,
}

#[derive(ScryptoSbor)]
#[sbor(type_name = "AuthZone")]
/// This is just the same as `AuthZone`, but with old field names.
/// This allows us to have a fixed genesis schema for the resource package.
pub struct GenesisSchemaAuthZone {
    pub proofs: Vec<Proof>,
    pub virtual_resources: BTreeSet<ResourceAddress>,
    pub virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    pub local_caller_package_address: Option<PackageAddress>,
    pub global_caller: Option<(GlobalCaller, Reference)>,
    pub parent: Option<Reference>,
}

impl AuthZone {
    pub fn new(
        proofs: Vec<Proof>,
        simulate_all_proofs_under_resources: BTreeSet<ResourceAddress>,
        implicit_non_fungible_proofs: BTreeSet<NonFungibleGlobalId>,
        direct_caller_package_address: Option<PackageAddress>,
        global_caller: Option<(GlobalCaller, Reference)>,
        parent: Option<Reference>,
    ) -> Self {
        Self {
            proofs,
            simulate_all_proofs_under_resources,
            implicit_non_fungible_proofs,
            direct_caller_package_address,
            global_caller,
            parent,
        }
    }

    pub fn proofs(&self) -> &[Proof] {
        &self.proofs
    }

    pub fn simulate_all_proofs_under_resources(&self) -> &BTreeSet<ResourceAddress> {
        &self.simulate_all_proofs_under_resources
    }

    pub fn implicit_non_fungible_proofs(&self) -> &BTreeSet<NonFungibleGlobalId> {
        &self.implicit_non_fungible_proofs
    }

    pub fn local_implicit_non_fungible_proofs(&self) -> BTreeSet<NonFungibleGlobalId> {
        let mut local_implicit_non_fungible_proofs = BTreeSet::new();

        // Local Caller package address
        if let Some(local_package_address) = self.direct_caller_package_address {
            let non_fungible_global_id =
                NonFungibleGlobalId::package_of_direct_caller_badge(local_package_address);
            local_implicit_non_fungible_proofs.insert(non_fungible_global_id);
        }

        // Global Caller
        if let Some((global_caller, _global_caller_reference)) = &self.global_caller {
            if !global_caller.is_actually_frame_owned() {
                let non_fungible_global_id =
                    NonFungibleGlobalId::global_caller_badge(global_caller.clone());
                local_implicit_non_fungible_proofs.insert(non_fungible_global_id);
            }
        }

        local_implicit_non_fungible_proofs
    }

    pub fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn pop(&mut self) -> Option<Proof> {
        self.proofs.pop()
    }

    pub fn remove_signature_proofs(&mut self) {
        self.simulate_all_proofs_under_resources
            .retain(|x| x != &SECP256K1_SIGNATURE_RESOURCE && x != &ED25519_SIGNATURE_RESOURCE);
        self.implicit_non_fungible_proofs.retain(|x| {
            x.resource_address() != SECP256K1_SIGNATURE_RESOURCE
                && x.resource_address() != ED25519_SIGNATURE_RESOURCE
        });
    }

    pub fn remove_regular_proofs(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
    }
}
