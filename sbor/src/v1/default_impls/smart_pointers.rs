use super::super::*;
use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::rc::Rc;
use crate::rust::cell::RefCell;
use crate::rust::sync::Arc;

impl<'a, B: ?Sized + 'a + ToOwned + Interpretation> Interpretation for Cow<'a, B> {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    #[inline]
    fn get_interpretation(&self) -> u8 {
        self.as_ref().get_interpretation()
    }

    #[inline]
    fn check_interpretation(actual: u8) -> Result<(), DecodeError> {
        B::check_interpretation(actual)
    }
}

impl<'a, E: Encoder, B: ?Sized + 'a + ToOwned + Interpretation + Encode<E>> Encode<E> for Cow<'a, B> {
    #[inline]
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_value(encoder)
    }
}

impl<'a, D: Decoder, B: ?Sized + 'a + ToOwned<Owned = O> + Interpretation, O: Decode<D>> Decode<D> for Cow<'a, B> {
    #[inline]
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        let value = O::decode_value(decoder)?;
        Ok(Cow::Owned(value))
    }
}

impl<T: Interpretation + ?Sized> Interpretation for Box<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    #[inline]
    fn get_interpretation(&self) -> u8 {
        self.as_ref().get_interpretation()
    }

    #[inline]
    fn check_interpretation(actual: u8) -> Result<(), DecodeError> {
        T::check_interpretation(actual)
    }
}

impl<E: Encoder, T: Encode<E> + Interpretation + ?Sized> Encode<E> for Box<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_value(encoder)
    }
}

impl<D: Decoder, T: Decode<D>> Decode<D> for Box<T> {
    #[inline]
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Box::new(T::decode_value(decoder)?))
    }
}

impl<T: Interpretation + ?Sized> Interpretation for Rc<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    #[inline]
    fn get_interpretation(&self) -> u8 {
        self.as_ref().get_interpretation()
    }

    #[inline]
    fn check_interpretation(actual: u8) -> Result<(), DecodeError> {
        T::check_interpretation(actual)
    }
}

impl<E: Encoder, T: Encode<E> + Interpretation + ?Sized> Encode<E> for Rc<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_value(encoder)
    }
}

impl<D: Decoder, T: Decode<D>> Decode<D> for Rc<T> {
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Rc::new(T::decode_value(decoder)?))
    }
}

impl<T: Interpretation + ?Sized> Interpretation for RefCell<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    #[inline]
    fn get_interpretation(&self) -> u8 {
        self.borrow().get_interpretation()
    }

    #[inline]
    fn check_interpretation(actual: u8) -> Result<(), DecodeError> {
        T::check_interpretation(actual)
    }
}

impl<E: Encoder, T: Encode<E> + Interpretation + ?Sized> Encode<E> for RefCell<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.borrow().encode_value(encoder)
    }
}

impl<D: Decoder, T: Decode<D>> Decode<D> for RefCell<T> {
    #[inline]
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::decode_value(decoder)?))
    }
}

impl<T: Interpretation + ?Sized> Interpretation for Arc<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    #[inline]
    fn get_interpretation(&self) -> u8 {
        self.as_ref().get_interpretation()
    }

    #[inline]
    fn check_interpretation(actual: u8) -> Result<(), DecodeError> {
        T::check_interpretation(actual)
    }
}

impl<E: Encoder, T: Encode<E> + Interpretation + ?Sized> Encode<E> for Arc<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode_value(encoder)
    }
}

impl<D: Decoder, T: Decode<D>> Decode<D> for Arc<T> {
    #[inline]
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Arc::new(T::decode_value(decoder)?))
    }
}

// Mutex doesn't exist in core/alloc
#[cfg(not(feature = "alloc"))]
use crate::rust::sync::Mutex;

#[cfg(not(feature = "alloc"))]
impl<T: Interpretation + ?Sized> Interpretation for Mutex<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::NOT_FIXED;

    #[inline]
    fn get_interpretation(&self) -> u8 {
        self.lock().unwrap().get_interpretation()
    }

    #[inline]
    fn check_interpretation(actual: u8) -> Result<(), DecodeError> {
        T::check_interpretation(actual)
    }
}

#[cfg(not(feature = "alloc"))]
impl<E: Encoder, T: Encode<E> + Interpretation + ?Sized> Encode<E> for Mutex<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.lock().unwrap().encode_value(encoder)
    }
}

#[cfg(not(feature = "alloc"))]
impl<D: Decoder, T: Decode<D>> Decode<D> for Mutex<T> {
    #[inline]
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Mutex::new(T::decode_value(decoder)?))
    }
}