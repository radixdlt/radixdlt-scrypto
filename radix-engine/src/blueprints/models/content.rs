use crate::internal_prelude::*;

pub trait FieldContent<FieldPayload: From<Self>>: Sized {
    fn into_locked_substate(self) -> FieldSubstate<FieldPayload> {
        FieldSubstate::new_locked_field(self.into())
    }

    fn into_mutable_substate(self) -> FieldSubstate<FieldPayload> {
        FieldSubstate::new_field(self.into())
    }
}

pub trait KeyContent<KeyPayload: From<Self>>: Sized {
    fn into_key(self) -> KeyPayload {
        self.into()
    }
}

pub trait KeyValueEntryContent<EntryPayload: From<Self>>: Sized {
    fn into_locked_substate(self) -> KeyValueEntrySubstate<EntryPayload> {
        KeyValueEntrySubstate::entry(self.into())
    }

    fn into_mutable_substate(self) -> KeyValueEntrySubstate<EntryPayload> {
        KeyValueEntrySubstate::locked_entry(self.into())
    }
}

pub trait IndexEntryContent<EntryPayload: From<Self>>: Sized {
    fn into_substate(self) -> EntryPayload {
        self.into()
    }
}

pub trait SortedIndexEntryContent<EntryPayload: From<Self>>: Sized {
    fn into_substate(self) -> EntryPayload {
        self.into()
    }
}
