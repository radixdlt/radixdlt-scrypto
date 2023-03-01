use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;

use self::SchemaSubPath::{Field, Index};
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Sbor, Ord, PartialOrd)]
pub enum SchemaSubPath {
    Index(usize),
    Field(String),
}

impl FromStr for SchemaSubPath {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: check that field is a valid field name string
        let sub_path = s
            .parse::<usize>()
            .map(|i| Index(i))
            .unwrap_or(Field(s.to_string()));
        Ok(sub_path)
    }
}

/// Describes a value located in some sbor given a schema for that sbor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Sbor, Ord, PartialOrd)]
pub struct SchemaPath(pub Vec<SchemaSubPath>);

impl SchemaPath {
    pub fn new() -> Self {
        SchemaPath(vec![])
    }

    fn sub_path(&mut self, sub_path: SchemaSubPath) -> &Self {
        self.0.push(sub_path);
        self
    }

    pub fn field(&mut self, field: &str) -> &Self {
        self.0.push(SchemaSubPath::Field(field.to_string()));
        self
    }

    pub fn index(&mut self, index: usize) -> &Self {
        self.0.push(SchemaSubPath::Index(index));
        self
    }
}

#[derive(Debug)]
pub enum SchemaPathParseError {
    InvalidPath,
}

impl FromStr for SchemaPath {
    type Err = SchemaPathParseError;

    fn from_str(s: &str) -> Result<Self, SchemaPathParseError> {
        let sub_paths = s.split("/");
        let mut schema_path = SchemaPath::new();
        for sub_path_str in sub_paths {
            let sub_path = sub_path_str
                .parse()
                .map_err(|_| SchemaPathParseError::InvalidPath)?;
            schema_path.sub_path(sub_path);
        }
        Ok(schema_path)
    }
}
