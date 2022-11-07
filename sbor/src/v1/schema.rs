use super::*;
use sbor::rust::collections::{HashMap, HashSet};

// NOTE COMPARED TO PROPOSAL:
// It turns out that Encoding/Decoding does not care about wrapping types - eg Box / Smart Pointers / Transparent
// BUT type recreation _does_ - eg to re-create a recursive type.
//
// THEREFORE - proposal is to create a separate trait / type id for recreation information, which is divorced
// from SBOR itself

pub trait Schema {
    /// This should denote a unique identifier for this type (once turned into a payload)
    /// 
    /// In particular, it should capture the uniqueness of anything relevant to the codec/payload, for example:
    /// * The payloads the codec can decode
    /// * The uniqueness of display instructions applied to the payload. EG if a wrapper type is intended to give
    ///   the value a different display interpretation, this should create a unique identifier.
    /// 
    /// Note however that entirely "transparent" types such as pointers/smart pointers/etc are intended to be
    /// transparent to the schema, so should inherit their wrapped type id. Recreation of types will be captured
    /// in a separate trait.
    ///
    /// If needing to generate a new type id, this can be generated via something like:
    /// ```
    /// impl Schema for MyType {
    ///     const SCHEMA_TYPE_ID: SchemaTypeId = generate_type_id(stringify!(MyType), &[], &[]);
    /// #   fn get_type_local_schema() { todo!() }
    /// }
    /// ```
    const SCHEMA_TYPE_ID: SchemaTypeId;

    /// Returns the local schema for the given type
    fn get_type_local_schema() -> LocalTypeSchema<SchemaTypeId>;

    /// Should add all the dependent schemas, if the type depends on any.
    /// 
    /// The algorithm should be:
    /// 
    /// - For each (POSSIBLY MUTATED) type dependency needed by this type
    ///   - Get its type id and local schema, and mutate (both!) if needed
    ///   - Do aggregator.attempt_add_schema() to the (mutated) hash and (mutated) local schema
    /// 
    /// - For each (BASE/UNMUTATED) type dependency `D`:
    ///   - If aggregator.should_read_descendents() then call `D::add_all_dependencies`
    fn add_all_dependencies(_aggregator: &mut SchemaAggregator) {}
}

pub fn generate_full_linear_schema<T: Schema>() -> FullTypeSchema {
    let mut aggregator = SchemaAggregator::new();
    aggregator.attempt_add_schema(T::SCHEMA_TYPE_ID, T::get_type_local_schema());
    T::add_all_dependencies(&mut aggregator);

    let mapped = aggregator
        .schemas
        .into_iter()
        .map(|(_, schema)| {
            // Map the LocalTypeSchema<SchemaTypeId> into LocalTypeSchema<usize>
            let decode = schema.decode.map_to_linearized(&aggregator.type_index_map);
            (decode, schema.naming)
        })
        .unzip();

    FullTypeSchema {
        types: mapped.0,
        naming: mapped.1,
    }
}

pub struct FullTypeSchema {
    pub types: Vec<TypeDecodeSchema<usize>>,
    pub naming: Vec<TypeNaming>,
}

pub struct SchemaAggregator {
    pub type_index_map: HashMap<SchemaTypeId, usize>,
    pub already_read_descendents: HashSet<SchemaTypeId>,
    pub schemas: Vec<(SchemaTypeId, LocalTypeSchema<SchemaTypeId>)>,
}

impl SchemaAggregator {
    pub fn new() -> Self {
        Self {
            type_index_map: HashMap::new(),
            already_read_descendents: HashSet::new(),
            schemas: Vec::new(),
        }
    }

    pub fn attempt_add_schema(&mut self, type_id: SchemaTypeId, schema: LocalTypeSchema<SchemaTypeId>) -> bool {
        if self.type_index_map.contains_key(&type_id) {
            return false;
        }
        self.type_index_map.insert(type_id, self.schemas.len());
        self.schemas.push((type_id, schema));
        return true;
    }

    pub fn should_read_descendents(&mut self, type_id: SchemaTypeId) -> bool {
        if self.already_read_descendents.contains(&type_id) {
            return false;
        }
        self.already_read_descendents.insert(type_id);
        return true;
    }
}

pub struct LocalTypeSchema<T> {
    decode: TypeDecodeSchema<T>,
    naming: TypeNaming,
}

/// This enables the type to be represented as eg JSON
pub struct TypeNaming {
    type_name: Option<String>,
    /// Only provided for the ProductType encoding
    field_names: Option<Vec<String>>,
}

pub enum TypeName {
    // TODO: Add explicit well-known types here
    Custom(String),
}

/// A schema for the encodings that the codec can decode
pub struct TypeDecodeSchema<T> {
    interpretation: TypeDecodeInterpretationClass,
    value: TypeDecodeSchemaClass<T>,
}

impl TypeDecodeSchema<SchemaTypeId> {
    pub fn map_to_linearized(self, type_id_map: &HashMap<SchemaTypeId, usize>) -> TypeDecodeSchema<usize> {
        TypeDecodeSchema {
            interpretation: self.interpretation,
            value: match self.value {
                TypeDecodeSchemaClass::RawBytes => TypeDecodeSchemaClass::RawBytes,
                TypeDecodeSchemaClass::ProductType { types } => {
                    TypeDecodeSchemaClass::ProductType {
                        types: types.iter().map(|t| resolve_index(type_id_map, t)).collect(),
                    }
                },
                TypeDecodeSchemaClass::SumType {
                    u8_discriminators,
                    u16_discriminators,
                    u32_discriminators,
                    u64_discriminators,
                    any_discriminators
                } => TypeDecodeSchemaClass::SumType {
                    u8_discriminators: u8_discriminators
                        .into_iter()
                        .map(|(d, t)| (d, resolve_index(type_id_map, &t)))
                        .collect(),
                    u16_discriminators: u16_discriminators
                        .into_iter()
                        .map(|(d, t)| (d, resolve_index(type_id_map, &t)))
                        .collect(),
                    u32_discriminators: u32_discriminators
                        .into_iter()
                        .map(|(d, t)| (d, resolve_index(type_id_map, &t)))
                        .collect(),
                    u64_discriminators: u64_discriminators
                        .into_iter()
                        .map(|(d, t)| (d, resolve_index(type_id_map, &t)))
                        .collect(),
                    any_discriminators: any_discriminators
                        .into_iter()
                        .map(|(d, t)| (resolve_index(type_id_map, &d), resolve_index(type_id_map, &t)))
                        .collect(),
                },
                TypeDecodeSchemaClass::List { item } => {
                    TypeDecodeSchemaClass::List { item: resolve_index(type_id_map, &item) }
                },
                TypeDecodeSchemaClass::Map { key, value } => {
                    TypeDecodeSchemaClass::Map {
                        key: resolve_index(type_id_map, &key),
                        value: resolve_index(type_id_map, &value),
                    }
                },
                TypeDecodeSchemaClass::Any => TypeDecodeSchemaClass::Any,
            },
        }
    }
}

fn resolve_index(type_id_map: &HashMap<SchemaTypeId, usize>, type_id: &SchemaTypeId) -> usize {
    *type_id_map.get(type_id)
        .unwrap_or_else(|| panic!("Something went wrong in the schema aggregation process - type id wasn't added: {:?}", type_id))
}

pub enum TypeDecodeInterpretationClass {
    Fixed(u8),
    OneOf(Vec<u8>),
    Any,
}

pub enum TypeDecodeSchemaClass<T> {
    RawBytes, // TODO: Add some kind of length validation
    ProductType { types: Vec<T> },
    SumType {
        u8_discriminators: Vec<(u8, T)>, // IndexMap?
        u16_discriminators: Vec<(u16, T)>, // IndexMap?
        u32_discriminators: Vec<(u32, T)>,  // IndexMap?
        u64_discriminators: Vec<(u64, T)>,  // IndexMap?
        any_discriminators: Vec<(T, T)>, // IndexMap?
    },
    List { item: T }, // TODO: Add some kind of length validation thing
    Map { key: T, value: T }, // TODO: Add some kind of length validation thing
    Any,
}

/* TYPES: */

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub struct SchemaTypeId(pub [u8; 20]);

pub const fn generate_type_id(name: &str, code: &[u8], dependencies: &[SchemaTypeId]) -> SchemaTypeId {
    let buffer = const_sha1::ConstBuffer::from_slice(name.as_bytes())
        .push_slice(&code);

    // Const looping isn't allowed - but we can use recursion instead
    let buffer = capture_dependent_type_ids(buffer, 0, dependencies);

    SchemaTypeId(const_sha1::sha1(&buffer).bytes())
}

const fn capture_dependent_type_ids(buffer: const_sha1::ConstBuffer, next: usize, dependencies: &[SchemaTypeId]) -> const_sha1::ConstBuffer {
    if next == dependencies.len() {
        return buffer;
    }
    capture_dependent_type_ids(
        buffer.push_slice(dependencies[next].0.as_slice()),
        next + 1,
        dependencies
    )
}

// The below might actually prove to be a bit useless
// TODO: We actually probably want reserved / default indices for the linear schema, to avoid capturing
//       type schemas for eg strings

pub const fn generate_well_known_type_id(interpretation: u8) -> SchemaTypeId {
    SchemaTypeId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, interpretation])
}

pub enum DefaultSchemaTypes { }

impl DefaultSchemaTypes {
    pub const BOOLEAN: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::BOOLEAN);
    pub const UTF8_STRING: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::UTF8_STRING);
    pub const UTF8_STRING_DISCRIMINATOR: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::UTF8_STRING_DISCRIMINATOR);
    pub const SBOR_ANY: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::SBOR_ANY);
    pub const PLAIN_RAW_BYTES: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::PLAIN_RAW_BYTES);

    pub const U8: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::U8);
    pub const U16: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::U16);
    pub const U32: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::U32);
    pub const U64: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::U64);
    pub const U128: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::U128);
    pub const U256: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::U256);
    pub const USIZE: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::USIZE);

    pub const I8: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::I8);
    pub const I16: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::I16);
    pub const I32: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::I32);
    pub const I64: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::I64);
    pub const I128: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::I128);
    pub const I256: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::I256);
    pub const ISIZE: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::ISIZE);

    pub const UNIT: SchemaTypeId = generate_well_known_type_id(DefaultInterpretations::UNIT);
}