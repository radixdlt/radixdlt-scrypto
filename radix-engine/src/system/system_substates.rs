use crate::internal_prelude::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(categorize_types = "")]
pub struct FieldSubstateV1<V> {
    pub payload: V,
    pub mutability: SubstateMutability,
}

// Note - we manually version these instead of using the defined_versioned! macro,
// to avoid having additional / confusing methods on FieldSubstate<X>
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(categorize_types = "")]
pub enum FieldSubstate<T> {
    V1(FieldSubstateV1<T>),
}

impl<V> FieldSubstate<V> {
    pub fn new_field(payload: V, mutability: SubstateMutability) -> Self {
        FieldSubstate::V1(FieldSubstateV1 {
            payload,
            mutability,
        })
    }

    pub fn new_mutable_field(payload: V) -> Self {
        Self::new_field(payload, SubstateMutability::Mutable)
    }

    pub fn new_locked_field(payload: V) -> Self {
        Self::new_field(payload, SubstateMutability::Immutable)
    }

    pub fn lock(&mut self) {
        let mutability = match self {
            FieldSubstate::V1(FieldSubstateV1 { mutability, .. }) => mutability,
        };
        *mutability = SubstateMutability::Immutable;
    }

    pub fn payload(&self) -> &V {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { payload, .. }) => payload,
        }
    }

    pub fn mutability(&self) -> &SubstateMutability {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { mutability, .. }) => mutability,
        }
    }

    pub fn into_payload(self) -> V {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { payload, .. }) => payload,
        }
    }

    pub fn into_mutability(self) -> SubstateMutability {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { mutability, .. }) => mutability,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SubstateMutability {
    Mutable,
    Immutable,
}
