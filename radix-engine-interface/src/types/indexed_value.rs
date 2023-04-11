use core::cell::RefCell;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::types::*;
use sbor::rust::cell::Ref;
use sbor::rust::fmt;
use sbor::rust::prelude::*;
use sbor::serde_serialization::SborPayloadWithoutSchema;
use sbor::serde_serialization::SchemalessSerializationContext;
use sbor::serde_serialization::SerializationMode;
use sbor::traversal::TraversalEvent;
use sbor::*;
use utils::ContextualDisplay;
use utils::ContextualSerialize;

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    bytes: Vec<u8>,
    references: IndexSet<NodeId>,
    owned_nodes: Vec<NodeId>,
    scrypto_value: RefCell<Option<ScryptoValue>>,
}

impl IndexedScryptoValue {
    fn new(bytes: Vec<u8>) -> Result<Self, DecodeError> {
        let mut traverser = ScryptoTraverser::new(
            &bytes,
            SCRYPTO_SBOR_V1_MAX_DEPTH,
            Some(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX),
            true,
        );
        let mut references = index_set_new::<NodeId>();
        let mut owned_nodes = Vec::<NodeId>::new();
        loop {
            let event = traverser.next_event();
            match event.event {
                TraversalEvent::PayloadPrefix => {}
                TraversalEvent::ContainerStart(_) => {}
                TraversalEvent::ContainerEnd(_) => {}
                TraversalEvent::TerminalValue(r) => {
                    if let traversal::TerminalValueRef::Custom(c) = r {
                        match c.0 {
                            ScryptoCustomValue::Reference(node_id) => {
                                references.insert(node_id.0.into());
                            }
                            ScryptoCustomValue::Own(node_id) => {
                                owned_nodes.push(node_id.0.into());
                            }
                            ScryptoCustomValue::Decimal(_)
                            | ScryptoCustomValue::PreciseDecimal(_)
                            | ScryptoCustomValue::NonFungibleLocalId(_) => {}
                        }
                    }
                }
                TraversalEvent::TerminalValueBatch(_) => {}
                TraversalEvent::End => {
                    break;
                }
                TraversalEvent::DecodeError(e) => {
                    return Err(e);
                }
            }
        }

        Ok(Self {
            bytes,
            references,
            owned_nodes,
            scrypto_value: RefCell::new(None),
        })
    }

    fn get_scrypto_value(&self) -> Ref<ScryptoValue> {
        let is_empty = { self.scrypto_value.borrow().is_none() };

        if is_empty {
            *self.scrypto_value.borrow_mut() = Some(
                scrypto_decode::<ScryptoValue>(&self.bytes)
                    .expect("Failed to decode bytes in IndexedScryptoValue"),
            );
        }

        Ref::map(self.scrypto_value.borrow(), |v| v.as_ref().unwrap())
    }

    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let bytes = scrypto_encode(value).expect("Failed to encode trusted Rust value");
        Self::new(bytes).expect("Failed to index trusted Rust value")
    }

    pub fn from_scrypto_value(value: ScryptoValue) -> Self {
        let bytes = scrypto_encode(&value).expect("Failed to encode trusted ScryptoValue");
        Self::new(bytes).expect("Failed to index trusted ScryptoValue")
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        Self::new(slice.to_vec())
    }

    pub fn from_vec(vec: Vec<u8>) -> Result<Self, DecodeError> {
        Self::new(vec)
    }

    pub fn to_scrypto_value(&self) -> ScryptoValue {
        self.get_scrypto_value().clone()
    }

    pub fn as_scrypto_value(&self) -> Ref<ScryptoValue> {
        self.get_scrypto_value()
    }

    pub fn as_typed<T: ScryptoDecode>(&self) -> Result<T, DecodeError> {
        scrypto_decode(&self.bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn references(&self) -> &IndexSet<NodeId> {
        &self.references
    }

    pub fn owned_node_ids(&self) -> &Vec<NodeId> {
        &self.owned_nodes
    }

    pub fn unpack(self) -> (Vec<u8>, Vec<NodeId>, IndexSet<NodeId>) {
        (self.bytes, self.owned_nodes, self.references)
    }
}

impl Into<Vec<u8>> for IndexedScryptoValue {
    fn into(self) -> Vec<u8> {
        self.bytes
    }
}

impl fmt::Debug for IndexedScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.contextual_format(f, &ScryptoValueSerializationContext::no_context())
    }
}

impl<'a> ContextualDisplay<ScryptoValueSerializationContext<'a>> for IndexedScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueSerializationContext<'a>,
    ) -> Result<(), Self::Error> {
        let json = serde_json::to_string(
            &SborPayloadWithoutSchema::<ScryptoCustomTypeExtension>::new(self.as_slice())
                .serializable(SchemalessSerializationContext {
                    mode: SerializationMode::Invertible,
                    custom_context: context.clone(),
                }),
        )
        .unwrap();

        f.write_str(&json)
    }
}
