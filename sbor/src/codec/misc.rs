use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::rc::Rc;
use crate::type_id::*;
use crate::*;

impl<'a, X: CustomTypeId, B: ?Sized + 'a + ToOwned + Encode<X> + TypeId<X>> Encode<X>
    for Cow<'a, B>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(B::type_id())
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        self.as_ref().encode_body(encoder);
    }
}

impl<X: CustomTypeId, T: Encode<X> + TypeId<X>> Encode<X> for Box<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(T::type_id())
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        self.as_ref().encode_body(encoder);
    }
}

impl<X: CustomTypeId, T: Encode<X> + TypeId<X>> Encode<X> for RefCell<T> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(T::type_id())
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        self.borrow().encode_body(encoder);
    }
}

impl<'a, X: CustomTypeId, D: Decoder<X>, B: ?Sized + 'a + ToOwned<Owned = O>, O: Decode<X, D>>
    Decode<X, D> for Cow<'a, B>
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Cow::Owned(O::decode_body_with_type_id(decoder, type_id)?))
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Box<T> {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Box::new(T::decode_body_with_type_id(decoder, type_id)?))
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Rc<T> {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Rc::new(T::decode_body_with_type_id(decoder, type_id)?))
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X>> Decode<X, D> for RefCell<T> {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::decode_body_with_type_id(decoder, type_id)?))
    }
}
