use sbor::path::{SborPath, SborPathBuf};
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::api::types::*;
use crate::data::types::*;
use crate::data::*;
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ScryptoValueDecodeError {
    RawValueEncodeError(EncodeError),
    TypedValueEncodeError(EncodeError),
    DecodeError(DecodeError),
    ValueIndexingError(ValueIndexingError),
}

pub enum ValueReplacingError {
    ProofIdNotFound(ManifestProof),
    BucketIdNotFound(ManifestBucket),
}

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    pub raw: Vec<u8>,
    pub dom: ScryptoValue,

    // Global addresses
    pub component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub package_addresses: HashSet<PackageAddress>,
    pub system_addresses: HashSet<SystemAddress>,

    // RE interpreted
    pub ownerships: HashSet<Own>,
    pub kv_store_ids: HashSet<KeyValueStoreId>,
    pub component_ids: HashSet<ComponentId>,
    pub non_fungible_addresses: HashSet<NonFungibleAddress>,
    pub blobs: Vec<(Blob, SborPath)>,

    // TX interpreted
    pub buckets: HashMap<ManifestBucket, SborPath>,
    pub proofs: HashMap<ManifestProof, SborPath>,
    pub expressions: Vec<(ManifestExpression, SborPath)>,
}

impl IndexedScryptoValue {
    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let bytes =
            scrypto_encode(value).expect("Failed to encode trusted value for IndexedScryptoValue");
        Self::from_slice(&bytes).expect("Failed to convert trusted value into IndexedScryptoValue")
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, ScryptoValueDecodeError> {
        let value = scrypto_decode(slice).map_err(ScryptoValueDecodeError::DecodeError)?;
        Self::from_value(value)
    }

    pub fn from_value(value: ScryptoValue) -> Result<Self, ScryptoValueDecodeError> {
        let mut visitor = ScryptoCustomValueVisitor::new();
        let index_result = traverse_any(&mut SborPathBuf::new(), &value, &mut visitor);
        if let Err(error) = index_result {
            return Err(ScryptoValueDecodeError::ValueIndexingError(error));
        }

        Ok(Self {
            raw: scrypto_encode(&value)
                .map_err(|err| ScryptoValueDecodeError::RawValueEncodeError(err))?,
            dom: value,
            component_addresses: visitor.component_addresses,
            resource_addresses: visitor.resource_addresses,
            package_addresses: visitor.package_addresses,
            system_addresses: visitor.system_addresses,

            ownerships: visitor.ownerships,
            kv_store_ids: visitor.kv_stores,
            component_ids: visitor.components,
            non_fungible_addresses: visitor.non_fungible_addresses,
            blobs: visitor.blobs,

            buckets: visitor.buckets,
            proofs: visitor.proofs,
            expressions: visitor.expressions,
        })
    }

    pub fn owned_node_ids(&self) -> HashSet<RENodeId> {
        let mut node_ids = HashSet::new();
        for ownership in &self.ownerships {
            match ownership {
                Own::Vault(vault_id) => {
                    node_ids.insert(RENodeId::Vault(*vault_id));
                }
                Own::Bucket(bucket_id) => {
                    node_ids.insert(RENodeId::Bucket(*bucket_id));
                }
                Own::Proof(proof_id) => {
                    node_ids.insert(RENodeId::Proof(*proof_id));
                }
            }
        }
        for kv_store_id in &self.kv_store_ids {
            node_ids.insert(RENodeId::KeyValueStore(*kv_store_id));
        }
        for component_id in &self.component_ids {
            node_ids.insert(RENodeId::Component(*component_id));
        }
        node_ids
    }

    pub fn owned_node_count(&self) -> usize {
        self.ownerships.len() + self.component_ids.len() + self.kv_store_ids.len()
    }

    pub fn global_references(&self) -> HashSet<GlobalAddress> {
        let mut node_ids = HashSet::new();
        for component_address in &self.component_addresses {
            node_ids.insert(GlobalAddress::Component(*component_address));
        }
        for resource_address in &self.resource_addresses {
            node_ids.insert(GlobalAddress::Resource(*resource_address));
        }
        for package_address in &self.package_addresses {
            node_ids.insert(GlobalAddress::Package(*package_address));
        }
        for system_address in &self.system_addresses {
            node_ids.insert(GlobalAddress::System(*system_address));
        }

        // Extract resource address from non-fungible address
        for non_fungible_address in &self.non_fungible_addresses {
            node_ids.insert(GlobalAddress::Resource(
                non_fungible_address.resource_address(),
            ));
        }
        node_ids
    }

    pub fn replace_manifest_buckets_and_proofs(
        &mut self,
        proof_replacements: &mut HashMap<ManifestProof, ProofId>,
        bucket_replacements: &mut HashMap<ManifestBucket, BucketId>,
    ) -> Result<(), ValueReplacingError> {
        for (proof_id, path) in self.proofs.drain() {
            let next_id = proof_replacements
                .remove(&proof_id)
                .ok_or(ValueReplacingError::ProofIdNotFound(proof_id))?;
            let value = path.get_from_value_mut(&mut self.dom).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Own(Own::Proof(next_id));
                self.ownerships.insert(Own::Proof(next_id));
            } else {
                panic!("Should be a custom value");
            }
        }

        for (bucket_id, path) in self.buckets.drain() {
            let next_id = bucket_replacements
                .remove(&bucket_id)
                .ok_or(ValueReplacingError::BucketIdNotFound(bucket_id))?;
            let value = path.get_from_value_mut(&mut self.dom).unwrap();
            if let SborValue::Custom { value } = value {
                *value = ScryptoCustomValue::Own(Own::Bucket(next_id));
                self.ownerships.insert(Own::Bucket(next_id));
            } else {
                panic!("Should be a custom value");
            }
        }

        replace_array_element_type_id(&mut self.dom);

        self.raw = scrypto_encode(&self.dom)
            .expect("Previously encodable raw value is no longer encodable after replacement");

        Ok(())
    }
}

pub fn replace_array_element_type_id(value: &mut ScryptoValue) {
    match value {
        // primitive types
        SborValue::Unit
        | SborValue::Bool { .. }
        | SborValue::I8 { .. }
        | SborValue::I16 { .. }
        | SborValue::I32 { .. }
        | SborValue::I64 { .. }
        | SborValue::I128 { .. }
        | SborValue::U8 { .. }
        | SborValue::U16 { .. }
        | SborValue::U32 { .. }
        | SborValue::U64 { .. }
        | SborValue::U128 { .. }
        | SborValue::String { .. } => {}
        SborValue::Tuple { fields } | SborValue::Enum { fields, .. } => {
            for e in fields {
                replace_array_element_type_id(e);
            }
        }
        SborValue::Array {
            elements,
            element_type_id,
        } => {
            match element_type_id {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Bucket) => {
                    *element_type_id = ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Own);
                }
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Proof) => {
                    *element_type_id = ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Own);
                }
                _ => {}
            }

            for e in elements {
                replace_array_element_type_id(e);
            }
        }
        SborValue::Custom { .. } => {}
    }
}

impl fmt::Debug for IndexedScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_scrypto_value(f, &self.dom, &ValueFormattingContext::no_context())
    }
}

impl<'a> ContextualDisplay<ValueFormattingContext<'a>> for IndexedScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ValueFormattingContext<'a>,
    ) -> Result<(), Self::Error> {
        format_scrypto_value(f, &self.dom, context)
    }
}

/// A visitor the indexes scrypto custom values.
pub struct ScryptoCustomValueVisitor {
    // Global addresses
    pub component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub package_addresses: HashSet<PackageAddress>,
    pub system_addresses: HashSet<SystemAddress>,
    // RE interpreted
    pub ownerships: HashSet<Own>,
    pub kv_stores: HashSet<KeyValueStoreId>,
    pub components: HashSet<ComponentId>,
    pub non_fungible_addresses: HashSet<NonFungibleAddress>,
    pub blobs: Vec<(Blob, SborPath)>,
    // TX interpreted
    pub buckets: HashMap<ManifestBucket, SborPath>,
    pub proofs: HashMap<ManifestProof, SborPath>,
    pub expressions: Vec<(ManifestExpression, SborPath)>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ValueIndexingError {
    DuplicateOwnership,
    DuplicateManifestBucket,
    DuplicateManifestProof,
}

impl ScryptoCustomValueVisitor {
    pub fn new() -> Self {
        Self {
            component_addresses: HashSet::new(),
            resource_addresses: HashSet::new(),
            package_addresses: HashSet::new(),
            system_addresses: HashSet::new(),

            ownerships: HashSet::new(),
            kv_stores: HashSet::new(),
            components: HashSet::new(),
            non_fungible_addresses: HashSet::new(),
            blobs: Vec::new(),

            buckets: HashMap::new(),
            proofs: HashMap::new(),
            expressions: Vec::new(),
        }
    }
}

impl CustomValueVisitor<ScryptoCustomValue> for ScryptoCustomValueVisitor {
    type Err = ValueIndexingError;

    fn visit(
        &mut self,
        path: &mut SborPathBuf,
        value: &ScryptoCustomValue,
    ) -> Result<(), Self::Err> {
        match value {
            // Global addresses
            ScryptoCustomValue::PackageAddress(value) => {
                self.package_addresses.insert(value.clone());
            }
            ScryptoCustomValue::ComponentAddress(value) => {
                self.component_addresses.insert(value.clone());
            }
            ScryptoCustomValue::ResourceAddress(value) => {
                self.resource_addresses.insert(value.clone());
            }
            ScryptoCustomValue::SystemAddress(value) => {
                self.system_addresses.insert(value.clone());
            }

            // RE interpreted
            ScryptoCustomValue::Own(value) => {
                if !self.ownerships.insert(value.clone()) {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::Component(value) => {
                if !self.components.insert(value.clone()) {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::KeyValueStore(value) => {
                if !self.kv_stores.insert(value.clone()) {
                    return Err(ValueIndexingError::DuplicateOwnership);
                }
            }
            ScryptoCustomValue::NonFungibleAddress(value) => {
                self.non_fungible_addresses.insert(value.clone());
            }
            ScryptoCustomValue::Blob(value) => {
                self.blobs.push((value.clone(), path.clone().into()));
            }

            // TX interpreted
            ScryptoCustomValue::Bucket(value) => {
                if self
                    .buckets
                    .insert(value.clone(), path.clone().into())
                    .is_some()
                {
                    return Err(ValueIndexingError::DuplicateManifestBucket);
                }
            }
            ScryptoCustomValue::Proof(value) => {
                if self
                    .proofs
                    .insert(value.clone(), path.clone().into())
                    .is_some()
                {
                    return Err(ValueIndexingError::DuplicateManifestProof);
                }
            }
            ScryptoCustomValue::Expression(value) => {
                self.expressions.push((value.clone(), path.clone().into()));
            }

            // Uninterpreted
            ScryptoCustomValue::Hash(_)
            | ScryptoCustomValue::EcdsaSecp256k1PublicKey(_)
            | ScryptoCustomValue::EcdsaSecp256k1Signature(_)
            | ScryptoCustomValue::EddsaEd25519PublicKey(_)
            | ScryptoCustomValue::EddsaEd25519Signature(_)
            | ScryptoCustomValue::Decimal(_)
            | ScryptoCustomValue::PreciseDecimal(_)
            | ScryptoCustomValue::NonFungibleId(_) => {
                // no-op
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::rust::vec;
    use super::*;

    #[test]
    fn should_reject_duplicate_ids() {
        let buckets = scrypto_encode(&vec![ManifestBucket(0), ManifestBucket(0)]).unwrap();
        assert_eq!(
            IndexedScryptoValue::from_slice(&buckets),
            Err(ScryptoValueDecodeError::ValueIndexingError(
                ValueIndexingError::DuplicateManifestBucket
            ))
        );
    }
}
