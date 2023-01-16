use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::rc::Rc;
use crate::value_kind::*;
use crate::*;

impl<'a, X: CustomValueKind, T: ?Sized + Categorize<X>> Categorize<X> for &T {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<'a, X: CustomValueKind, B: ?Sized + 'a + ToOwned + Categorize<X>> Categorize<X>
    for Cow<'a, B>
{
    #[inline]
    fn value_kind() -> ValueKind<X> {
        B::value_kind()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for Box<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for Rc<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for RefCell<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<'a, X: CustomValueKind, E: Encoder<X>, T: ?Sized + Encode<X, E>> Encode<X, E> for &T {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self).encode_value_kind(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self).encode_body(encoder)
    }
}

impl<'a, X: CustomValueKind, E: Encoder<X>, B: ?Sized + 'a + ToOwned + Encode<X, E>> Encode<X, E>
    for Cow<'a, B>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_value_kind(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for Box<T> {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_value_kind(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for Rc<T> {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_value_kind(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for RefCell<T> {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.borrow().encode_value_kind(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.borrow().encode_body(encoder)
    }
}

impl<
        'a,
        X: CustomValueKind,
        D: Decoder<X>,
        B: ?Sized + 'a + ToOwned<Owned = O>,
        O: Decode<X, D>,
    > Decode<X, D> for Cow<'a, B>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Cow::Owned(O::decode_body_with_value_kind(
            decoder, value_kind,
        )?))
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Box<T> {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Box::new(T::decode_body_with_value_kind(
            decoder, value_kind,
        )?))
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Rc<T> {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Rc::new(T::decode_body_with_value_kind(
            decoder, value_kind,
        )?))
    }
}

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for RefCell<T> {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::decode_body_with_value_kind(
            decoder, value_kind,
        )?))
    }
}

pub use schema::*;

mod schema {
    use super::*;

    wrapped_generic_describe!(T, &T, T);

    impl<'a, C: CustomTypeKind<GlobalTypeId>, B: ?Sized + 'a + ToOwned + Describe<C>> Describe<C>
        for Cow<'a, B>
    {
        const TYPE_ID: GlobalTypeId = <B>::TYPE_ID;

        fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
            <B>::type_data()
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            <B>::add_all_dependencies(aggregator)
        }
    }

    wrapped_generic_describe!(T, Box<T>, T);
    wrapped_generic_describe!(T, Rc<T>, T);
    wrapped_generic_describe!(T, RefCell<T>, T);
}
