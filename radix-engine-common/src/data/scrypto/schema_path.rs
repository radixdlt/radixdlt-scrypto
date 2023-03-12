use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;

use self::SchemaSubPath::{Field, Index};
use crate::*;

use super::ScryptoSchema;
use super::ScryptoTypeKind;

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

    pub fn to_sbor_path(
        &self,
        schema: &ScryptoSchema,
        type_index: LocalTypeIndex,
    ) -> Option<(SborPath, ScryptoTypeKind<LocalTypeIndex>)> {
        let mut sbor_path: Vec<usize> = vec![];
        let mut cur_type = type_index;

        for sub_path in &self.0 {
            match sub_path {
                SchemaSubPath::Index(index) => match schema.resolve_type_kind(cur_type) {
                    Some(TypeKind::Array { element_type }) => {
                        cur_type = element_type.clone();
                        sbor_path.push(*index);
                    }
                    _ => return None,
                },
                SchemaSubPath::Field(field) => match (
                    schema.resolve_type_kind(cur_type),
                    schema.resolve_type_metadata(cur_type),
                ) {
                    (
                        Some(TypeKind::Tuple { field_types }),
                        Some(TypeMetadata { child_names, .. }),
                    ) => match child_names.as_ref() {
                        Some(ChildNames::NamedFields(fields)) => {
                            if let Some(index) = fields.iter().position(|f| f.eq(field)) {
                                cur_type = field_types
                                    .get(index)
                                    .cloned()
                                    .expect("Inconsistent schema");
                                sbor_path.push(index);
                            } else {
                                return None;
                            }
                        }
                        _ => return None,
                    },
                    _ => return None,
                },
            }
        }

        let type_kind = schema
            .resolve_type_kind(cur_type)
            .cloned()
            .expect("Inconsistent schema");

        Some((SborPath::new(sbor_path), type_kind))
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
