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

impl<'a, X: CustomTypeId, B: ?Sized + 'a + ToOwned<Owned = O>, O: Decode<X> + TypeId<X>> Decode<X>
    for Cow<'a, B>
{
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, O::type_id())?;
        let v = O::decode_with_type_id(decoder, type_id)?;
        Ok(Cow::Owned(v))
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>> Decode<X> for Box<T> {
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, T::type_id())?;
        let v = T::decode_with_type_id(decoder, type_id)?;
        Ok(Box::new(v))
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>> Decode<X> for Rc<T> {
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, T::type_id())?;
        let v = T::decode_with_type_id(decoder, type_id)?;
        Ok(Rc::new(v))
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>> Decode<X> for RefCell<T> {
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, T::type_id())?;
        let v = T::decode_with_type_id(decoder, type_id)?;
        Ok(RefCell::new(v))
    }
}
