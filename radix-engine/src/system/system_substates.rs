use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct FieldSubstateV1<V> {
    pub payload: V,
    pub lock_status: LockStatus,
}

// Note - we manually version these instead of using the defined_versioned! macro,
// to avoid FieldSubstate<X> implementing UpgradableVersioned and inheriting
// potentially confusing methods
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FieldSubstate<T> {
    V1(FieldSubstateV1<T>),
}

impl<V> FieldSubstate<V> {
    pub fn new_field(payload: V, lock_status: LockStatus) -> Self {
        FieldSubstate::V1(FieldSubstateV1 {
            payload,
            lock_status,
        })
    }

    pub fn new_unlocked_field(payload: V) -> Self {
        Self::new_field(payload, LockStatus::Unlocked)
    }

    pub fn new_locked_field(payload: V) -> Self {
        Self::new_field(payload, LockStatus::Locked)
    }

    pub fn lock(&mut self) {
        let lock_status = match self {
            FieldSubstate::V1(FieldSubstateV1 { lock_status, .. }) => lock_status,
        };
        *lock_status = LockStatus::Locked;
    }

    pub fn payload(&self) -> &V {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { payload, .. }) => payload,
        }
    }

    pub fn lock_status(&self) -> LockStatus {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { lock_status, .. }) => *lock_status,
        }
    }

    pub fn into_payload(self) -> V {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { payload, .. }) => payload,
        }
    }

    pub fn into_lock_status(self) -> LockStatus {
        match self {
            FieldSubstate::V1(FieldSubstateV1 { lock_status, .. }) => lock_status,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum LockStatus {
    Unlocked,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct KeyValueEntrySubstateV1<V> {
    pub value: Option<V>,
    pub lock_status: LockStatus,
}

// Note - we manually version these instead of using the defined_versioned! macro,
// to avoid KeyValueEntrySubstate<X> implementing UpgradableVersioned and inheriting
// potentially confusing methods
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum KeyValueEntrySubstate<V> {
    V1(KeyValueEntrySubstateV1<V>),
}

impl<V> KeyValueEntrySubstate<V> {
    pub fn lock(&mut self) {
        match self {
            KeyValueEntrySubstate::V1(substate) => {
                substate.lock_status = LockStatus::Locked;
            }
        }
    }

    pub fn into_value(self) -> Option<V> {
        match self {
            KeyValueEntrySubstate::V1(KeyValueEntrySubstateV1 { value, .. }) => value,
        }
    }

    pub fn is_locked(&self) -> bool {
        match self {
            KeyValueEntrySubstate::V1(substate) => {
                matches!(substate.lock_status, LockStatus::Locked)
            }
        }
    }

    pub fn unlocked_entry(value: V) -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: Some(value),
            lock_status: LockStatus::Unlocked,
        })
    }

    pub fn locked_entry(value: V) -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: Some(value),
            lock_status: LockStatus::Locked,
        })
    }

    pub fn locked_empty_entry() -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: None,
            lock_status: LockStatus::Locked,
        })
    }

    pub fn remove(&mut self) -> Option<V> {
        match self {
            KeyValueEntrySubstate::V1(substate) => substate.value.take(),
        }
    }

    pub fn lock_status(&self) -> LockStatus {
        match self {
            KeyValueEntrySubstate::V1(substate) => substate.lock_status.clone(),
        }
    }
}

impl<V> Default for KeyValueEntrySubstate<V> {
    fn default() -> Self {
        Self::V1(KeyValueEntrySubstateV1 {
            value: None,
            lock_status: LockStatus::Unlocked,
        })
    }
}

pub type IndexEntrySubstateV1<V> = V;

// Note - we manually version these instead of using the defined_versioned! macro,
// to avoid IndexEntrySubstate<X> implementing UpgradableVersioned and inheriting
// potentially confusing methods
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IndexEntrySubstate<V> {
    V1(IndexEntrySubstateV1<V>),
}

impl<V> IndexEntrySubstate<V> {
    pub fn entry(value: V) -> Self {
        Self::V1(value)
    }

    pub fn value(&self) -> &V {
        match self {
            IndexEntrySubstate::V1(value) => value,
        }
    }

    pub fn into_value(self) -> V {
        match self {
            IndexEntrySubstate::V1(value) => value,
        }
    }
}

pub type SortedIndexEntrySubstateV1<V> = V;

// Note - we manually version these instead of using the defined_versioned! macro,
// to avoid SortedIndexEntrySubstate<X> implementing UpgradableVersioned and inheriting
// potentially confusing methods
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SortedIndexEntrySubstate<V> {
    V1(SortedIndexEntrySubstateV1<V>),
}

impl<V> SortedIndexEntrySubstate<V> {
    pub fn entry(value: V) -> Self {
        Self::V1(value)
    }

    pub fn value(&self) -> &V {
        match self {
            SortedIndexEntrySubstate::V1(value) => value,
        }
    }

    pub fn into_value(self) -> V {
        match self {
            SortedIndexEntrySubstate::V1(value) => value,
        }
    }
}
