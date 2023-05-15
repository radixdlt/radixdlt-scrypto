use crate::*;

/// The `Categorize` trait marks a rust type as having a fixed value kind for SBOR encoding/decoding.
///
/// Most rust types will have a fixed value kind in the SBOR model, and so can implement `Categorize`,
/// but some (such as the SBOR [`Value`][crate::Value]) do not.
///
/// Implementing `Categorize` is required for being able to directly [`Encode`][crate::Encode] / [`Decode`][crate::Decode] any
/// collection containing the rust type - because the value kind is lifted/deduplicated in the encoded payload.
///
/// If a type cannot implement `Categorize`, as a work-around, you can put it into a collection by (eg)
/// wrapping it in a tuple of size 1.
pub trait Categorize<X: CustomValueKind> {
    fn value_kind() -> ValueKind<X>;
}

//=========================
// OTHER MARKER TRAITS
//=========================

/// Marker trait for encoding as an Sbor Tuple
///
/// Note - we add the X so that this can be safely implemented by different Custom macros without clashing
/// Otherwise you couldn't implement ManifestSbor and ScryptoSbor on the same type
pub trait SborTuple<X: CustomValueKind> {
    fn get_length(&self) -> usize;
}

/// Marker trait for encoding as an Sbor Enum
///
/// Note - we add the X so that this can be safely implemented by different Custom macros without clashing
/// Otherwise you couldn't implement ManifestSbor and ScryptoSbor on the same type
pub trait SborEnum<X: CustomValueKind> {
    fn get_length(&self) -> usize;
    fn get_discriminator(&self) -> u8;
}

// Macros for use within this crate
macro_rules! categorize_simple {
    ($type:ty, $value_kind:expr) => {
        impl<X: CustomValueKind> Categorize<X> for $type {
            #[inline]
            fn value_kind() -> ValueKind<X> {
                $value_kind
            }
        }
    };
}

pub(crate) use categorize_simple;

macro_rules! categorize_generic {
    ($type:ty, <$($generic:ident),+>, $value_kind:expr) => {
        impl<X: CustomValueKind, $($generic,)+> Categorize<X> for $type {
            #[inline]
            fn value_kind() -> ValueKind<X> {
                $value_kind
            }
        }
    };
}

pub(crate) use categorize_generic;
