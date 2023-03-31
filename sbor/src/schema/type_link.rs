use sbor::rust::fmt::Debug;
use sbor::*;

/// Marker trait for a link between [`TypeKind`]s:
/// - [`GlobalTypeId`]: A global identifier for a type (a well known id, or type hash)
/// - [`LocalTypeIndex`]: A link in the context of a schema (a well known id, or a local index)
pub trait SchemaTypeLink: Debug + Clone + PartialEq + Eq {}

/// This is a global identifier for a type.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Sbor)]
pub enum GlobalTypeId {
    /// This takes a well_known type index.
    ///
    /// This wraps a `[u8; 1]` because it needs to be able to be turned into a `[u8]` in a const context.
    WellKnown([u8; 1]),
    /// The global type hash of a type - used for types which aren't well known.
    Novel(TypeHash),
}
impl SchemaTypeLink for GlobalTypeId {}

pub type TypeHash = [u8; 20];

impl GlobalTypeId {
    pub const fn novel(name: &str, dependencies: &[GlobalTypeId]) -> Self {
        generate_type_hash(&[name], &[], dependencies)
    }

    pub const fn novel_with_code(name: &str, dependencies: &[GlobalTypeId], code: &[u8]) -> Self {
        generate_type_hash(&[name], &[("code", code)], dependencies)
    }

    pub const fn novel_validated(
        name: &str,
        dependencies: &[GlobalTypeId],
        validations: &[(&str, &[u8])],
    ) -> Self {
        generate_type_hash(&[name], validations, dependencies)
    }

    pub const fn well_known(well_known_type_id: u8) -> Self {
        Self::WellKnown([well_known_type_id])
    }

    pub const fn as_slice(&self) -> &[u8] {
        match &self {
            GlobalTypeId::WellKnown(x) => x,
            GlobalTypeId::Novel(hash) => hash,
        }
    }
}

const fn generate_type_hash(
    names: &[&str],
    type_data: &[(&str, &[u8])],
    dependencies: &[GlobalTypeId],
) -> GlobalTypeId {
    let buffer = const_sha1::ConstSlice::new();

    // Const looping isn't allowed - but we can use recursion instead
    let buffer = capture_names(buffer, 0, names);
    let buffer = capture_type_data(buffer, 0, type_data);
    let buffer = capture_dependent_type_ids(buffer, 0, dependencies);

    GlobalTypeId::Novel(const_sha1::sha1(buffer.as_slice()).as_bytes())
}

const fn capture_names(
    buffer: const_sha1::ConstSlice,
    next: usize,
    names: &[&str],
) -> const_sha1::ConstSlice {
    if next == names.len() {
        return buffer;
    }
    let buffer = buffer.push_slice(names[next].as_bytes());
    capture_names(buffer, next + 1, names)
}

const fn capture_type_data(
    buffer: const_sha1::ConstSlice,
    next: usize,
    type_data: &[(&str, &[u8])],
) -> const_sha1::ConstSlice {
    if next == type_data.len() {
        return buffer;
    }
    let buffer = buffer.push_slice(type_data[next].0.as_bytes());
    let buffer = buffer.push_slice(type_data[next].1);
    capture_type_data(buffer, next + 1, type_data)
}

const fn capture_dependent_type_ids(
    buffer: const_sha1::ConstSlice,
    next: usize,
    dependencies: &[GlobalTypeId],
) -> const_sha1::ConstSlice {
    if next == dependencies.len() {
        return buffer;
    }
    let buffer = buffer.push_slice(dependencies[next].as_slice());
    capture_dependent_type_ids(buffer, next + 1, dependencies)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// This is the [`SchemaTypeLink`] used in a linearized [`Schema`] to link [`TypeKind`]s.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Sbor)]
pub enum LocalTypeIndex {
    /// This takes a well_known type index
    WellKnown(u8),
    /// For non-simple types
    SchemaLocalIndex(usize),
}
impl SchemaTypeLink for LocalTypeIndex {}

impl LocalTypeIndex {
    pub fn any() -> Self {
        Self::WellKnown(basic_well_known_types::ANY_ID)
    }
}
