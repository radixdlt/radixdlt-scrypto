use crate::internal_prelude::*;

macro_rules! declare_payload_new_type {
    (
        content_trait: $content_trait:ident,
        payload_trait: $payload_trait:ident,
        ----
        $(#[$attributes:meta])*
        $vis:vis struct $payload_type_name:ident
            $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
            ($content_type:ty)$(;)?
    ) => {
        $(#[$attributes])*
        #[sbor(transparent)]
        /// This new type represents the payload of a particular field or collection.
        /// It is unique to this particular field/collection.
        ///
        /// Therefore, it can be used to disambiguate if the same content type is used
        /// by different blueprints (e.g. two different versions of the same blueprint)
        $vis struct $payload_type_name
        $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
        {
            content: $content_type
        }

        impl
        $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
        $payload_type_name $(< $( $lt ),+ >)?
        {
            pub fn of(content: $content_type) -> Self {
                Self {
                    content,
                }
            }
        }


        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::From<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn from(value: $content_type) -> Self {
                Self {
                    content: value,
                }
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::AsRef<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn as_ref(&self) -> &$content_type {
                &self.content
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::ops::Deref
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            type Target = $content_type;

            fn deref(&self) -> &Self::Target {
                &self.content
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::convert::AsMut<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn as_mut(&mut self) -> &mut $content_type {
                &mut self.content
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            core::ops::DerefMut
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.content
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $payload_trait
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            type Content = $content_type;

            fn into_content(self) -> Self::Content {
                self.content
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $content_trait<$payload_type_name$(< $( $lt ),+ >)?>
            for $content_type
        {
            fn into_content(self) -> $content_type {
                self
            }
        }
    }
}
use crate::system::system_substates::FieldSubstate;
#[allow(unused)]
pub(crate) use declare_payload_new_type;

/// This trait is intended to be implemented by an explicit new type for for the given
/// `{ content: T }` payload of a particular field.
pub trait FieldPayload:
    Sized + AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: FieldContentSource<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content(content: Self::Content) -> Self {
        Self::from(content)
    }

    fn from_content_source<T: FieldContentSource<Self>>(content: T) -> Self {
        Self::from_content(content.into_content())
    }

    fn into_locked_substate(self) -> FieldSubstate<Self> {
        FieldSubstate::new_locked_field(self)
    }

    fn into_unlocked_substate(self) -> FieldSubstate<Self> {
        FieldSubstate::new_unlocked_field(self)
    }
}

/// This trait is intended to be implemented by types which embody the content
/// of a particular field payload.
///
/// Note:
/// * Multiple types might be mappable into the payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a payload for multiple
///   substates
pub trait FieldContentSource<Payload: FieldPayload>: Sized {
    fn into_content(self) -> Payload::Content;

    fn into_payload(self) -> Payload {
        Payload::from_content_source(self)
    }

    fn into_locked_substate(self) -> FieldSubstate<Payload> {
        self.into_payload().into_locked_substate()
    }

    fn into_unlocked_substate(self) -> FieldSubstate<Payload> {
        self.into_payload().into_unlocked_substate()
    }
}

/// This trait is intended to be implemented by an explicit new type for for the given
/// `{ content: T }` payload of a particular key value collection.
pub trait KeyValueEntryPayload:
    Sized + AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: KeyValueEntryContentSource<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content(inner_content: Self::Content) -> Self {
        Self::from(inner_content)
    }

    fn from_content_source<T: KeyValueEntryContentSource<Self>>(content: T) -> Self {
        Self::from_content(content.into_content())
    }

    fn into_locked_substate(self) -> KeyValueEntrySubstate<Self> {
        KeyValueEntrySubstate::locked_entry(self)
    }

    fn into_unlocked_substate(self) -> KeyValueEntrySubstate<Self> {
        KeyValueEntrySubstate::unlocked_entry(self)
    }
}

/// This trait is intended to be implemented by types which embody the content
/// of a particular key value entry payload.
///
/// Note:
/// * Multiple types might be mappable into the payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a payload for multiple
///   substates
pub trait KeyValueEntryContentSource<Payload: KeyValueEntryPayload>: Sized {
    fn into_content(self) -> Payload::Content;

    fn into_payload(self) -> Payload {
        Payload::from_content_source(self)
    }

    fn into_locked_substate(self) -> KeyValueEntrySubstate<Payload> {
        self.into_payload().into_locked_substate()
    }

    fn into_unlocked_substate(self) -> KeyValueEntrySubstate<Payload> {
        self.into_payload().into_unlocked_substate()
    }
}

/// This trait is intended to be implemented by an explicit new type for for the given
/// `{ content: T }` payload of a particular index collection.
pub trait IndexEntryPayload:
    Sized + AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: IndexEntryContentSource<Self>;

    fn into_content(self) -> Self::Content;
    fn from_content(inner_content: Self::Content) -> Self {
        Self::from(inner_content)
    }

    fn from_content_source<T: IndexEntryContentSource<Self>>(content: T) -> Self {
        Self::from_content(content.into_content())
    }

    fn into_substate(self) -> IndexEntrySubstate<Self> {
        IndexEntrySubstate::entry(self)
    }
}

/// This trait is intended to be implemented by types which embody the content
/// of a particular index entry payload.
///
/// Note:
/// * Multiple types might be mappable into the payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a payload for multiple
///   substates
pub trait IndexEntryContentSource<Payload: IndexEntryPayload>: Sized {
    fn into_content(self) -> Payload::Content;

    fn into_payload(self) -> Payload {
        Payload::from_content_source(self)
    }

    fn into_substate(self) -> IndexEntrySubstate<Payload> {
        self.into_payload().into_substate()
    }
}

/// This trait is intended to be implemented by an explicit new type for for the given
/// `{ content: T }` payload of a particular sorted index collection.
pub trait SortedIndexEntryPayload:
    Sized + AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: SortedIndexEntryContentSource<Self>;

    fn into_content(self) -> Self::Content;

    fn from_content(inner_content: Self::Content) -> Self {
        Self::from(inner_content)
    }

    fn from_content_source<T: SortedIndexEntryContentSource<Self>>(content: T) -> Self {
        Self::from_content(content.into_content())
    }

    fn into_substate(self) -> SortedIndexEntrySubstate<Self> {
        SortedIndexEntrySubstate::entry(self)
    }
}

/// This trait is intended to be implemented by types which embody the content
/// of a particular sorted index entry payload.
///
/// Note:
/// * Multiple types might be mappable into the payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a payload for multiple
///   substates
pub trait SortedIndexEntryContentSource<Payload: SortedIndexEntryPayload>: Sized {
    fn into_content(self) -> Payload::Content;

    fn into_payload(self) -> Payload {
        Payload::from_content_source(self)
    }

    fn into_substate(self) -> SortedIndexEntrySubstate<Payload> {
        self.into_payload().into_substate()
    }
}
