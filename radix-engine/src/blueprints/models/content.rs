use crate::internal_prelude::*;

macro_rules! declare_payload_new_type {
    (
        content_trait: $content_trait:ident,
        payload_trait: $payload_trait:ident,
        $(#[$attributes:meta])*
        $vis:vis struct $payload_type_name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
        ($content_type:ty)$(;)?
    ) => {
        $(#[$attributes])*
        #[sbor(transparent, categorize_types = "")]
        $vis struct $payload_type_name
            $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            (pub $content_type);

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::From<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn from(value: $content_type) -> Self {
                Self(value)
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::AsRef<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn as_ref(&self) -> &$content_type {
                &self.0
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::AsMut<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn as_mut(&mut self) -> &mut $content_type {
                &mut self.0
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $payload_trait
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            type Content = $content_type;

            fn into_content(self) -> Self::Content {
                self.0
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $content_trait<$payload_type_name$(< $( $lt ),+ >)?>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn into_payload(self) -> $payload_type_name$(< $( $lt ),+ >)? {
                self
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $content_trait<$payload_type_name$(< $( $lt ),+ >)?>
            for $content_type
        {
            fn into_payload(self) -> $payload_type_name$(< $( $lt ),+ >)? {
                $payload_type_name(self)
            }
        }
    }
}
#[allow(unused)]
pub(crate) use declare_payload_new_type;

pub trait FieldPayload: AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content> {
    type Content: FieldContent<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content<T: FieldContent<Self>>(content: T) -> Self
    where
        Self: From<T>,
    {
        content.into_payload()
    }
}

pub trait FieldContent<Payload: FieldPayload>: Sized {
    fn into_payload(self) -> Payload;

    fn into_locked_substate(self) -> FieldSubstate<Payload> {
        FieldSubstate::new_locked_field(self.into_payload())
    }

    fn into_mutable_substate(self) -> FieldSubstate<Payload> {
        FieldSubstate::new_field(self.into_payload())
    }
}

pub trait KeyPayload: AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content> {
    type Content: KeyContent<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content<T: KeyContent<Self>>(content: T) -> Self
    where
        Self: From<T>,
    {
        content.into_key()
    }
}

pub trait KeyContent<Payload: KeyPayload>: Sized {
    fn into_payload(self) -> Payload;

    fn into_key(self) -> Payload {
        self.into_payload()
    }
}

pub trait KeyValueEntryPayload:
    AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: KeyValueEntryContent<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content<T: KeyValueEntryContent<Self>>(content: T) -> Self
    where
        Self: From<T>,
    {
        content.into_payload()
    }
}

pub trait KeyValueEntryContent<Payload: KeyValueEntryPayload>: Sized {
    fn into_payload(self) -> Payload;

    fn into_locked_substate(self) -> KeyValueEntrySubstate<Payload> {
        KeyValueEntrySubstate::entry(self.into_payload())
    }

    fn into_mutable_substate(self) -> KeyValueEntrySubstate<Payload> {
        KeyValueEntrySubstate::locked_entry(self.into_payload())
    }
}

pub trait IndexEntryPayload:
    AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: IndexEntryContent<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content<T: IndexEntryContent<Self>>(content: T) -> Self
    where
        Self: From<T>,
    {
        content.into_payload()
    }
}

pub trait IndexEntryContent<Payload: IndexEntryPayload>: Sized {
    fn into_payload(self) -> Payload;

    fn into_substate(self) -> Payload {
        self.into_payload()
    }
}

pub trait SortedIndexEntryPayload:
    AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: SortedIndexEntryContent<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content<T: SortedIndexEntryContent<Self>>(content: T) -> Self
    where
        Self: From<T>,
    {
        content.into_payload()
    }
}

pub trait SortedIndexEntryContent<Payload: SortedIndexEntryPayload>: Sized {
    fn into_payload(self) -> Payload;

    fn into_substate(self) -> Payload {
        self.into_payload()
    }
}
