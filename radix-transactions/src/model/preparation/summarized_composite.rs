use crate::internal_prelude::*;
use radix_common::constants::*;

pub enum ConcatenatedDigest {}

impl ConcatenatedDigest {
    /// For use when creating a transaction payload
    pub fn prepare_transaction_payload<T: TuplePreparable>(
        decoder: &mut TransactionDecoder,
        discriminator: TransactionDiscriminator,
        header: ExpectedHeaderKind,
    ) -> Result<(T, Summary), PrepareError> {
        let digest = HashAccumulator::new()
            .concat(&[TRANSACTION_HASHABLE_PAYLOAD_PREFIX, discriminator as u8]);
        T::prepare_into_concatenated_digest(
            decoder,
            digest,
            header.with_discriminator(discriminator as u8),
        )
    }

    #[deprecated = "Use prepare_from_sbor_array_value_body instead for new models"]
    pub fn prepare_from_sbor_array_full_value<T: ArrayPreparable>(
        decoder: &mut TransactionDecoder,
        value_type: ValueType,
        max_length: usize,
    ) -> Result<(T, Summary), PrepareError> {
        T::prepare_into_concatenated_digest(
            decoder,
            HashAccumulator::new(),
            value_type,
            max_length,
            true,
        )
    }

    pub fn prepare_from_sbor_array_value_body<T: ArrayPreparable>(
        decoder: &mut TransactionDecoder,
        value_type: ValueType,
        max_length: usize,
    ) -> Result<(T, Summary), PrepareError> {
        T::prepare_into_concatenated_digest(
            decoder,
            HashAccumulator::new(),
            value_type,
            max_length,
            false,
        )
    }

    #[deprecated = "Use prepare_from_sbor_tuple_value_body instead for new models"]
    pub fn prepare_from_sbor_tuple_full_value<T: TuplePreparable>(
        decoder: &mut TransactionDecoder,
    ) -> Result<(T, Summary), PrepareError> {
        T::prepare_into_concatenated_digest(
            decoder,
            HashAccumulator::new(),
            ExpectedTupleHeader::TupleWithValueKind,
        )
    }

    pub fn prepare_from_sbor_tuple_value_body<T: TuplePreparable>(
        decoder: &mut TransactionDecoder,
    ) -> Result<(T, Summary), PrepareError> {
        T::prepare_into_concatenated_digest(
            decoder,
            HashAccumulator::new(),
            ExpectedTupleHeader::TupleNoValueKind,
        )
    }
}

pub trait ArrayPreparable: Sized {
    fn prepare_into_concatenated_digest(
        decoder: &mut TransactionDecoder,
        accumulator: HashAccumulator,
        value_type: ValueType,
        max_length: usize,
        read_value_kind: bool,
    ) -> Result<(Self, Summary), PrepareError>;
}

impl<T: TransactionPreparableFromValueBody> ArrayPreparable for Vec<T> {
    fn prepare_into_concatenated_digest(
        decoder: &mut TransactionDecoder,
        mut accumulator: HashAccumulator,
        value_type: ValueType,
        max_length: usize,
        read_value_kind: bool,
    ) -> Result<(Self, Summary), PrepareError> {
        decoder.track_stack_depth_increase()?;
        let length = if read_value_kind {
            decoder.read_array_header(T::value_kind())?
        } else {
            decoder.read_array_header_without_value_kind(T::value_kind())?
        };

        if length > max_length {
            return Err(PrepareError::TooManyValues {
                value_type,
                actual: length,
                max: max_length,
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
            let prepared = T::prepare_from_value_body(decoder)?;
            effective_length = effective_length
                .checked_add(prepared.get_summary().effective_length)
                .ok_or(PrepareError::LengthOverflow)?;
            total_bytes_hashed = total_bytes_hashed
                .checked_add(prepared.get_summary().total_bytes_hashed)
                .ok_or(PrepareError::LengthOverflow)?;
            accumulator = accumulator.concat(prepared.get_summary().hash);
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
        header: ExpectedTupleHeader,
    ) -> Result<(Self, Summary), PrepareError>;
}

pub enum ExpectedHeaderKind {
    EnumNoValueKind,
    EnumWithValueKind,
    TupleNoValueKind,
    TupleWithValueKind,
}

impl ExpectedHeaderKind {
    pub fn with_discriminator(self, discriminator: u8) -> ExpectedTupleHeader {
        match self {
            Self::EnumNoValueKind => ExpectedTupleHeader::EnumNoValueKind { discriminator },
            Self::EnumWithValueKind => ExpectedTupleHeader::EnumWithValueKind { discriminator },
            Self::TupleNoValueKind => ExpectedTupleHeader::TupleNoValueKind,
            Self::TupleWithValueKind => ExpectedTupleHeader::TupleWithValueKind,
        }
    }
}

pub enum ExpectedTupleHeader {
    EnumNoValueKind { discriminator: u8 },
    EnumWithValueKind { discriminator: u8 },
    TupleWithValueKind,
    TupleNoValueKind,
}

macro_rules! prepare_tuple {
    ($n:tt$( $var_name:ident $type_name:ident)*) => {
        impl<$($type_name: TransactionPreparableFromValue,)*> TuplePreparable for ($($type_name,)*) {
            #[allow(unused_mut)]
            fn prepare_into_concatenated_digest(decoder: &mut TransactionDecoder, mut accumulator: HashAccumulator, header: ExpectedTupleHeader) -> Result<(Self, Summary), PrepareError> {
                decoder.track_stack_depth_increase()?;
                decoder.read_header(header, $n)?;

                // NOTE: We purposefully don't take the effective_length from the size of the SBOR type header
                // This is because the SBOR value header isn't included in the hash...
                // And we want to protect against non-determinism in the effective_length due to a different serializations of the SBOR value header.
                // Whilst we believe the SBOR value header to currently be unique (eg we don't allow trailing bytes in the encoded size) - I'd rather not rely on that.
                // So just assume it's 2 here (1 byte for value kind + 1 byte for length if length sufficiently short)
                // ALSO this makes the length independent of whether it is an enum or tuple
                let mut effective_length = 2usize;
                let mut total_bytes_hashed = 0usize;

                $(
                    let $var_name = <$type_name>::prepare_from_value(decoder)?;
                    effective_length = effective_length.checked_add($var_name.get_summary().effective_length).ok_or(PrepareError::LengthOverflow)?;
                    total_bytes_hashed = total_bytes_hashed.checked_add($var_name.get_summary().total_bytes_hashed).ok_or(PrepareError::LengthOverflow)?;
                    accumulator = accumulator.concat($var_name.get_summary().hash);
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
