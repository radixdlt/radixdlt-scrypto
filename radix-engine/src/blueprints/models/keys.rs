use crate::internal_prelude::*;

macro_rules! declare_key_new_type {
    // First - explicitly support SortedIndex
    (
        content_trait: SortedIndexKeyContentSource,
        payload_trait: SortedIndexKeyPayload,
        full_key_content: {
            full_content_type: $full_content_type:ty,
            sort_prefix_property_name: $sort_prefix_property_name:ident
            $(,)?
        },
        ----
        $(#[$attributes:meta])*
        $vis:vis struct $payload_type_name:ident
            $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
            ($content_type:ty)$(;)?
    ) => {
        $(#[$attributes])*
        /// This type represents the payload of the key of a particular sorted index collection.
        $vis struct $payload_type_name
            $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                pub $sort_prefix_property_name: u16,
                pub content: $content_type,
            }

        impl $payload_type_name {
            pub fn new($sort_prefix_property_name: u16, content: $content_type) -> Self {
                Self {
                    $sort_prefix_property_name,
                    content,
                }
            }

            pub fn sort_prefix(&self) -> u16 {
                self.$sort_prefix_property_name
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
            core::convert::AsMut<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn as_mut(&mut self) -> &mut $content_type {
                &mut self.content
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            SortedIndexKeyPayload
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            type Content = $content_type;
            type FullContent = $full_content_type;

            fn into_sort_key_and_content(self) -> (u16, Self::Content) {
                (self.sort_prefix(), self.content)
            }

            fn as_sort_key_and_content(&self) -> (u16, &Self::Content) {
                (self.sort_prefix(), &self.content)
            }

            fn from_sort_key_and_content(sort_prefix: u16, content: Self::Content) -> Self {
                Self {
                    $sort_prefix_property_name: sort_prefix,
                    content,
                }
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            TryFrom<&SubstateKey>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            type Error = ();

            fn try_from(substate_key: &SubstateKey) -> Result<Self, Self::Error> {
                let (sort_prefix, payload_bytes) = substate_key.for_sorted().ok_or(())?;
                let content = scrypto_decode(payload_bytes).map_err(|_| ())?;
                Ok(Self::from_sort_key_and_content(u16::from_be_bytes(*sort_prefix), content))
            }
        }

        // Note - we assume that both:
        // * SortedIndexKeyContentSource<_Payload>
        // * SortedIndexKeyFullContent<_Payload>
        // are already/manually implemented for $content_type
    };
    (
        content_trait: SortedIndexKeyContentSource,
        payload_trait: SortedIndexKeyPayload,
        ----
        $(#[$attributes:meta])*
        $vis:vis struct $payload_type_name:ident
            $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
            ($content_type:ty)$(;)?
    ) => {
        compile_error!(
            "Make sure to add a `full_key_content: { full_content_type: <ident>, sort_prefix_property_name: <..>}` property after the `key_type` property for SortedIndex collection definitions"
        )
    };
    (
        content_trait: $content_trait:ident,
        payload_trait: $payload_trait:ident,
        full_key_content: $full_key_content:tt,
        ----
        $(#[$attributes:meta])*
        $vis:vis struct $payload_type_name:ident
            $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? $( = $deflt:tt)? ),+ >)?
            ($content_type:ty)$(;)?
    ) => {
        compile_error!(
            "Only add a `full_key_content` property for SortedIndex collection definitions"
        )
    };
    // Then - support the other collection types
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
        #[sbor(transparent, transparent_name)]
        /// This new type represents the payload of the key of a particular collection.
        $vis struct $payload_type_name
            $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                pub content: $content_type,
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
            core::convert::AsMut<$content_type>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            fn as_mut(&mut self) -> &mut $content_type {
                &mut self.content
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            TryFrom<&SubstateKey>
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            type Error = ();

            fn try_from(substate_key: &SubstateKey) -> Result<Self, Self::Error> {
                let key = substate_key.for_map().ok_or(())?;
                scrypto_decode::<Self>(&key).map_err(|_| ())
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
#[allow(unused)]
pub(crate) use declare_key_new_type;

/// This trait is intended to be implemented by an explicit new type for for the given
/// `{ content: T }` key of a particular key value collection.
pub trait KeyValueKeyPayload:
    Sized + AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: KeyValueKeyContentSource<Self>;

    fn into_content(self) -> Self::Content;
    fn from_content(inner_content: Self::Content) -> Self {
        Self::from(inner_content)
    }

    fn from_content_source<T: KeyValueKeyContentSource<Self>>(content: T) -> Self {
        Self::from_content(content.into_content())
    }
}

/// This trait is intended to be implemented by types which embody the content
/// of a key for a particular key value collection.
///
/// Note:
/// * Multiple types might be mappable into the key payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a key for multiple
///   substates
pub trait KeyValueKeyContentSource<Payload: KeyValueKeyPayload>: Sized {
    fn into_content(self) -> Payload::Content;

    fn into_key(self) -> Payload {
        Payload::from_content_source(self)
    }
}

/// This trait is intended to be implemented by an explicit new type for the given
/// `{ content: T }` key of a particular index collection.
pub trait IndexKeyPayload:
    Sized + AsRef<Self::Content> + AsMut<Self::Content> + From<Self::Content>
{
    type Content: IndexKeyContentSource<Self>;

    fn into_content(self) -> Self::Content;
    fn from_content(content: Self::Content) -> Self {
        Self::from(content)
    }

    fn from_content_source<T: IndexKeyContentSource<Self>>(content: T) -> Self {
        Self::from_content(content.into_content())
    }
}

/// This trait is intended to be implemented by types which embody the content
/// of a key for a particular index collection.
///
/// Note:
/// * Multiple types might be mappable into the key payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a key for multiple
///   substates
pub trait IndexKeyContentSource<Payload: IndexKeyPayload>: Sized {
    fn into_content(self) -> Payload::Content;

    fn into_key(self) -> Payload {
        Payload::from_content_source(self)
    }
}

/// This trait is intended to be implemented by an explicit new type for the given
/// `{ sort_index: u16, content: T }` key for a particular sorted index collection.
pub trait SortedIndexKeyPayload: Sized + AsRef<Self::Content> + AsMut<Self::Content> {
    type Content;
    type FullContent: SortedIndexKeyFullContent<Self>;

    fn from_sort_key_and_content(sort_key: u16, content: Self::Content) -> Self;
    fn into_sort_key_and_content(self) -> (u16, Self::Content);
    fn as_sort_key_and_content(&self) -> (u16, &Self::Content);

    fn into_full_content(self) -> Self::FullContent {
        let (sort_key, content) = self.into_sort_key_and_content();
        Self::FullContent::from_sort_key_and_content(sort_key, content)
    }

    fn from_content_source<T: SortedIndexKeyContentSource<Self>>(content: T) -> Self {
        let (sort_key, content) = content.into_sort_key_and_content();
        Self::from_sort_key_and_content(sort_key, content)
    }
}

/// This trait is intended to be implemented by types which embody the content
/// of a key for a particular sorted index collection.
///
/// Note:
/// * Multiple types might be mappable into the key payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a key for multiple
///   explicit substates
pub trait SortedIndexKeyContentSource<Payload: SortedIndexKeyPayload>: Sized {
    fn sort_key(&self) -> u16;
    fn into_content(self) -> Payload::Content;

    fn into_sort_key_and_content(self) -> (u16, Payload::Content) {
        (self.sort_key(), self.into_content())
    }

    fn into_key(self) -> Payload {
        Payload::from_content_source(self)
    }
}

/// This trait is intended to be implemented by the canonical content
/// of a key for a particular sorted index collection.
pub trait SortedIndexKeyFullContent<Payload: SortedIndexKeyPayload>:
    SortedIndexKeyContentSource<Payload>
{
    fn from_sort_key_and_content(sort_key: u16, content: Payload::Content) -> Self;
    fn as_content(&self) -> &Payload::Content;

    fn as_sort_key_and_content(&self) -> (u16, &Payload::Content) {
        (self.sort_key(), self.as_content())
    }
}
