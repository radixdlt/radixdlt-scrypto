use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::str::FromStr;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::describe::Fields;
use sbor::path::SborPath;
use sbor::*;
use crate::resource::schema_path::SchemaSubPath::{Field, Index};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
enum SchemaSubPath {
    Index(usize),
    Field(String),
}

impl FromStr for SchemaSubPath {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: check that field is a valid field name string
        let sub_path = s.parse::<usize>()
            .map(|i| Index(i))
            .unwrap_or(Field(s.to_string()));
        Ok(sub_path)
    }
}

/// Describes a value located in some sbor given a schema for that sbor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub struct SchemaPath(Vec<SchemaSubPath>);

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

    pub fn to_sbor_path(&self, schema: &Type) -> Option<SborPath> {
        let mut cur_type = schema;
        let mut sbor_path: Vec<usize> = vec![];

        for sub_path in &self.0 {
            match sub_path {
                SchemaSubPath::Index(index) => match cur_type {
                    Type::Vec { element } => {
                        cur_type = element.as_ref();
                        sbor_path.push(*index);
                    }
                    Type::Array { element, length: _ } => {
                        cur_type = element.as_ref();
                        sbor_path.push(*index);
                    }
                    _ => return Option::None,
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
                                    return Option::None;
                                }
                            }
                            _ => return Option::None,
                        }
                    } else {
                        return Option::None;
                    }
                }
            }
        }

        Option::Some(SborPath::new(sbor_path))
    }
}

#[derive(Debug)]
pub enum SchemaPathParseError {
    InvalidPath
}

impl FromStr for SchemaPath {
    type Err = SchemaPathParseError;

    fn from_str(s: &str) -> Result<Self, SchemaPathParseError> {
        let sub_paths = s.split("/");
        let mut schema_path = SchemaPath::new();
        for sub_path_str in sub_paths {
            let sub_path = sub_path_str.parse()
                .map_err(|_| SchemaPathParseError::InvalidPath)?;
            schema_path.sub_path(sub_path);
        }
        Ok(schema_path)
    }
}