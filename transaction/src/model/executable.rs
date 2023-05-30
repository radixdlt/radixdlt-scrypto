use crate::internal_prelude::*;
use radix_engine_interface::blueprints::transaction_processor::RuntimeValidationRequest;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AuthZoneParams {
    pub initial_proofs: BTreeSet<NonFungibleGlobalId>,
    pub virtual_resources: BTreeSet<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ExecutionContext {
    pub transaction_hash: Hash,
    pub pre_allocated_addresses: Vec<(BlueprintId, GlobalAddress)>,
    pub payload_size: usize,
    pub auth_zone_params: AuthZoneParams,
    pub fee_payment: FeePayment,
    pub runtime_validations: Vec<RuntimeValidationRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct FeePayment {
    pub tip_percentage: u16,
    /// Free credit for execution, for preview only!
    /// It's the `u128` representation of Decimal, see `transmute_decimal_as_u128`.
    pub free_credit_in_xrd: u128,
}

/// Executable form of transaction, post stateless validation.
#[derive(Debug)]
pub struct Executable<'a> {
    encoded_instructions: &'a [u8],
    references: IndexSet<Reference>,
    blobs: &'a IndexMap<Hash, Vec<u8>>,
    context: ExecutionContext,
}

impl<'a> Executable<'a> {
    pub fn new(
        encoded_instructions: &'a [u8],
        references: &IndexSet<Reference>,
        blobs: &'a IndexMap<Hash, Vec<u8>>,
        context: ExecutionContext,
    ) -> Self {
        let mut references = references.clone();

        for proof in &context.auth_zone_params.initial_proofs {
            references.insert(proof.resource_address().clone().into());
        }
        for resource in &context.auth_zone_params.virtual_resources {
            references.insert(resource.clone().into());
        }

        Self {
            encoded_instructions,
            references,
            blobs,
            context,
        }
    }

    pub fn transaction_hash(&self) -> &Hash {
        &self.context.transaction_hash
    }

    pub fn overwrite_transaction_hash(&mut self, hash: Hash) {
        self.context.transaction_hash = hash;
    }

    pub fn fee_payment(&self) -> &FeePayment {
        &self.context.fee_payment
    }

    pub fn blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        &self.blobs
    }

    pub fn encoded_instructions(&self) -> &[u8] {
        &self.encoded_instructions
    }

    pub fn references(&self) -> &IndexSet<Reference> {
        &self.references
    }

    pub fn auth_zone_params(&self) -> &AuthZoneParams {
        &self.context.auth_zone_params
    }

    pub fn pre_allocated_addresses(&self) -> &Vec<(BlueprintId, GlobalAddress)> {
        &self.context.pre_allocated_addresses
    }

    pub fn payload_size(&self) -> usize {
        self.context.payload_size
    }

    pub fn runtime_validations(&self) -> &Vec<RuntimeValidationRequest> {
        &self.context.runtime_validations
    }
}
