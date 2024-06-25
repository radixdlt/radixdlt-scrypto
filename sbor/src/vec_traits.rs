use crate::{
    internal_prelude::*, CustomExtension, CustomSchema, Decoder as _, Describe, Encoder as _,
    VecDecoder, VecEncoder,
};

pub trait VecEncode<X: CustomValueKind>: for<'a> Encode<X, VecEncoder<'a, X>> {}
impl<X: CustomValueKind, T: for<'a> Encode<X, VecEncoder<'a, X>> + ?Sized> VecEncode<X> for T {}

pub fn vec_encode<E: CustomExtension, T: VecEncode<E::CustomValueKind> + ?Sized>(
    value: &T,
    max_depth: usize,
) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = VecEncoder::<'_, E::CustomValueKind>::new(&mut buf, max_depth);
    encoder.encode_payload(value, E::PAYLOAD_PREFIX)?;
    Ok(buf)
}

pub trait VecDecode<X: CustomValueKind>: for<'a> Decode<X, VecDecoder<'a, X>> {}
impl<X: CustomValueKind, T: for<'a> Decode<X, VecDecoder<'a, X>>> VecDecode<X> for T {}

pub fn vec_decode<E: CustomExtension, T: VecDecode<E::CustomValueKind>>(
    buf: &[u8],
    max_depth: usize,
) -> Result<T, DecodeError> {
    VecDecoder::<'_, E::CustomValueKind>::new(buf, max_depth).decode_payload(E::PAYLOAD_PREFIX)
}

pub trait VecSbor<E: CustomExtension>:
    Categorize<E::CustomValueKind>
    + VecEncode<E::CustomValueKind>
    + VecDecode<E::CustomValueKind>
    + Describe<<E::CustomSchema as CustomSchema>::CustomTypeKind<RustTypeId>>
{
}

impl<E: CustomExtension, T> VecSbor<E> for T
where
    T: Categorize<E::CustomValueKind>,
    T: VecEncode<E::CustomValueKind>,
    T: VecDecode<E::CustomValueKind>,
    T: Describe<<E::CustomSchema as CustomSchema>::CustomTypeKind<RustTypeId>>,
{
}
