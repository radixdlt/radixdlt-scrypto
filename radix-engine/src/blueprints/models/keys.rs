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
        #[sbor(categorize_types = "")]
        /// This type represents the payload of the key of a particular sorted index collection.
        $vis struct $payload_type_name
            $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                pub $sort_prefix_property_name: u16,
                pub content: $content_type,
            }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            SortedIndexKeyPayload
            for $payload_type_name $(< $( $lt ),+ >)?
        {
            type Content = $content_type;

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
        #[sbor(transparent, categorize_types = "", transparent_name)]
        /// This new type represents the payload of the key of a particular collection.
        $vis struct $payload_type_name
            $(< $( $lt $( : $clt $(+ $dlt )* )? $( = $deflt)? ),+ >)?
            {
                pub content: $content_type,
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
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            $content_trait<$payload_type_name$(< $( $lt ),+ >)?>
            for $content_type
        {
        }
    }
}
#[allow(unused)]
pub(crate) use declare_key_new_type;

/// This trait is intended to be implemented by an explicit new type for for the given
/// `{ content: T }` key of a particular key value collection.
pub trait KeyValueKeyPayload: Sized {
    type Content: KeyValueKeyContentSource<Self>;
}

/// This trait is intended to be implemented by types which embody the content
/// of a key for a particular key value collection.
///
/// Note:
/// * Multiple types might be mappable into the key payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a key for multiple
///   substates
pub trait KeyValueKeyContentSource<Payload: KeyValueKeyPayload>: Sized {}

/// This trait is intended to be implemented by an explicit new type for the given
/// `{ content: T }` key of a particular index collection.
pub trait IndexKeyPayload: Sized {
    type Content: IndexKeyContentSource<Self>;
}

/// This trait is intended to be implemented by types which embody the content
/// of a key for a particular index collection.
///
/// Note:
/// * Multiple types might be mappable into the key payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a key for multiple
///   substates
pub trait IndexKeyContentSource<Payload: IndexKeyPayload>: Sized {}

/// This trait is intended to be implemented by an explicit new type for the given
/// `{ sort_index: u16, content: T }` key for a particular sorted index collection.
pub trait SortedIndexKeyPayload: Sized {
    type Content;

    fn from_sort_key_and_content(sort_key: u16, content: Self::Content) -> Self;
}

/// This trait is intended to be implemented by types which embody the content
/// of a key for a particular sorted index collection.
///
/// Note:
/// * Multiple types might be mappable into the key payload, and so implement this trait
/// * This trait is only one way - from value into content
/// * This trait uses a generic, because the same type might be usable as a key for multiple
///   explicit substates
pub trait SortedIndexKeyContentSource<Payload: SortedIndexKeyPayload>: Sized {}
