use crate::types::*;

/// The unique identifier of a (stored) node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId([u8; 27]);

impl NodeId {
    pub const LENGTH: usize = 27;

    pub fn new(entity_byte: u8, random_bytes: &[u8; Self::LENGTH - 1]) -> Self {
        let mut buf = [0u8; Self::LENGTH];
        buf[0] = entity_byte;
        buf[1..random_bytes.len() + 1].copy_from_slice(random_bytes);
        Self(buf)
    }
}

impl AsRef<[u8]> for NodeId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<[u8; NodeId::LENGTH]> for NodeId {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0
    }
}

/// The unique identifier of a node module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(pub u8);

impl ModuleId {
    pub const TYPE_INFO: Self = Self(0x00);
    pub const SELF: Self = Self(0x01);
    pub const METADATA: Self = Self(0x02);
    pub const ROYALTY: Self = Self(0x03);
    pub const ACCESS_RULES: Self = Self(0x04);
}

/// The unique identifier of a substate within node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SubstateKey {
    Config,
    State(StateKey),
}

/// The configuration of a node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub struct ModuleConfig {
    /// When activated, the store will check the substate key for all operations (e.g. PUT/GET/LIST)
    sbor_key_enabled: bool,
    /// When activated, the store will allow LIST over the substates within the module.
    sorting_enabled: bool,
}

/// The unique identifier of a state within node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateKey(Vec<u8>);

impl StateKey {
    pub const MIN_LENGTH: usize = 1;
    pub const MAX_LENGTH: usize = 128;
    pub const MIN: Self = Self(vec![u8::MIN; StateKey::MIN_LENGTH]);
    pub const MAX: Self = Self(vec![u8::MAX; StateKey::MAX_LENGTH]);

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        Self::from_vec(slice.to_vec())
    }

    pub fn from_vec(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() < Self::MIN_LENGTH || bytes.len() > Self::MAX_LENGTH {
            None
        } else {
            Some(Self(bytes))
        }
    }
}

impl AsRef<[u8]> for StateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<Vec<u8>> for StateKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

pub fn encode_substate_id(
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(&node_id.0);
    buffer.push(module_id.0);
    match substate_key {
        SubstateKey::Config => {
            buffer.push(0);
        }
        SubstateKey::State(state_id) => {
            buffer.push(1);
            buffer.extend(state_id.as_ref()); // Length is marked by EOF
        }
    }
    buffer
}

pub fn decode_substate_id(slice: &[u8]) -> (NodeId, ModuleId, SubstateKey) {
    if slice.len() >= NodeId::LENGTH + 1 + 1 {
        // Decode node id
        let mut node_id = [0u8; NodeId::LENGTH];
        node_id.copy_from_slice(&slice[0..NodeId::LENGTH]);
        let node_id = NodeId(node_id);

        // Decode module id
        let module_id = ModuleId(slice[NodeId::LENGTH]);

        // Decode substate key
        let kind = slice[NodeId::LENGTH + 1];
        if kind == 0 && slice.len() == NodeId::LENGTH + 2 {
            return (node_id, module_id, SubstateKey::Config);
        } else if let Some(id) = StateKey::from_slice(&slice[NodeId::LENGTH + 2..]) {
            return (node_id, module_id, SubstateKey::State(id));
        }
    }
    panic!("Invalid substate id: {}", hex::encode(slice));
}

pub fn encode_substate_value(value: &IndexedScryptoValue) -> Vec<u8> {
    value.as_slice().to_vec()
}

pub fn decode_substate_value(slice: &[u8]) -> IndexedScryptoValue {
    match IndexedScryptoValue::from_slice(slice) {
        Ok(value) => value,
        Err(_) => panic!("Invalid substate value: {}", hex::encode(slice)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_substate_id() {
        let node_id = NodeId([1u8; NodeId::LENGTH]);
        let module_id = ModuleId(2);
        let substate_key = SubstateKey::State(StateKey::from_vec(vec![3]).unwrap());
        let substate_id = encode_substate_id(&node_id, module_id, &substate_key);
        assert_eq!(
            substate_id,
            vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, // node id
                2, // module id
                1, 3, // substate key
            ]
        );
        assert_eq!(
            decode_substate_id(&substate_id),
            (node_id, module_id, substate_key)
        )
    }

    #[test]
    fn test_encode_decode_substate_value() {
        let value = IndexedScryptoValue::from_typed("Hello");
        let substate_value = encode_substate_value(&value);
        assert_eq!(
            substate_value,
            vec![
                92, // prefix
                12, // string
                5,  // length
                72, 101, 108, 108, 111 // "Hello"
            ]
        );
        assert_eq!(decode_substate_value(&substate_value), value)
    }
}
