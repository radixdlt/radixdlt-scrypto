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

#[derive(Debug, Copy, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SubstateMutability {
    Mutable,
    Immutable,
}


pub type KeyValueEntrySubstateV1<V> = KVSubstate<Option<V>>;

// NOTE: This type trick seems to fix the generic ScryptoSbor issue
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct KVSubstate<E> {
    pub value: E,
    pub mutability: SubstateMutability,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(categorize_types = "")]
pub enum KeyValueEntrySubstate<V> {
    V1(KeyValueEntrySubstateV1<V>)
}

impl<V> KeyValueEntrySubstate<V> {
    pub fn lock(&mut self) {
        match self {
            KeyValueEntrySubstate::V1(substate) => {
                substate.mutability = SubstateMutability::Immutable;
            }
        }
    }

    pub fn into_value(self) -> Option<V> {
        match self {
            KeyValueEntrySubstate::V1(KeyValueEntrySubstateV1 { value, .. }) => value,
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            KeyValueEntrySubstate::V1(substate) => {
                matches!(substate.mutability, SubstateMutability::Mutable)
            }
        }
    }

    pub fn entry(value: V) -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: Some(value),
            mutability: SubstateMutability::Mutable,
        })
    }

    pub fn locked_entry(value: V) -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: Some(value),
            mutability: SubstateMutability::Immutable,
        })
    }

    pub fn locked_empty_entry() -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: None,
            mutability: SubstateMutability::Immutable,
        })
    }

    pub fn remove(&mut self) -> Option<V> {
        match self {
            KeyValueEntrySubstate::V1(substate) => {
                substate.value.take()
            }
        }
    }

    pub fn mutability(&self) -> SubstateMutability {
        match self {
            KeyValueEntrySubstate::V1(substate) => {
                substate.mutability.clone()
            }
        }
    }
}

impl<V> Default for KeyValueEntrySubstate<V> {
    fn default() -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: Option::None,
            mutability: SubstateMutability::Mutable,
        })
    }
}

pub type IndexEntrySubstate<V> = V;
