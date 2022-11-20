use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::rc::Rc;
use crate::type_id::*;
use crate::*;

impl<'a, X: CustomTypeId, E: Encoder<X>, B: ?Sized + 'a + ToOwned + Encode<X, E> + TypeId<X>>
    Encode<X, E> for Cow<'a, B>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(B::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E> + TypeId<X>> Encode<X, E> for Box<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(T::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E> + TypeId<X>> Encode<X, E> for Rc<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(T::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_body(encoder)
    }
}

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E> + TypeId<X>> Encode<X, E> for RefCell<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(T::type_id())
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

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X>> Decode<X, D> for RefCell<T> {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::decode_body_with_type_id(decoder, type_id)?))
    }
}
