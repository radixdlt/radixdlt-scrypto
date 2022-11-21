#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum TypeRef {
    /// This takes a well_known type index.
    /// Would use SborTypeId<X> here, but blocked on https://github.com/rust-lang/rfcs/pull/2632
    WellKnownType([u8; 1]),
    /// For non-simple types
    Complex(ComplexTypeHash),
}

pub type ComplexTypeHash = [u8; 20];

impl TypeRef {
    pub const fn complex(name: &str, dependencies: &[TypeRef], code: &[u8]) -> Self {
        generate_type_ref(name, code, dependencies)
    }

    pub const fn well_known(type_id: u8) -> Self {
        Self::WellKnownType([type_id])
    }

    pub const fn is_complex(&self) -> bool {
        match self {
            TypeRef::WellKnownType(_) => false,
            TypeRef::Complex(_) => true,
        }
    }

    pub const fn as_slice(&self) -> &[u8] {
        match &self {
            TypeRef::WellKnownType(x) => x,
            TypeRef::Complex(hash) => hash,
        }
    }
}

const fn generate_type_ref(name: &str, code: &[u8], dependencies: &[TypeRef]) -> TypeRef {
    let buffer = const_sha1::ConstBuffer::from_slice(name.as_bytes()).push_slice(&code);

    // Const looping isn't allowed - but we can use recursion instead
    let buffer = capture_dependent_type_ids(buffer, 0, dependencies);

    TypeRef::Complex(const_sha1::sha1(&buffer).bytes())
}

const fn capture_dependent_type_ids(
    buffer: const_sha1::ConstBuffer,
    next: usize,
    dependencies: &[TypeRef],
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
