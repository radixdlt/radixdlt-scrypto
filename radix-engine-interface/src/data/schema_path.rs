use sbor::path::SborPath;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;

use self::SchemaSubPath::{Field, Index};
use crate::*;
use scrypto_abi::*;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, LegacyDescribe, Categorize, Encode, Decode, Ord, PartialOrd,
)]
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
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, LegacyDescribe, Categorize, Encode, Decode, Ord, PartialOrd,
)]
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

    pub fn to_sbor_path<'a>(&self, schema: &'a Type) -> Option<(SborPath, &'a Type)> {
        let mut cur_type = schema;
        let mut sbor_path: Vec<usize> = vec![];

        for sub_path in &self.0 {
            match sub_path {
                SchemaSubPath::Index(index) => match cur_type {
                    Type::Vec { element_type } => {
                        cur_type = element_type.as_ref();
                        sbor_path.push(*index);
                    }
                    Type::Array {
                        element_type,
                        length: _,
                    } => {
                        cur_type = element_type.as_ref();
                        sbor_path.push(*index);
                    }
                    _ => return None,
                },
                SchemaSubPath::Field(field) => {
                    if let Type::Struct { name: _, fields } = cur_type {
                        match fields {
                            Fields::Named { named } => {
                                if let Some(index) = named
                                    .iter()
                                    .position(|(field_name, _)| field_name.eq(field))
                                {
                                    let (_, next_type) = named.get(index).unwrap();
                                    cur_type = next_type;
                                    sbor_path.push(index);
                                } else {
                                    return None;
                                }
                            }
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                }
            }
        }

        Some((SborPath::new(sbor_path), cur_type))
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
