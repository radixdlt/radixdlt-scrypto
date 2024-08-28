use crate::internal_prelude::*;
use sbor::*;

/// For new values, prefer using [`SummarizedRawValueBody`].
///
/// For use where the value is:
/// * Only ever serialized as a full SBOR body (with its value kind prefix)
/// * Wants a hash which represents a hash of the full SBOR body in its SBOR-encoding
#[derive(Debug, Clone, Eq, PartialEq)]
#[deprecated = "For new models, prefer SummarizedRawValueBody, which allows the hash to be computed both when it's a full value, OR when it's only a child (e.g. in a vec)"]
pub struct SummarizedRawFullValue<T: ManifestDecode> {
    pub inner: T,
    pub summary: Summary,
}

impl_has_summary!(<T: ManifestDecode> SummarizedRawFullValue<T>);

#[allow(deprecated)]
impl<T: ManifestDecode> TransactionPreparableFromValue for SummarizedRawFullValue<T> {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let start_offset = decoder.get_offset();
        let inner = decoder.decode::<T>()?;
        let end_offset = decoder.get_offset();
        let summary = Summary {
            effective_length: end_offset - start_offset,
            total_bytes_hashed: end_offset - start_offset,
            hash: hash(&decoder.get_slice_with_valid_bounds(start_offset, end_offset)),
        };
        Ok(Self { inner, summary })
    }
}

/// For use where the value is:
/// * Serialized as a full SBOR body (with its value kind prefix)
/// * Wants a hash which represents a hash of the full SBOR body in its SBOR-encoding
/// * Also wants a list of references
#[derive(Debug, Clone, Eq, PartialEq)]
#[deprecated = "For new models, prefer SummarizedRawValueBodyWithReferences, which allows the hash to be computed both when it's a full value, OR when it's only a child (e.g. in a vec)"]
pub struct SummarizedRawFullValueWithReferences<T: ManifestDecode> {
    pub inner: T,
    pub summary: Summary,
    pub references: IndexSet<Reference>,
}

#[allow(deprecated)]
impl<T: ManifestDecode> HasSummary for SummarizedRawFullValueWithReferences<T> {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }

    fn summary_mut(&mut self) -> &mut Summary {
        &mut self.summary
    }
}

#[allow(deprecated)]
impl<T: ManifestDecode> TransactionPreparableFromValue for SummarizedRawFullValueWithReferences<T> {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let start_offset = decoder.get_offset();
        let inner = decoder.decode::<T>()?;
        let end_offset = decoder.get_offset();

        let slice = decoder.get_slice_with_valid_bounds(start_offset, end_offset);
        let references = extract_references(slice, traversal::ExpectedStart::Value);
        let summary = Summary {
            effective_length: end_offset - start_offset,
            total_bytes_hashed: end_offset - start_offset,
            hash: hash(slice),
        };
        Ok(Self {
            inner,
            summary,
            references,
        })
    }
}

/// Similar to [`SummarizedRawValueBody`] For use where the value is:
/// * EITHER exists as a full value OR is contained inside a Vec or Map under its SBOR parent
/// * Wants a hash which represents a hash of all of the bytes in its SBOR-encoding (EXCEPT the possibly missing value kind prefix)
/// * Also wants a list of references
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SummarizedRawValueBodyWithReferences<T: ManifestDecode + ManifestCategorize> {
    pub inner: T,
    pub summary: Summary,
    pub references: IndexSet<Reference>,
}

impl_has_summary!(<T: ManifestDecode + ManifestCategorize> SummarizedRawValueBodyWithReferences<T>);

impl<T: ManifestDecode + ManifestCategorize> TransactionPreparableFromValueBody
    for SummarizedRawValueBodyWithReferences<T>
{
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let start_offset = decoder.get_offset();
        let inner = decoder.decode_deeper_body_with_value_kind::<T>(T::value_kind())?;
        let end_offset = decoder.get_offset();
        let slice = decoder.get_slice_with_valid_bounds(start_offset, end_offset);
        let references =
            extract_references(slice, traversal::ExpectedStart::ValueBody(T::value_kind()));
        let summary = Summary {
            effective_length: end_offset - start_offset,
            total_bytes_hashed: end_offset - start_offset,
            hash: hash(slice),
        };
        Ok(Self {
            inner,
            references,
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        T::value_kind()
    }
}

/// For use where the value is:
/// * EITHER exists as a full value OR is contained inside a Vec or Map under its SBOR parent
/// * Wants a hash which represents a hash of all of the bytes in its SBOR-encoding (EXCEPT the possibly missing value kind prefix)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SummarizedRawValueBody<T: ManifestDecode + ManifestCategorize> {
    pub inner: T,
    pub summary: Summary,
}

impl_has_summary!(<T: ManifestDecode + ManifestCategorize> SummarizedRawValueBody<T>);

impl<T: ManifestDecode + ManifestCategorize> TransactionPreparableFromValueBody
    for SummarizedRawValueBody<T>
{
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let start_offset = decoder.get_offset();
        let inner = decoder.decode_deeper_body_with_value_kind::<T>(T::value_kind())?;
        let end_offset = decoder.get_offset();
        let summary = Summary {
            effective_length: end_offset - start_offset,
            total_bytes_hashed: end_offset - start_offset,
            hash: hash(&decoder.get_slice_with_valid_bounds(start_offset, end_offset)),
        };
        Ok(Self { inner, summary })
    }

    fn value_kind() -> ManifestValueKind {
        T::value_kind()
    }
}

/// For use where the value is:
/// * Contained inside a Vec or Map under its SBOR parent
/// * AND is actually a Vec<u8> itself
/// * AND wants a hash which represents a hash of its underlying raw bytes
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SummarizedRawValueBodyRawBytes {
    pub inner: Vec<u8>,
    pub summary: Summary,
}

impl_has_summary!(SummarizedRawValueBodyRawBytes);

impl TransactionPreparableFromValueBody for SummarizedRawValueBodyRawBytes {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let inner = decoder.decode_deeper_body_with_value_kind::<Vec<u8>>(Self::value_kind())?;

        // NOTE: We purposefully don't take the effective_length from the size of the SBOR type header
        // This is because the SBOR value header isn't included in the hash...
        // And we want to protect against non-determinism in the effective_length due to a different serializations of the SBOR value header.
        // Whilst we believe the SBOR value header to currently be unique (eg we don't allow trailing bytes in the encoded size) - I'd rather not rely on that.
        // So just assume it's 2 here (1 byte for value kind + 1 byte for length if length sufficiently short)
        let effective_length = 2usize;

        let summary = Summary {
            effective_length: effective_length
                .checked_add(inner.len())
                .ok_or(PrepareError::LengthOverflow)?,
            total_bytes_hashed: inner.len(),
            hash: hash(&inner),
        };
        Ok(Self { inner, summary })
    }

    fn value_kind() -> ManifestValueKind {
        Vec::<u8>::value_kind()
    }
}

/// For use where the value is:
/// * Already a hash, and it should be prepared as itself
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawHash {
    pub hash: Hash,
    pub summary: Summary,
}

impl_has_summary!(RawHash);

impl TransactionPreparableFromValueBody for RawHash {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let start_offset = decoder.get_offset();
        let hash = decoder.decode_deeper_body_with_value_kind::<Hash>(Self::value_kind())?;
        let end_offset = decoder.get_offset();
        let summary = Summary {
            effective_length: end_offset - start_offset,
            // It's already been hashed before prepare, so don't count it
            total_bytes_hashed: 0,
            hash,
        };
        Ok(Self { hash, summary })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Array
    }
}
