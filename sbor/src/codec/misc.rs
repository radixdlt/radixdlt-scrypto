use crate::internal_prelude::*;
use crate::value_kind::*;
use crate::*;

impl<'a, X: CustomValueKind, T: ?Sized + Categorize<X>> Categorize<X> for &T {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<'a, X: CustomValueKind, T: ?Sized + SborTuple<X>> SborTuple<X> for &'a T {
    fn get_length(&self) -> usize {
        T::get_length(self)
    }
}

impl<'a, X: CustomValueKind, T: ?Sized + SborEnum<X>> SborEnum<X> for &'a T {
    fn get_discriminator(&self) -> u8 {
        T::get_discriminator(self)
    }

    fn get_length(&self) -> usize {
        T::get_length(self)
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

impl<'a, X: CustomValueKind, B: ?Sized + 'a + ToOwned + SborTuple<X>> SborTuple<X> for Cow<'a, B> {
    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<'a, X: CustomValueKind, B: ?Sized + 'a + ToOwned + SborEnum<X>> SborEnum<X> for Cow<'a, B> {
    fn get_discriminator(&self) -> u8 {
        self.as_ref().get_discriminator()
    }

    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for Box<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<'a, X: CustomValueKind, T: SborTuple<X>> SborTuple<X> for Box<T> {
    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<'a, X: CustomValueKind, T: SborEnum<X>> SborEnum<X> for Box<T> {
    fn get_discriminator(&self) -> u8 {
        self.as_ref().get_discriminator()
    }

    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for Rc<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<'a, X: CustomValueKind, T: SborTuple<X>> SborTuple<X> for Rc<T> {
    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<'a, X: CustomValueKind, T: SborEnum<X>> SborEnum<X> for Rc<T> {
    fn get_discriminator(&self) -> u8 {
        self.as_ref().get_discriminator()
    }

    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for Arc<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<'a, X: CustomValueKind, T: SborTuple<X>> SborTuple<X> for Arc<T> {
    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<'a, X: CustomValueKind, T: SborEnum<X>> SborEnum<X> for Arc<T> {
    fn get_discriminator(&self) -> u8 {
        self.as_ref().get_discriminator()
    }

    fn get_length(&self) -> usize {
        self.as_ref().get_length()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for RefCell<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<'a, X: CustomValueKind, T: SborTuple<X>> SborTuple<X> for RefCell<T> {
    fn get_length(&self) -> usize {
        self.borrow().get_length()
    }
}

impl<'a, X: CustomValueKind, T: SborEnum<X>> SborEnum<X> for RefCell<T> {
    fn get_discriminator(&self) -> u8 {
        self.borrow().get_discriminator()
    }

    fn get_length(&self) -> usize {
        self.borrow().get_length()
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

impl<X: CustomValueKind, E: Encoder<X>, T: Encode<X, E>> Encode<X, E> for Arc<T> {
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

impl<X: CustomValueKind, D: Decoder<X>, T: Decode<X, D>> Decode<X, D> for Arc<T> {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        // This sadly won't allow us to decode into `Arc<[X]>` but we can't do that without
        // some form of specialization, or we could create an `ArcSlice<T>` newtype to permit this.
        // For now, we can use `Arc<Vec<u8>>` in these cases, even if it's a double-indirection.
        Ok(Arc::new(T::decode_body_with_value_kind(
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

mod schema {
    use super::*;

    wrapped_generic_describe!(T, &T, T);

    impl<'a, C: CustomTypeKind<RustTypeId>, B: ?Sized + 'a + ToOwned + Describe<C>> Describe<C>
        for Cow<'a, B>
    {
        const TYPE_ID: RustTypeId = <B>::TYPE_ID;

        fn type_data() -> TypeData<C, RustTypeId> {
            <B>::type_data()
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            <B>::add_all_dependencies(aggregator)
        }
    }

    wrapped_generic_describe!(T, Box<T>, T);
    wrapped_generic_describe!(T, Rc<T>, T);
    wrapped_generic_describe!(T, Arc<T>, T);
    wrapped_generic_describe!(T, RefCell<T>, T);
}
