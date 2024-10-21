use crate::internal_prelude::*;

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

pub fn vec_decode_with_nice_error<
    E: ValidatableCustomExtension<()>,
    T: VecDecode<E::CustomValueKind>
        + Describe<<E::CustomSchema as CustomSchema>::CustomAggregatorTypeKind>,
>(
    buf: &[u8],
    max_depth: usize,
) -> Result<T, String> {
    vec_decode::<E, T>(buf, max_depth)
        .map_err(|err| create_nice_error_following_decode_error::<E, T>(buf, err, max_depth))
}

pub fn create_nice_error_following_decode_error<
    E: ValidatableCustomExtension<()>,
    T: Describe<<E::CustomSchema as CustomSchema>::CustomAggregatorTypeKind>,
>(
    buf: &[u8],
    decode_error: DecodeError,
    max_depth: usize,
) -> String {
    let (local_type_id, schema) = generate_full_schema_from_single_type::<T, E::CustomSchema>();
    let schema = schema.as_unique_version();
    match validate_payload_against_schema::<E, _>(buf, schema, local_type_id, &(), max_depth) {
        Ok(()) => {
            // This case is unexpected. We got a decode error, but it's valid against the schema.
            // In this case, let's just debug-print the DecodeError.
            format!("{decode_error:?}")
        }
        Err(err) => err.error_message(schema),
    }
}

pub trait VecSbor<E: CustomExtension>:
    Categorize<E::CustomValueKind>
    + VecEncode<E::CustomValueKind>
    + VecDecode<E::CustomValueKind>
    + Describe<<E::CustomSchema as CustomSchema>::CustomAggregatorTypeKind>
{
}

impl<E: CustomExtension, T> VecSbor<E> for T
where
    T: Categorize<E::CustomValueKind>,
    T: VecEncode<E::CustomValueKind>,
    T: VecDecode<E::CustomValueKind>,
    T: Describe<<E::CustomSchema as CustomSchema>::CustomAggregatorTypeKind>,
{
}
