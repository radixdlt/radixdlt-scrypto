use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::rc::Rc;
use crate::type_id::*;
use crate::*;

impl<'a, X: CustomTypeId, E: Encoder<X>, T: ?Sized + Encode<X, E>> Encode<X, E> for &T {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self).encode_type_id(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self).encode_body(encoder)
    }
}

impl<'a, X: CustomTypeId, E: Encoder<X>, B: ?Sized + 'a + ToOwned + Encode<X, E>> Encode<X, E>
    for Cow<'a, B>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_type_id(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for Box<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_type_id(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for Rc<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_type_id(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for RefCell<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.borrow().encode_type_id(encoder)
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.borrow().encode_body(encoder)
    }
}

impl<'a, X: CustomTypeId, D: Decoder<X>, B: ?Sized + 'a + ToOwned<Owned = O>, O: Decode<X, D>>
    Decode<X, D> for Cow<'a, B>
{
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Cow::Owned(O::decode_body_with_type_id(decoder, type_id)?))
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Box<T> {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Box::new(T::decode_body_with_type_id(decoder, type_id)?))
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Rc<T> {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Rc::new(T::decode_body_with_type_id(decoder, type_id)?))
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for RefCell<T> {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::decode_body_with_type_id(decoder, type_id)?))
    }
}

#[cfg(feature = "schema")]
pub use schema::*;

#[cfg(feature = "schema")]
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
