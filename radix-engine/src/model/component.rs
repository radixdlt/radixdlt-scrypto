use hashbrown::HashMap;
use sbor::*;
use scrypto::types::*;

use crate::model::*;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Component {
    blueprint: Address,
    name: String,
    state: Vec<u8>,
    buckets: HashMap<BID, Bucket>,
}

impl Component {
    pub fn new(blueprint: Address, name: String, state: Vec<u8>) -> Self {
        Self {
            blueprint,
            name,
            state,
            buckets: HashMap::new(),
        }
    }

    pub fn blueprint(&self) -> Address {
        self.blueprint
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }

    pub fn buckets(&mut self) -> &mut HashMap<BID, Bucket> {
        self.buckets()
    }
}
