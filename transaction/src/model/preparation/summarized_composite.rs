use crate::internal_prelude::*;
use radix_engine_constants::*;

pub enum ConcatenatedDigest {}

impl ConcatenatedDigest {
    /// For use when creating a transaction payload
    pub fn prepare_from_transaction_payload_enum<T: EnumPreparable>(
        decoder: &mut TransactionDecoder,
        discriminator: TransactionDiscriminator,
    ) -> Result<(T, Summary), PrepareError> {
        let digest = HashAccumulator::new()
            .update(&[TRANSACTION_HASHABLE_PAYLOAD_PREFIX, discriminator as u8]);
        T::prepare_into_concatenated_digest(decoder, digest, discriminator as u8)
    }

    /// Creates a digest which matches `prepare_from_transaction_payload_enum`
    pub fn prepare_from_transaction_child_struct<T: TuplePreparable>(
        decoder: &mut TransactionDecoder,
        discriminator: TransactionDiscriminator,
    ) -> Result<(T, Summary), PrepareError> {
        let digest = HashAccumulator::new()
            .update(&[TRANSACTION_HASHABLE_PAYLOAD_PREFIX, discriminator as u8]);
        T::prepare_into_concatenated_digest(decoder, digest)
    }

    pub fn prepare_from_sbor_array<T: ArrayPreparable, const MAX_LENGTH: usize>(
        decoder: &mut TransactionDecoder,
        accumulator: HashAccumulator,
        value_type: ValueType,
    ) -> Result<(T, Summary), PrepareError> {
        T::prepare_into_concatenated_digest::<MAX_LENGTH>(decoder, accumulator, value_type)
    }

    pub fn prepare_from_sbor_tuple<T: TuplePreparable>(
        decoder: &mut TransactionDecoder,
        accumulator: HashAccumulator,
    ) -> Result<(T, Summary), PrepareError> {
        T::prepare_into_concatenated_digest(decoder, accumulator)
    }

    pub fn prepare_from_sbor_enum<T: EnumPreparable>(
        decoder: &mut TransactionDecoder,
        accumulator: HashAccumulator,
        discriminator: u8,
    ) -> Result<(T, Summary), PrepareError> {
        T::prepare_into_concatenated_digest(decoder, accumulator, discriminator as u8)
    }
}

pub trait ArrayPreparable: Sized {
    fn prepare_into_concatenated_digest<const MAX_LENGTH: usize>(
        decoder: &mut TransactionDecoder,
        accumulator: HashAccumulator,
        value_type: ValueType,
    ) -> Result<(Self, Summary), PrepareError>;
}

impl<T: TransactionChildBodyPreparable> ArrayPreparable for Vec<T> {
    fn prepare_into_concatenated_digest<const MAX_LENGTH: usize>(
        decoder: &mut TransactionDecoder,
        mut accumulator: HashAccumulator,
        value_type: ValueType,
    ) -> Result<(Self, Summary), PrepareError> {
        decoder.track_stack_depth_increase()?;
        let length = decoder.read_array_header(T::value_kind())?;

        if length > MAX_LENGTH {
            return Err(PrepareError::TooManyValues {
                value_type,
                actual: length,
                max: MAX_LENGTH,
            });
        }

        // NOTE: We purposefully don't take the effective_length from the size of the SBOR type header
        // This is because the SBOR value header isn't included in the hash...
        // And we want to protect against non-determinism in the effective_length due to a different serializations of the SBOR value header.
        // Whilst we believe the SBOR value header to currently be unique (eg we don't allow trailing bytes in the encoded size) - I'd rather not rely on that.
        // So just assume it's 2 here (1 byte for value kind + 1 byte for length if length sufficiently short)
        let mut effective_length = 2usize;
        let mut total_bytes_hashed = 0usize;

        let mut all_prepared: Vec<T> = Vec::with_capacity(length);
        for _ in 0..length {
            let prepared = T::prepare_as_inner_body_child(decoder)?;
            effective_length = effective_length
                .checked_add(prepared.get_summary().effective_length)
                .ok_or(PrepareError::LengthOverflow)?;
            total_bytes_hashed = total_bytes_hashed
                .checked_add(prepared.get_summary().total_bytes_hashed)
                .ok_or(PrepareError::LengthOverflow)?;
            accumulator = accumulator.update(prepared.get_summary().hash);
            all_prepared.push(prepared);
        }

        decoder.track_stack_depth_decrease()?;

        total_bytes_hashed = total_bytes_hashed
            .checked_add(accumulator.input_length())
            .ok_or(PrepareError::LengthOverflow)?;

        let summary = Summary {
            effective_length,
            total_bytes_hashed,
            hash: accumulator.finalize(),
        };

        Ok((all_prepared, summary))
    }
}

pub trait TuplePreparable: Sized {
    fn prepare_into_concatenated_digest(
        decoder: &mut TransactionDecoder,
        accumulator: HashAccumulator,
    ) -> Result<(Self, Summary), PrepareError>;
}

pub trait EnumPreparable: Sized {
    fn prepare_into_concatenated_digest(
        decoder: &mut TransactionDecoder,
        accumulator: HashAccumulator,
        expected_discriminator: u8,
    ) -> Result<(Self, Summary), PrepareError>;
}

macro_rules! prepare_tuple {
    ($n:tt$( $var_name:ident $type_name:ident)*) => {
        impl<$($type_name: TransactionFullChildPreparable,)*> TuplePreparable for ($($type_name,)*) {
            #[allow(unused_mut)]
            fn prepare_into_concatenated_digest(decoder: &mut TransactionDecoder, mut accumulator: HashAccumulator) -> Result<(Self, Summary), PrepareError> {
                decoder.track_stack_depth_increase()?;
                decoder.read_struct_header($n)?;

                // NOTE: We purposefully don't take the effective_length from the size of the SBOR type header
                // This is because the SBOR value header isn't included in the hash...
                // And we want to protect against non-determinism in the effective_length due to a different serializations of the SBOR value header.
                // Whilst we believe the SBOR value header to currently be unique (eg we don't allow trailing bytes in the encoded size) - I'd rather not rely on that.
                // So just assume it's 2 here (1 byte for value kind + 1 byte for length if length sufficiently short)
                let mut effective_length = 2usize;
                let mut total_bytes_hashed = 0usize;

                $(
                    let $var_name = <$type_name>::prepare_as_full_body_child(decoder)?;
                    effective_length = effective_length.checked_add($var_name.get_summary().effective_length).ok_or(PrepareError::LengthOverflow)?;
                    total_bytes_hashed = total_bytes_hashed.checked_add($var_name.get_summary().total_bytes_hashed).ok_or(PrepareError::LengthOverflow)?;
                    accumulator = accumulator.update($var_name.get_summary().hash);
                )*

                decoder.track_stack_depth_decrease()?;

                total_bytes_hashed = total_bytes_hashed.checked_add(accumulator.input_length()).ok_or(PrepareError::LengthOverflow)?;

                let summary = Summary {
                    effective_length,
                    total_bytes_hashed,
                    hash: accumulator.finalize(),
                };
                Ok((($($var_name,)*), summary))
            }
        }

        impl<$($type_name: TransactionFullChildPreparable,)*> EnumPreparable for ($($type_name,)*) {
            #[allow(unused_mut)]
            fn prepare_into_concatenated_digest(decoder: &mut TransactionDecoder, mut accumulator: HashAccumulator, expected_discriminator: u8) -> Result<(Self, Summary), PrepareError> {
                decoder.track_stack_depth_increase()?;
                decoder.read_expected_enum_variant_header(expected_discriminator, $n)?;

                // NOTE: We purposefully don't take the effective_length from the size of the SBOR type header
                // This is because the SBOR value header isn't included in the hash...
                // And we want to protect against non-determinism in the effective_length due to a different serializations of the SBOR value header.
                // Whilst we believe the SBOR value header to currently be unique (eg we don't allow trailing bytes in the encoded size) - I'd rather not rely on that.
                // So just assume it's 2 here (1 byte for value kind + 1 byte for length if length sufficiently short)
                let mut effective_length = 2usize;
                let mut total_bytes_hashed = 0usize;

                $(
                    let $var_name = <$type_name>::prepare_as_full_body_child(decoder)?;
                    effective_length = effective_length.checked_add($var_name.get_summary().effective_length).ok_or(PrepareError::LengthOverflow)?;
                    total_bytes_hashed = total_bytes_hashed.checked_add($var_name.get_summary().total_bytes_hashed).ok_or(PrepareError::LengthOverflow)?;
                    accumulator = accumulator.update($var_name.get_summary().hash);
                )*

                decoder.track_stack_depth_decrease()?;

                total_bytes_hashed = total_bytes_hashed.checked_add(accumulator.input_length()).ok_or(PrepareError::LengthOverflow)?;

                let summary = Summary {
                    effective_length,
                    total_bytes_hashed,
                    hash: accumulator.finalize(),
                };
                Ok((($($var_name,)*), summary))
            }
        }
    };
}

prepare_tuple! { 0 }
prepare_tuple! { 1 p0 T0 }
prepare_tuple! { 2 p0 T0 p1 T1 }
prepare_tuple! { 3 p0 T0 p1 T1 p2 T2 }
prepare_tuple! { 4 p0 T0 p1 T1 p2 T2 p3 T3 }
prepare_tuple! { 5 p0 T0 p1 T1 p2 T2 p3 T3 p4 T4 }
prepare_tuple! { 6 p0 T0 p1 T1 p2 T2 p3 T3 p4 T4 p5 T5 }
