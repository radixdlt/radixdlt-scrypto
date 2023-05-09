use crate::model::Instruction;
use radix_engine_common::crypto::hash;
use radix_engine_common::data::manifest::*;
use radix_engine_common::data::scrypto::model::Reference;
use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;
use radix_engine_interface::blueprints::transaction_processor::RuntimeValidationRequest;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::traversal::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AuthZoneParams {
    pub initial_proofs: BTreeSet<NonFungibleGlobalId>,
    pub virtual_resources: BTreeSet<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ExecutionContext {
    pub transaction_hash: Hash,
    pub pre_allocated_ids: BTreeSet<NodeId>,
    pub payload_size: usize,
    pub auth_zone_params: AuthZoneParams,
    pub fee_payment: FeePayment,
    pub runtime_validations: Vec<RuntimeValidationRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum FeePayment {
    User {
        cost_unit_limit: u32,
        tip_percentage: u16,
    },
    NoFee,
}

/// Executable form of transaction, post stateless validation.
#[derive(Debug)]
pub struct Executable<'a> {
    blobs: BTreeMap<Hash, &'a Vec<u8>>,
    instructions: Vec<u8>,
    references: BTreeSet<Reference>,
    context: ExecutionContext,
}

impl<'a> Executable<'a> {
    pub fn new(
        blobs: &'a [Vec<u8>],
        instructions: &[Instruction],
        context: ExecutionContext,
    ) -> Self {
        let mut blobs_by_hash = BTreeMap::new();
        for b in blobs {
            blobs_by_hash.insert(hash(b), b);
        }
        let (encoded_instructions, references) = extract_references(instructions, &context);

        Self {
            blobs: blobs_by_hash,
            instructions: encoded_instructions,
            references,
            context,
        }
    }

    pub fn new_no_blobs(instructions: &[Instruction], context: ExecutionContext) -> Self {
        Self::new(&[], instructions, context)
    }

    pub fn transaction_hash(&self) -> &Hash {
        &self.context.transaction_hash
    }

    pub fn reset_transaction_hash(&mut self, hash: Hash) {
        self.context.transaction_hash = hash;
    }

    pub fn fee_payment(&self) -> &FeePayment {
        &self.context.fee_payment
    }

    pub fn blobs(&self) -> &BTreeMap<Hash, &'a Vec<u8>> {
        &self.blobs
    }

    pub fn instructions(&self) -> &Vec<u8> {
        &self.instructions
    }

    pub fn references(&self) -> &BTreeSet<Reference> {
        &self.references
    }

    pub fn auth_zone_params(&self) -> &AuthZoneParams {
        &self.context.auth_zone_params
    }

    pub fn pre_allocated_ids(&self) -> &BTreeSet<NodeId> {
        &self.context.pre_allocated_ids
    }

    pub fn payload_size(&self) -> usize {
        self.context.payload_size
    }

    pub fn runtime_validations(&self) -> &Vec<RuntimeValidationRequest> {
        &self.context.runtime_validations
    }
}

// TODO: we can potentially save manifest_encode by passing a slice of the raw transaction payload.
pub fn extract_references(
    instructions: &[Instruction],
    context: &ExecutionContext,
) -> (Vec<u8>, BTreeSet<Reference>) {
    let encoded = manifest_encode(instructions).unwrap();

    let mut references = BTreeSet::new();
    let mut traverser = ManifestTraverser::new(
        &encoded,
        MANIFEST_SBOR_V1_MAX_DEPTH,
        ExpectedStart::PayloadPrefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX),
        true,
    );
    loop {
        let event = traverser.next_event();
        match event.event {
            TraversalEvent::ContainerStart(_) => {}
            TraversalEvent::ContainerEnd(_) => {}
            TraversalEvent::TerminalValue(r) => {
                if let traversal::TerminalValueRef::Custom(c) = r {
                    match c.0 {
                        ManifestCustomValue::Address(address) => {
                            references.insert(Reference(address.0));
                        }
                        ManifestCustomValue::Bucket(_)
                        | ManifestCustomValue::Proof(_)
                        | ManifestCustomValue::Expression(_)
                        | ManifestCustomValue::Blob(_)
                        | ManifestCustomValue::Decimal(_)
                        | ManifestCustomValue::PreciseDecimal(_)
                        | ManifestCustomValue::NonFungibleLocalId(_) => {}
                    }
                }
            }
            TraversalEvent::TerminalValueBatch(_) => {}
            TraversalEvent::End => {
                break;
            }
            TraversalEvent::DecodeError(e) => {
                panic!("Unexpected decoding error: {:?}", e);
            }
        }
    }

    // TODO: how about pre-allocated IDs?

    for proof in &context.auth_zone_params.initial_proofs {
        references.insert(proof.resource_address().clone().into());
    }
    for resource in &context.auth_zone_params.virtual_resources {
        references.insert(resource.clone().into());
    }

    (encoded, references)
}
