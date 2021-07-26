extern crate alloc;
use alloc::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blueprint {
    pub version: String,
    pub metadata: Metadata,
    pub components: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub version: String,
    pub author: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub inputs: Vec<Type>,
    pub output: Type,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Type {
    /// &self
    SelfRef,

    /// &mut self
    SelfMut,

    /// u8 integer
    U8,

    /// u16 integer
    U16,

    /// u32 integer
    U32,

    /// String
    String,

    /// Object
    Object {
        name: String,
        attributes: BTreeMap<String, Type>,
    },

    /// Array
    Array { elements: Vec<Type> },

    /// Vector
    Vec { element: Box<Type> },
}
