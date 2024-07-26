use crate::rust::hash::Hash;
use crate::rust::ops::Deref;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

/// A wrapper for unverified bytes which are expected (but not guaranteed to be)
/// encoded bytes for a single SBOR value. Under the surface, [`UnvalidatedRawPayload`] can
/// be an owned [`Vec<u8>`] or a borrowed slice `&'a [u8]` of an underlying encoded payload.
///
/// This type SBOR encodes as a [`Vec<u8>`] of the underlying payload, in _any_ SBOR extension.
/// Use [`RawValue`] if you want to encode as the value.
///
/// Validating can turn this into to a [`RawPayload`] which can be achieved via:
///
/// * Self validation, using the `validate` method which checks the payload is valid SBOR,
///   or using the `validate_against_type` method, which checks both that it is valid SBOR,
///   and that it aligns with a type from a schema.
/// * Confirmed validation, where the payload is validated externally, and then
///   `confirm_validated` is called on the payload.
///
/// A [`RawPayload`] can then optionally be turned into a [`RawValue`] with `into_value()`.
///
/// This type guarantees to wrap a full untrusted payload, unlike [`UnvalidatedRawValue`].
/// Therefore, we are allowed to directly read this full untrusted payload cheaply.
///
/// ## Trait Implementations
///
/// SBOR traits [`Categorize`], [`Encode`], [`Decode`] and [`Describe`] are implemented
/// for all extensions, by encoding the bytes of the untrusted payload as a [`Vec<u8>`].
///
/// [`Hash`], [`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`] are all implemented with respect to
/// the byte representation of the untrusted payload.
#[derive(Debug, Clone, Sbor)]
#[sbor(
    as_type = "Cow<'a, [u8]>",
    as_ref = "&self.0",
    from_value = "Self::from_payload_cow(value)",
    child_types = "",
    transparent_name, // This means Describe will not create a new type and just use Vec<u8> (via Cow)
)]
pub struct UnvalidatedRawPayload<'a, E: CustomExtension>(Cow<'a, [u8]>, PhantomData<E>);

// Lots of manual trait implementations because the automated derivations
// are all conditional on E implementing stuff
impl<'a, E: CustomExtension> PartialEq for UnvalidatedRawPayload<'a, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, E: CustomExtension> Eq for UnvalidatedRawPayload<'a, E> {}

impl<'a, E: CustomExtension> Hash for UnvalidatedRawPayload<'a, E> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a, E: CustomExtension> PartialOrd for UnvalidatedRawPayload<'a, E> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, E: CustomExtension> Ord for UnvalidatedRawPayload<'a, E> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<'a, E: CustomExtension> UnvalidatedRawPayload<'a, E> {
    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    pub fn from_payload_slice(payload_slice: &'a [u8]) -> Self {
        Self(Cow::Borrowed(payload_slice), PhantomData)
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    pub fn from_payload(payload_vec: Vec<u8>) -> Self {
        Self(Cow::Owned(payload_vec), PhantomData)
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    pub fn from_payload_cow(payload_cow: Cow<'a, [u8]>) -> Self {
        Self(payload_cow, PhantomData)
    }

    pub fn unvalidated_payload_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn unvalidated_payload_len(&self) -> usize {
        self.0.as_ref().len()
    }

    pub fn into_value(self) -> UnvalidatedRawValue<'a, E> {
        match self.0 {
            Cow::Borrowed(payload_slice) => UnvalidatedRawValue::from_payload_slice(payload_slice),
            Cow::Owned(payload_vec) => UnvalidatedRawValue::from_payload(payload_vec),
        }
    }

    pub fn as_value(&self) -> UnvalidatedRawValue<E> {
        UnvalidatedRawValue::from_payload_slice(self.0.as_ref())
    }

    pub fn traverser(&self, max_depth: usize) -> VecTraverser<E::CustomTraversal> {
        let slice = self.unvalidated_payload_slice();
        let expected_start = ExpectedStart::PayloadPrefix(E::PAYLOAD_PREFIX);
        VecTraverser::<E::CustomTraversal>::new(slice, max_depth, expected_start, true)
    }

    pub fn typed_traverser<'b, 's>(
        &'b self,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
    ) -> TypedTraverser<'b, 's, E> {
        let slice = self.unvalidated_payload_slice();
        let expected_start = ExpectedStart::PayloadPrefix(E::PAYLOAD_PREFIX);
        TypedTraverser::new(slice, schema, type_id, max_depth, expected_start, true)
    }

    /// Uses the default max depth for the given extension.
    pub fn validate(self) -> Result<RawPayload<'a, E>, DecodeError> {
        self.validate_with_max_depth(E::DEFAULT_DEPTH_LIMIT)
    }

    pub fn validate_with_max_depth(
        self,
        max_depth: usize,
    ) -> Result<RawPayload<'a, E>, DecodeError> {
        self.traverser(max_depth).traverse_to_end()?;
        Ok(self.confirm_validated())
    }

    /// Uses the default max depth from the extension.
    /// If you want a custom max depth, use `validate_against_type_with_max_depth`.
    pub fn validate_against_type<'b, 's, T>(
        self,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<RawPayload<'a, E>, LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        self.validate_against_type_with_max_depth(E::DEFAULT_DEPTH_LIMIT, schema, type_id, context)
    }

    pub fn validate_against_type_with_max_depth<'b, 's, T>(
        self,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<RawPayload<'a, E>, LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        validate_typed_traverser(
            &mut self.typed_traverser(max_depth, schema, type_id),
            context,
        )?;
        Ok(self.confirm_validated())
    }

    /// By calling this you confirm that you know the underlying payload is valid,
    /// which means the payload is a valid SBOR encoding under extension `E`
    ///
    /// Failing to do so can cause a panic now, or much later down the line.
    pub fn confirm_validated(self) -> RawPayload<'a, E> {
        match self.0 {
            Cow::Borrowed(payload_slice) => RawPayload::from_valid_payload_slice(payload_slice),
            Cow::Owned(payload_vec) => RawPayload::from_valid_payload(payload_vec),
        }
    }
}

/// A wrapper for encoded bytes of a valid single SBOR value. Under the surface, a
/// [`RawPayload`] can be an owned [`Vec<u8>`] or a borrowed slice `&'a [u8]` of an
/// underlying encoded payload.
///
/// This type SBOR encodes as a [`Vec<u8>`] of the underlying payload, in _any_ SBOR extension.
/// Use [`RawValue`] if you want to encode as the value. A [`RawPayload`] can be cheaply turned into
/// a [`RawValue`] with `into_value()`.
///
/// If the bytes are not trusted yet to be valid SBOR, use [`UnvalidatedRawPayload`] or
/// [`UnvalidatedRawValue`] instead.
///
/// ## Conversion between forms
///
/// The following conversions are all useful:
/// * `as_payload_ref<'b>(&'b self) > RawPayload<'b, E>` - Cheaply creates an owned `RawPayload`,
///   referring to a slice of the underlying bytes from `&self`.
/// * `into_owned(self) > RawPayload<'static, E>` - Converts the underlying bytes into an owned
///    `Vec<u8>` - either moving out of `self` if possible, if not, creating a `Vec<u8>` of the
///    whole payload.
/// * `ref_into_owned(&self) > RawPayload<'static, E>` - Converts from a reference to a new owned
///    value with owned underlying bytes (moving out of `self` if possible).
/// * `as_value<'b>(&'b self) -> RawValue<'b, E>` - Cheaply creates an owned `RawValue`,
///   referring to a slice of the underlying bytes from `&self`.
/// * `into_value(self) -> RawValue<'a, E>` - Converts from a `RawPayload` into a `RawValue`.
/// * `into_unvalidated(self) -> UnvalidatedRawPayload<'a, E>` - Forgets that the value has already
///    been validated, moving self.
/// * `as_unvalidated<'b>(&'b self) -> UnvalidatedRawPayload<'b, E>` - Forgets that the value has already
///    been validated, taking a reference to the underlying value. Useful for temporary validation
///    without moving the value.
///
/// ## Trait implementations
///
/// SBOR traits [`Categorize`], [`Encode`], [`Decode`] and [`Describe`] are implemented
/// for all extensions, by encoding the bytes of the untrusted payload as a [`Vec<u8>`].
///
/// [`Hash`], [`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`] are all implemented with respect to
/// the byte representation of the payload.
#[derive(Debug, Clone, Sbor)]
#[sbor(
    as_type = "Cow<'a, [u8]>",
    as_ref = "&self.0",
    from_value = "Self::from_valid_payload_cow(value)",
    child_types = "",
    transparent_name, // This means Describe will not create a new type and just use Vec<u8> (via Cow)
)]
pub struct RawPayload<'a, E: CustomExtension>(Cow<'a, [u8]>, PhantomData<E>);

// Lots of manual trait implementations because the automated derivations
// are all conditional on E implementing stuff
impl<'a, E: CustomExtension> PartialEq for RawPayload<'a, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, E: CustomExtension> Eq for RawPayload<'a, E> {}

impl<'a, E: CustomExtension> Hash for RawPayload<'a, E> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a, E: CustomExtension> PartialOrd for RawPayload<'a, E> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, E: CustomExtension> Ord for RawPayload<'a, E> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<'a, E: CustomExtension> RawPayload<'a, E> {
    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_payload_slice(payload_slice: &'a [u8]) -> Self {
        Self(Cow::Borrowed(payload_slice), PhantomData)
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_payload(payload_bytes: Vec<u8>) -> Self {
        Self(Cow::Owned(payload_bytes), PhantomData)
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_payload_cow(payload_cow: Cow<'a, [u8]>) -> Self {
        Self(payload_cow, PhantomData)
    }

    pub fn from_value(value: RawValue<'a, E>) -> Self {
        value.into_payload()
    }

    pub fn unit() -> Self {
        Self::from_valid_payload(vec![
            E::PAYLOAD_PREFIX,
            ValueKind::<E::CustomValueKind>::Tuple.as_u8(),
            0,
        ])
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.deref()
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_owned()
    }

    pub fn as_payload_ref<'b>(&'b self) -> RawPayload<'b, E> {
        RawPayload::from_valid_payload_slice(self.as_slice())
    }

    pub fn ref_into_owned(&self) -> RawPayload<'static, E> {
        self.as_payload_ref().into_owned()
    }

    pub fn into_owned(self) -> RawPayload<'static, E> {
        RawPayload::from_valid_payload(self.into_bytes())
    }

    pub fn as_value<'b>(&'b self) -> RawValue<'b, E> {
        RawValue::from_valid_payload_slice(self.as_slice())
    }

    pub fn into_value(self) -> RawValue<'a, E> {
        RawValue(RawValueContent::from_payload_cow(self.0))
    }

    pub fn as_unvalidated(&self) -> UnvalidatedRawPayload<E> {
        UnvalidatedRawPayload::from_payload_slice(self.as_slice())
    }

    pub fn into_unvalidated(self) -> UnvalidatedRawPayload<'a, E> {
        UnvalidatedRawPayload::from_payload(self.into_bytes())
    }

    pub fn decode_as<T: for<'b> Decode<E::CustomValueKind, VecDecoder<'b, E::CustomValueKind>>>(
        &self,
    ) -> Result<T, DecodeError> {
        self.decode_as_with_depth_limit(E::DEFAULT_DEPTH_LIMIT)
    }

    pub fn decode_as_with_depth_limit<
        T: for<'b> Decode<E::CustomValueKind, VecDecoder<'b, E::CustomValueKind>>,
    >(
        &self,
        depth_limit: usize,
    ) -> Result<T, DecodeError> {
        VecDecoder::new(self.as_slice(), depth_limit).decode_payload(E::PAYLOAD_PREFIX)
    }

    pub fn traverser(&self, max_depth: usize) -> VecTraverser<E::CustomTraversal> {
        let slice = self.as_slice();
        let expected_start = ExpectedStart::PayloadPrefix(E::PAYLOAD_PREFIX);
        VecTraverser::<E::CustomTraversal>::new(slice, max_depth, expected_start, true)
    }

    pub fn typed_traverser<'s>(
        &self,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
    ) -> TypedTraverser<'_, 's, E> {
        let slice = self.as_slice();
        let expected_start = ExpectedStart::PayloadPrefix(E::PAYLOAD_PREFIX);
        TypedTraverser::new(slice, schema, type_id, max_depth, expected_start, true)
    }

    pub fn validate_against_type_with_max_depth<'b, 's, T>(
        &'b self,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<(), LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        validate_typed_traverser(
            &mut self.typed_traverser(max_depth, schema, type_id),
            context,
        )
    }

    /// Uses the default max depth from the extension.
    /// If you want a custom max depth, use `validate_against_type_with_max_depth`.
    pub fn validate_against_type<'b, 's, T>(
        &'b self,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<(), LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        self.validate_against_type_with_max_depth(E::DEFAULT_DEPTH_LIMIT, schema, type_id, context)
    }
}

/// A wrapper for unvalidated bytes which are expected to be encoded bytes for a single SBOR value,
/// but the bytes may not be valid SBOR.
///
/// The validation process is marked by conversion into a [`RawValue`], which can either be done via:
///
/// * Self validation, using the `validate` method which checks the payload is valid SBOR,
///   or using the `validate_against_type` method, which checks both that it is valid SBOR,
///   and that it aligns with a type from a schema.
/// * Confirmed validation, where the payload is validated externally, and then
///   `confirm_validated` is called on the payload.
///
/// Under the surface, [`UnvalidatedRawValue`] can an owned [`Vec<u8>`] or a borrowed slice
/// `&'a [u8]` of an underlying encoded payload, value body, or value. The actual underlying
/// value is stored for efficient conversions out of this type.
///
/// ## Trait Implementations
///
/// Very few traits are implemented, because we have little trust in the underlying value /
/// structure.
///
/// If you want [`Hash`], [`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`] on the underlying
/// untrusted bytes, try using an [`UnvalidatedRawPayload`] instead, which only accepts a full
/// payload slice or `Vec`. This can freely be converted into an [`UnvalidatedRawValue`] later.
///
/// If you want SBOR traits implemented to encode as the underlying value, you need to
/// validate the value to a [`RawValue`].
///
/// If you want the SBOR traits implemented to encode as a [`Vec<u8>`] of the Payload, you
/// should use [`UnvalidatedRawPayload`] instead.
#[derive(Debug, Clone)]
pub struct UnvalidatedRawValue<'a, E: CustomExtension>(RawValueContent<'a, E>);

impl<'a, E: CustomExtension> UnvalidatedRawValue<'a, E> {
    pub fn from_payload_slice(payload_slice: &'a [u8]) -> Self {
        Self(RawValueContent::PayloadSlice(payload_slice))
    }

    pub fn from_payload(payload_bytes: Vec<u8>) -> Self {
        Self(RawValueContent::OwnedPayload(payload_bytes))
    }

    pub fn from_value_slice(value_slice: &'a [u8]) -> Self {
        Self(RawValueContent::ValueSlice(value_slice))
    }

    pub fn from_value(value_vec: Vec<u8>) -> Self {
        Self(RawValueContent::OwnedValue(value_vec))
    }

    pub fn from_value_body_slice(
        value_kind: ValueKind<E::CustomValueKind>,
        value_body_slice: &'a [u8],
    ) -> Self {
        Self(RawValueContent::ValueBodySlice(
            value_kind,
            value_body_slice,
        ))
    }

    pub fn from_value_body(
        value_kind: ValueKind<E::CustomValueKind>,
        value_body_vec: Vec<u8>,
    ) -> Self {
        Self(RawValueContent::OwnedValueBody(value_kind, value_body_vec))
    }

    /// Cheaply returns an owned [`UnvalidatedRawValue`] which internally is a reference to the underlying content.
    pub fn as_value_ref(&self) -> UnvalidatedRawValue<E> {
        UnvalidatedRawValue(self.0.as_content_ref())
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    pub fn traverser(
        &self,
        parent_depth: usize,
        max_depth: usize,
    ) -> VecTraverser<E::CustomTraversal> {
        self.0.traverser(max_depth - parent_depth)
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    pub fn typed_traverser<'b, 's>(
        &'b self,
        parent_depth: usize,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
    ) -> TypedTraverser<'b, 's, E> {
        self.0
            .typed_traverser(max_depth - parent_depth, schema, type_id)
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    ///
    /// Uses the default max depth for the given extension.
    pub fn validate(self, parent_depth: usize) -> Result<RawValue<'a, E>, DecodeError> {
        self.validate_with_max_depth(parent_depth, E::DEFAULT_DEPTH_LIMIT)
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    pub fn validate_with_max_depth(
        self,
        parent_depth: usize,
        max_depth: usize,
    ) -> Result<RawValue<'a, E>, DecodeError> {
        self.traverser(parent_depth, max_depth).traverse_to_end()?;
        Ok(self.confirm_validated())
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    pub fn validate_against_type_with_max_depth<'b, 's, T>(
        self,
        parent_depth: usize,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<RawValue<'a, E>, LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        self.0.payload_len().ok_or(LocatedValidationError {
            error: PayloadValidationError::TraversalError(TypedTraversalError::DecodeError(
                DecodeError::PayloadTooLong,
            )),
            location: FullLocation {
                start_offset: usize::MAX,
                end_offset: usize::MAX,
                ancestor_path: vec![],
                current_value_info: None,
            },
        })?;
        validate_typed_traverser(
            &mut self.typed_traverser(parent_depth, max_depth, schema, type_id),
            context,
        )?;
        Ok(self.confirm_validated())
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    ///
    /// This method assumes that that the type should be validated with the default max depth
    /// for the extension. If you want a custom max depth, use `validate_against_type_with_max_depth`.
    pub fn validate_against_type<'b, 's, T>(
        self,
        parent_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<RawValue<'a, E>, LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        self.validate_against_type_with_max_depth(
            parent_depth,
            E::DEFAULT_DEPTH_LIMIT,
            schema,
            type_id,
            context,
        )
    }

    /// By calling this you confirm that you know the underlying payload is valid,
    /// which means:
    /// * The full (or partial) payload is a valid SBOR encoding under extension `E`
    /// * The full payload length is <= usize
    ///
    /// Failing to do so can cause a panic now, or much later down the line.
    pub fn confirm_validated(self) -> RawValue<'a, E> {
        RawValue(self.0)
    }
}

/// A wrapper for encoded bytes of a valid single SBOR value.
///
/// [`RawValue`] is SBOR-encoded as the underlying value. This is unlike [`RawPayload`], which
/// is SBOR-encoded as a byte array of the encoded payload.
///
/// Under the surface, [`RawValue`] can an owned [`Vec<u8>`] or a borrowed slice `&'a [u8]` of
/// an underlying encoded payload, value body, or value. The actual underlying value is stored
/// for efficient conversions out of this type. For example, it is zero-copy to decode into a
/// [`RawValue`] and then turn it into payload bytes.
///
/// If the bytes are not trusted yet to be valid SBOR, use [`UnvalidatedRawPayload`] or
/// [`UnvalidatedRawValue`] instead.
///
/// ## Conversion between forms
///
/// The following are common conversions between different forms:
/// * `as_value_ref<'b>(&'b self) > RawValue<'b, E>` - Cheaply creates an owned `RawValue`,
///   referring to a slice of the underlying bytes from `&self`.
/// * `into_owned(self) > RawValue<'static, E>` - Converts the underlying bytes into an owned
///    `Vec<u8>` - either moving out of `self` if possible, if not, creating a `Vec<u8>` of the
///    whole payload.
/// * `ref_into_owned(&self) > RawValue<'static, E>` - Converts from a reference to a new owned
///    value with owned underlying bytes (moving out of `self` if possible).
/// * `as_payload<'b>(&'b self) -> RawPayload<'b, E>` - Cheaply creates an owned `RawPayload`,
///   referring to a slice of the underlying bytes from `&self`.
/// * `into_payload(self) -> RawPayload<'a, E>` - Converts from a `RawValue` into a `RawPayload`.
/// * `into_owned_payload(self) -> RawPayload<'static, E>` - Converts to a `RawPayload` wrapping
///    a `Vec<u8>` of the full payload.
/// * `into_unvalidated(self) -> UnvalidatedRawValue<'a, E>` - Forgets that the value has already
///    been validated, moving self.
/// * `as_unvalidated<'b>(&'b self) -> UnvalidatedRawValue<'b, E>` - Forgets that the value has already
///    been validated, taking a reference to the underlying value. Useful for temporary validation
///    without moving the value.
///
/// ## Trait implementations
///
/// SBOR traits [`Encode`], [`Decode`] and [`Describe`] are implemented for the extension `E`:
/// * Encoding is direct, as the bytes are known to be valid SBOR.
/// * Decoding goes through a traverser, which calculates the length of the bytes and ensures
///   that the bytes are valid.
/// * Describe is implemented as Any.
///
/// But SBOR [`Categorize`] can't be implemented, because we can't guarantee the value kind is constant.
/// This means [`RawValue`] can't be put as an immediate child to a Vec or Map. You could consider
/// serializing them opaquely as a [`Vec<u8>`] by converting them to [`RawPayload`]s, if you wish.
///
/// [`Hash`], [`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`] are all implemented with respect to
/// the effective underlying byte representation of the payload.
#[derive(Debug, Clone)]
pub struct RawValue<'a, E: CustomExtension>(RawValueContent<'a, E>);

impl<'a, E: CustomExtension> RawValue<'a, E> {
    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_payload_slice(payload_slice: &'a [u8]) -> Self {
        Self(RawValueContent::from_payload_slice(payload_slice))
    }

    /// The bytes should include the prefix byte (eg 0x5b for basic SBOR).
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR payload for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_payload(payload_vec: Vec<u8>) -> Self {
        Self(RawValueContent::from_payload(payload_vec))
    }

    /// The bytes should include the value kind byte, but not the payload prefix byte.
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR value for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_value_slice(value_slice: &'a [u8]) -> Self {
        Self(RawValueContent::from_value_slice(value_slice))
    }

    /// The bytes should include the value kind byte, but not the payload prefix byte.
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR value for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_value(value_vec: Vec<u8>) -> Self {
        Self(RawValueContent::from_value(value_vec))
    }

    /// The bytes should include the value, not the value kind or the payload prefix byte.
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR value body for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_value_body_slice(
        value_kind: ValueKind<E::CustomValueKind>,
        value_body_slice: &'a [u8],
    ) -> Self {
        Self(RawValueContent::from_value_body_slice(
            value_kind,
            value_body_slice,
        ))
    }

    /// The bytes should include the value, not the value kind or the payload prefix byte.
    ///
    /// It is the caller's responsibility to ensure that a valid SBOR value body for extension E is
    /// passed to the caller. Otherwise, this constructor or later operations can panic.
    pub fn from_valid_value_body(
        value_kind: ValueKind<E::CustomValueKind>,
        value_body_vec: Vec<u8>,
    ) -> Self {
        Self(RawValueContent::from_value_body(value_kind, value_body_vec))
    }

    pub fn as_unvalidated(&self) -> UnvalidatedRawValue<'_, E> {
        UnvalidatedRawValue(self.0.as_content_ref())
    }

    pub fn into_unvalidated(self) -> UnvalidatedRawValue<'a, E> {
        UnvalidatedRawValue(self.0)
    }

    /// Cheaply returns an owned [`RawValue`] which internally is a reference to the underlying content.
    pub fn as_value_ref(&self) -> RawValue<E> {
        RawValue(self.0.as_content_ref())
    }

    pub fn ref_into_owned(&self) -> RawValue<'static, E> {
        self.as_value_ref().into_owned()
    }

    pub fn into_owned(self) -> RawValue<'static, E> {
        let content = match self.0 {
            RawValueContent::OwnedPayload(vec) => RawValueContent::OwnedPayload(vec),
            RawValueContent::OwnedValue(vec) => RawValueContent::OwnedValue(vec),
            RawValueContent::OwnedValueBody(value_kind, vec) => {
                RawValueContent::OwnedValueBody(value_kind, vec)
            }
            // For any of the slices, we have to allocate into a vec.
            // If so, we may as well allocate a full payload, for maximum utility
            _ => RawValueContent::OwnedPayload(self.into_payload_bytes()),
        };
        RawValue(content)
    }

    pub fn as_payload<'b>(&'b self) -> RawPayload<'b, E> {
        RawPayload::from_valid_payload_cow(self.as_payload_cow())
    }

    pub fn into_payload(self) -> RawPayload<'a, E> {
        RawPayload::from_valid_payload_cow(self.into_payload_cow())
    }

    pub fn into_owned_payload(self) -> RawPayload<'static, E> {
        RawPayload::from_valid_payload(self.into_payload_bytes())
    }

    fn as_payload_cow(&self) -> Cow<'_, [u8]> {
        match &self.0 {
            // Explicitly handle the owned payload case for efficiency
            RawValueContent::OwnedPayload(payload_bytes) => Cow::Borrowed(payload_bytes),
            RawValueContent::PayloadSlice(payload_slice) => Cow::Borrowed(payload_slice),
            // Otherwise we have to create a new raw payload
            _ => Cow::Owned(self.construct_payload_vec()),
        }
    }

    fn into_payload_cow(self) -> Cow<'a, [u8]> {
        match self.0 {
            // Explicitly handle the owned payload case for efficiency
            RawValueContent::OwnedPayload(payload_bytes) => Cow::Owned(payload_bytes),
            RawValueContent::PayloadSlice(payload_slice) => Cow::Borrowed(payload_slice),
            // Otherwise we have to create a new raw payload
            _ => Cow::Owned(self.construct_payload_vec()),
        }
    }

    pub fn into_payload_bytes(self) -> Vec<u8> {
        match self.0 {
            // Explicitly handle the owned payload case for efficiency
            RawValueContent::OwnedPayload(payload_bytes) => payload_bytes,
            // Otherwise we have to create a new raw payload
            _ => self.construct_payload_vec(),
        }
    }

    fn construct_payload_vec(&self) -> Vec<u8> {
        let body_bytes = self.value_body();
        let mut vec = Vec::with_capacity(self.payload_len());
        vec.push(E::PAYLOAD_PREFIX);
        vec.push(self.value_kind().as_u8());
        vec.extend_from_slice(body_bytes);
        vec
    }

    pub fn decode_as_with_depth_limit<
        T: for<'b> Decode<E::CustomValueKind, VecDecoder<'b, E::CustomValueKind>>,
    >(
        &self,
        depth_limit: usize,
    ) -> Result<T, DecodeError> {
        VecDecoder::new(self.value_body(), depth_limit)
            .decode_deeper_body_with_value_kind(self.value_kind())
    }

    pub fn decode_as<T: for<'b> Decode<E::CustomValueKind, VecDecoder<'b, E::CustomValueKind>>>(
        &self,
    ) -> Result<T, DecodeError> {
        VecDecoder::new(self.value_body(), E::DEFAULT_DEPTH_LIMIT)
            .decode_deeper_body_with_value_kind(self.value_kind())
    }

    pub fn unit() -> Self {
        let value_body = vec![0]; // Length = 0
        Self::from_valid_value_body(ValueKind::Tuple, value_body)
    }

    pub fn value_kind(&self) -> ValueKind<E::CustomValueKind> {
        // Unwrap is safe because we're validated now.
        self.0.value_kind().unwrap()
    }

    pub fn payload_len(&self) -> usize {
        // Unwrap is safe because we're validated now.
        self.0.payload_len().unwrap()
    }

    pub fn value_body(&self) -> &[u8] {
        match &self.0 {
            RawValueContent::PayloadSlice(slice) => &slice[2..],
            RawValueContent::ValueSlice(slice) => &slice[1..],
            RawValueContent::ValueBodySlice(_, slice) => slice,
            RawValueContent::OwnedPayload(vec) => &vec[2..],
            RawValueContent::OwnedValue(vec) => &vec[1..],
            RawValueContent::OwnedValueBody(_, vec) => &vec,
        }
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    pub fn traverser(
        &self,
        parent_depth: usize,
        max_depth: usize,
    ) -> VecTraverser<E::CustomTraversal> {
        self.0.traverser(max_depth - parent_depth)
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    pub fn typed_traverser<'b, 's>(
        &'b self,
        parent_depth: usize,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
    ) -> TypedTraverser<'b, 's, E> {
        self.0
            .typed_traverser(max_depth - parent_depth, schema, type_id)
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    ///
    /// This method assumes that that the type should be validated with the default max depth
    /// for the extension. If you want a custom max depth, use `validate_against_type_with_max_depth`.
    pub fn validate_against_type<'b, 's, T>(
        &'b self,
        parent_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<(), LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        self.validate_against_type_with_max_depth(
            parent_depth,
            E::DEFAULT_DEPTH_LIMIT,
            schema,
            type_id,
            context,
        )
    }

    /// For values which are contained within a larger value, the `parent_depth` is the
    /// current depth before entering the current value. For root values, `parent_depth` should be 0.
    pub fn validate_against_type_with_max_depth<'b, 's, T>(
        &'b self,
        parent_depth: usize,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<(), LocatedValidationError<'s, E>>
    where
        E: ValidatableCustomExtension<T>,
    {
        validate_typed_traverser(
            &mut self.typed_traverser(parent_depth, max_depth, schema, type_id),
            context,
        )
    }
}

/// An underlying type, used for both validated and unvalidated values.
///
/// We efficiently store the "best" / largest value we can.
#[derive(Debug, Clone)]
enum RawValueContent<'a, E: CustomExtension> {
    PayloadSlice(&'a [u8]),
    ValueSlice(&'a [u8]),
    ValueBodySlice(ValueKind<E::CustomValueKind>, &'a [u8]),
    OwnedPayload(Vec<u8>),
    OwnedValue(Vec<u8>),
    OwnedValueBody(ValueKind<E::CustomValueKind>, Vec<u8>),
}

impl<'a, E: CustomExtension> RawValueContent<'a, E> {
    pub fn from_payload_slice(payload_slice: &'a [u8]) -> Self {
        Self::PayloadSlice(payload_slice)
    }

    pub fn from_payload(payload_bytes: Vec<u8>) -> Self {
        Self::OwnedPayload(payload_bytes)
    }

    pub fn from_payload_cow(payload_cow: Cow<'a, [u8]>) -> Self {
        match payload_cow {
            Cow::Borrowed(payload_slice) => Self::PayloadSlice(payload_slice),
            Cow::Owned(payload_vec) => Self::OwnedPayload(payload_vec),
        }
    }

    pub fn from_value_slice(value_slice: &'a [u8]) -> Self {
        Self::ValueSlice(value_slice)
    }

    pub fn from_value(value_vec: Vec<u8>) -> Self {
        Self::OwnedValue(value_vec)
    }

    pub fn from_value_body_slice(
        value_kind: ValueKind<E::CustomValueKind>,
        value_body_slice: &'a [u8],
    ) -> Self {
        Self::ValueBodySlice(value_kind, value_body_slice)
    }

    pub fn from_value_body(
        value_kind: ValueKind<E::CustomValueKind>,
        value_body_vec: Vec<u8>,
    ) -> Self {
        Self::OwnedValueBody(value_kind, value_body_vec)
    }

    fn as_content_ref<'b>(&'b self) -> RawValueContent<'b, E> {
        match self {
            RawValueContent::PayloadSlice(slice) => RawValueContent::PayloadSlice(slice),
            RawValueContent::ValueSlice(slice) => RawValueContent::ValueSlice(slice),
            RawValueContent::ValueBodySlice(value_kind, slice) => {
                RawValueContent::ValueBodySlice(*value_kind, slice)
            }
            RawValueContent::OwnedPayload(vec) => RawValueContent::PayloadSlice(&vec),
            RawValueContent::OwnedValue(vec) => RawValueContent::ValueSlice(&vec),
            RawValueContent::OwnedValueBody(value_kind, vec) => {
                RawValueContent::ValueBodySlice(*value_kind, &vec)
            }
        }
    }

    /// Will be Some if already validated
    fn payload_len(&self) -> Option<usize> {
        match self {
            RawValueContent::PayloadSlice(slice) => Some(slice.len()),
            RawValueContent::ValueSlice(slice) => slice.len().checked_add(1),
            RawValueContent::ValueBodySlice(_, slice) => slice.len().checked_add(2),
            RawValueContent::OwnedPayload(vec) => Some(vec.len()),
            RawValueContent::OwnedValue(vec) => vec.len().checked_add(1),
            RawValueContent::OwnedValueBody(_, vec) => vec.len().checked_add(2),
        }
    }

    /// Will be Some if already validated
    fn value_kind(&self) -> Option<ValueKind<E::CustomValueKind>> {
        match self {
            RawValueContent::PayloadSlice(slice) => {
                slice.get(1).and_then(|v| ValueKind::from_u8(*v))
            }
            RawValueContent::ValueSlice(slice) => slice.get(0).and_then(|v| ValueKind::from_u8(*v)),
            RawValueContent::ValueBodySlice(value_kind, _) => Some(*value_kind),
            RawValueContent::OwnedPayload(vec) => vec.get(1).and_then(|v| ValueKind::from_u8(*v)),
            RawValueContent::OwnedValue(vec) => vec.get(0).and_then(|v| ValueKind::from_u8(*v)),
            RawValueContent::OwnedValueBody(value_kind, _) => Some(*value_kind),
        }
    }

    fn traverser_context(&self) -> (&[u8], ExpectedStart<E::CustomValueKind>) {
        match self {
            RawValueContent::PayloadSlice(slice) => {
                (slice, ExpectedStart::PayloadPrefix(E::PAYLOAD_PREFIX))
            }
            RawValueContent::ValueSlice(slice) => (slice, ExpectedStart::Value),
            RawValueContent::ValueBodySlice(value_kind, slice) => {
                (slice, ExpectedStart::ValueBody(*value_kind))
            }
            RawValueContent::OwnedPayload(vec) => {
                (vec, ExpectedStart::PayloadPrefix(E::PAYLOAD_PREFIX))
            }
            RawValueContent::OwnedValue(vec) => (vec, ExpectedStart::Value),
            RawValueContent::OwnedValueBody(value_kind, vec) => {
                (vec, ExpectedStart::ValueBody(*value_kind))
            }
        }
    }

    fn traverser(&self, max_depth: usize) -> VecTraverser<E::CustomTraversal> {
        let (slice, expected_start) = self.traverser_context();
        VecTraverser::<E::CustomTraversal>::new(slice, max_depth, expected_start, true)
    }

    fn typed_traverser<'s>(
        &self,
        max_depth: usize,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
    ) -> TypedTraverser<'_, 's, E> {
        let (slice, expected_start) = self.traverser_context();
        TypedTraverser::new(slice, schema, type_id, max_depth, expected_start, true)
    }
}

impl<'a, E: CustomExtension> PartialEq for RawValue<'a, E> {
    fn eq(&self, other: &Self) -> bool {
        self.value_kind() == other.value_kind() && self.value_body() == other.value_body()
    }
}

impl<'a, E: CustomExtension> Eq for RawValue<'a, E> {}

impl<'a, E: CustomExtension> PartialOrd for RawValue<'a, E> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, E: CustomExtension> Ord for RawValue<'a, E> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value_kind()
            .as_u8()
            .cmp(&other.value_kind().as_u8())
            .then_with(|| self.value_body().cmp(other.value_body()))
    }
}

impl<'a, E: CustomExtension> Hash for RawValue<'a, E> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value_kind().as_u8().hash(state);
        self.value_body().hash(state);
    }
}

impl<Ext: CustomExtension, E: Encoder<Ext::CustomValueKind>> Encode<Ext::CustomValueKind, E>
    for RawValue<'_, Ext>
{
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(self.value_kind())
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_slice(self.value_body())
    }
}

impl<Ext: CustomExtension, D: Decoder<Ext::CustomValueKind>> Decode<Ext::CustomValueKind, D>
    for RawValue<'_, Ext>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<Ext::CustomValueKind>,
    ) -> Result<Self, DecodeError> {
        // Because SBOR isn't a length-first decoding, you don't know how long a value "tree" is until you've decoded it.
        // So we use a traverser to calculate the length of the subpayload, and then read that many bytes.
        let length = calculate_value_tree_body_byte_length::<Ext>(
            decoder.peek_remaining(),
            value_kind,
            decoder.get_stack_depth(),
            decoder.get_depth_limit(),
        )?;
        // Because Decode doesn't (currently) allow borrowing from the decoder, we have to to_vec here
        Ok(Self::from_valid_value_body(
            value_kind,
            decoder.read_slice(length)?.to_vec(),
        ))
    }
}

impl<Ext: CustomExtension, C: CustomTypeKind<RustTypeId>> Describe<C> for RawValue<'_, Ext> {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(basic_well_known_types::ANY_TYPE);

    fn type_data() -> TypeData<C, RustTypeId> {
        basic_well_known_types::any_type_data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(BasicSbor)]
    struct RawValueStruct {
        field1: BasicOwnedRawValue,
        field2: (BasicOwnedRawValue, BasicOwnedRawValue),
    }

    #[test]
    pub fn can_encode_and_decode_raw_value() {
        let encoded = basic_encode(&BasicValue::Tuple {
            fields: vec![
                // Field1
                BasicValue::Enum {
                    discriminator: 1,
                    fields: vec![],
                },
                // Field2
                BasicValue::Tuple {
                    fields: vec![BasicValue::U8 { value: 1 }, BasicValue::U16 { value: 5125 }],
                },
            ],
        })
        .unwrap();
        let decoded: RawValueStruct = basic_decode(&encoded).unwrap();
        // Check that the content of the raw value makes sense
        assert_eq!(decoded.field2.1.value_kind(), ValueKind::U16);
        assert_eq!(
            decoded.field2.1.value_body(),
            // Extract the value body (ie remove the payload prefix byte and the value kind byte)
            &basic_encode(&5125u16).unwrap()[2..],
        );
        // Check that it can be encoded back to the original value
        let encoded2 = basic_encode(&decoded).unwrap();
        assert_eq!(encoded, encoded2);
    }

    #[test]
    pub fn unit_is_correct() {
        let encoded_unit = basic_encode(&()).unwrap();
        let decoded_unit: BasicRawValue = basic_decode(&encoded_unit).unwrap();
        let raw_unit = BasicRawValue::unit();
        let encoded_raw_unit = basic_encode(&BasicRawValue::unit()).unwrap();
        assert_eq!(raw_unit, decoded_unit);
        assert_eq!(encoded_raw_unit, encoded_unit);
    }
}
