use crate::v2::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum GlobalTypeRef {
    /// This takes a well_known type index.
    /// Would use SborTypeId<X> here, but this needs to be usable from a const context
    /// so we're blocked on https://github.com/rust-lang/rfcs/pull/2632
    WellKnown([u8; 1]),
    /// For non-simple types
    Complex(ComplexTypeHash),
}
impl TypeLink for GlobalTypeRef {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SchemaLocalTypeRef {
    /// This takes a well_known type index
    WellKnown(u8),
    /// For non-simple types
    SchemaLocal(usize),
}
impl TypeLink for SchemaLocalTypeRef {}

pub type ComplexTypeHash = [u8; 20];

impl GlobalTypeRef {
    pub const fn complex(name: &str, dependencies: &[GlobalTypeRef]) -> Self {
        generate_type_ref(&[name], &[], dependencies)
    }

    pub const fn complex_with_code(
        name: &str,
        dependencies: &[GlobalTypeRef],
        code: &[u8],
    ) -> Self {
        generate_type_ref(&[name], &[code], dependencies)
    }

    pub const fn enum_variant(
        name: &str,
        variant_name: &str,
        dependencies: &[GlobalTypeRef],
    ) -> Self {
        generate_type_ref(&[name, variant_name], &[], dependencies)
    }

    pub const fn enum_variant_with_code(
        name: &str,
        variant_name: &str,
        dependencies: &[GlobalTypeRef],
        code: &[u8],
    ) -> Self {
        generate_type_ref(&[name, variant_name], &[code], dependencies)
    }

    pub const fn complex_sized(name: &str, dependencies: &[GlobalTypeRef], size: usize) -> Self {
        generate_type_ref(&[name], &[&size.to_le_bytes()], dependencies)
    }

    pub const fn well_known(type_id: u8) -> Self {
        Self::WellKnown([type_id])
    }

    pub const fn is_complex(&self) -> bool {
        match self {
            GlobalTypeRef::WellKnown(_) => false,
            GlobalTypeRef::Complex(_) => true,
        }
    }

    pub const fn as_slice(&self) -> &[u8] {
        match &self {
            GlobalTypeRef::WellKnown(x) => x,
            GlobalTypeRef::Complex(hash) => hash,
        }
    }
}

const fn generate_type_ref(
    names: &[&str],
    type_data: &[&[u8]],
    dependencies: &[GlobalTypeRef],
) -> GlobalTypeRef {
    let buffer = const_sha1::ConstBuffer::new();

    // Const looping isn't allowed - but we can use recursion instead
    let buffer = capture_names(buffer, 0, names);
    let buffer = capture_type_data(buffer, 0, type_data);
    let buffer = capture_dependent_type_ids(buffer, 0, dependencies);

    GlobalTypeRef::Complex(const_sha1::sha1(&buffer).bytes())
}

const fn capture_names(
    buffer: const_sha1::ConstBuffer,
    next: usize,
    names: &[&str],
) -> const_sha1::ConstBuffer {
    if next == names.len() {
        return buffer;
    }
    capture_names(buffer.push_slice(names[next].as_bytes()), next + 1, names)
}

const fn capture_type_data(
    buffer: const_sha1::ConstBuffer,
    next: usize,
    type_data: &[&[u8]],
) -> const_sha1::ConstBuffer {
    if next == type_data.len() {
        return buffer;
    }
    capture_type_data(buffer.push_slice(type_data[next]), next + 1, type_data)
}

const fn capture_dependent_type_ids(
    buffer: const_sha1::ConstBuffer,
    next: usize,
    dependencies: &[GlobalTypeRef],
) -> const_sha1::ConstBuffer {
    if next == dependencies.len() {
        return buffer;
    }
    capture_dependent_type_ids(
        buffer.push_slice(dependencies[next].as_slice()),
        next + 1,
        dependencies,
    )
}
