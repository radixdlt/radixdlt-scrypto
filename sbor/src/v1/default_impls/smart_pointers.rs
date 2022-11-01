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

impl<'a, B: ?Sized + 'a + ToOwned + Interpretation + Encode> Encode for Cow<'a, B> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_ref().encode_value(encoder);
    }
}

impl<'a, B: ?Sized + 'a + ToOwned<Owned = O> + Interpretation, O: Decode> Decode for Cow<'a, B> {
    #[inline]
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let value = O::decode_value(decoder)?;
        Ok(Cow::Owned(value))
    }
}

impl<T: Interpretation> Interpretation for Box<T> {
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

impl<T: Encode> Encode for Box<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_ref().encode_value(encoder);
    }
}

impl<T: Decode> Decode for Box<T> {
    #[inline]
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(Box::new(T::decode_value(decoder)?))
    }
}

impl<T: Interpretation> Interpretation for Rc<T> {
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

impl<T: Encode> Encode for Rc<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_ref().encode_value(encoder);
    }
}

impl<T: Decode> Decode for Rc<T> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(Rc::new(T::decode_value(decoder)?))
    }
}

impl<T: Interpretation> Interpretation for RefCell<T> {
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

impl<T: Encode> Encode for RefCell<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.borrow().encode_value(encoder);
    }
}

impl<T: Decode> Decode for RefCell<T> {
    #[inline]
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::decode_value(decoder)?))
    }
}

impl<T: Interpretation> Interpretation for Arc<T> {
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

impl<T: Encode> Encode for Arc<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_ref().encode_value(encoder);
    }
}

impl<T: Decode> Decode for Arc<T> {
    #[inline]
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(Arc::new(T::decode_value(decoder)?))
    }
}

// Mutex doesn't exist in core/alloc
#[cfg(not(feature = "alloc"))]
use crate::rust::sync::Mutex;

#[cfg(not(feature = "alloc"))]
impl<T: Interpretation> Interpretation for Mutex<T> {
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
impl<T: Encode> Encode for Mutex<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.lock().unwrap().encode_value(encoder);
    }
}

#[cfg(not(feature = "alloc"))]
impl<T: Decode> Decode for Mutex<T> {
    #[inline]
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(Mutex::new(T::decode_value(decoder)?))
    }
}