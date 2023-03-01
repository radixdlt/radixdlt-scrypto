use crate::data::model::*;
use crate::data::*;
use core::convert::Infallible;
use sbor::path::SborPathBuf;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use utils::ContextualDisplay;

#[derive(Clone, PartialEq, Eq)]
pub struct IndexedScryptoValue {
    bytes: Vec<u8>,
    value: ScryptoValue,
    global_references: HashSet<Address>,
    owned_nodes: Vec<Own>,
}

impl IndexedScryptoValue {
    fn new(bytes: Vec<u8>, value: ScryptoValue) -> Self {
        let mut visitor = ScryptoValueVisitor::new();
        traverse_any(&mut SborPathBuf::new(), &value, &mut visitor).expect("Infallible");

        Self {
            bytes,
            value,
            global_references: visitor.global_references,
            owned_nodes: visitor.owned_nodes,
        }
    }

    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    /// Converts a rust value into `IndexedScryptoValue`, assuming it follows RE semantics.
    pub fn from_typed<T: ScryptoEncode + ?Sized>(value: &T) -> Self {
        let bytes = scrypto_encode(value).expect("Failed to encode typed value");
        let value =
            scrypto_decode(&bytes).expect("Failed to decode Scrypto SBOR bytes into ScryptoValue");

        Self::new(bytes, value)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        let value = scrypto_decode(slice)?;
        Ok(Self::new(slice.to_vec(), value))
    }

    pub fn from_vec(vec: Vec<u8>) -> Result<Self, DecodeError> {
        let value = scrypto_decode(&vec)?;
        Ok(Self::new(vec, value))
    }

    pub fn from_value(value: ScryptoValue) -> Self {
        let bytes = scrypto_encode(&value).expect("Failed to encode ScryptoValue into bytes");
        Self::new(bytes, value)
    }

    pub fn as_typed<T: ScryptoDecode>(&self) -> Result<T, DecodeError> {
        scrypto_decode(&self.bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn as_value(&self) -> &ScryptoValue {
        &self.value
    }

    pub fn global_references(&self) -> &HashSet<Address> {
        &self.global_references
    }

    pub fn owned_node_ids(&self) -> &Vec<Own> {
        &self.owned_nodes
    }

    pub fn unpack(self) -> (Vec<u8>, ScryptoValue, Vec<Own>, HashSet<Address>) {
        (
            self.bytes,
            self.value,
            self.owned_nodes,
            self.global_references,
        )
    }
}

impl Into<Vec<u8>> for IndexedScryptoValue {
    fn into(self) -> Vec<u8> {
        self.bytes
    }
}
impl Into<ScryptoValue> for IndexedScryptoValue {
    fn into(self) -> ScryptoValue {
        self.value
    }
}

impl fmt::Debug for IndexedScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_scrypto_value(
            f,
            self.as_value(),
            &ScryptoValueDisplayContext::no_context(),
        )
    }
}

impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for IndexedScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_scrypto_value(f, self.as_value(), context)
    }
}

pub struct ScryptoValueVisitor {
    pub global_references: HashSet<Address>,
    pub owned_nodes: Vec<Own>,
}

impl ScryptoValueVisitor {
    pub fn new() -> Self {
        Self {
            global_references: HashSet::new(),
            owned_nodes: Vec::new(),
        }
    }
}

impl ValueVisitor<ScryptoCustomValueKind, ScryptoCustomValue> for ScryptoValueVisitor {
    type Err = Infallible;

    fn visit(
        &mut self,
        _path: &mut SborPathBuf,
        value: &ScryptoCustomValue,
    ) -> Result<(), Self::Err> {
        match value {
            ScryptoCustomValue::Address(value) => {
                self.global_references.insert(value.clone());
            }
            ScryptoCustomValue::Own(value) => {
                self.owned_nodes.push(value.clone());
            }

            ScryptoCustomValue::Decimal(_)
            | ScryptoCustomValue::PreciseDecimal(_)
            | ScryptoCustomValue::NonFungibleLocalId(_) => {
                // no-op
            }
        }
        Ok(())
    }
}
