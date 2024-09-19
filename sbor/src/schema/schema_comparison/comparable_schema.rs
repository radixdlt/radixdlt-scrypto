use super::*;

/// A list of named comparable schemas, intended to capture various versions
/// of the same schema over time.
pub struct NamedSchemaVersions<S: CustomSchema, C: ComparableSchema<S>> {
    ordered_versions: IndexMap<String, C>,
    custom_schema: PhantomData<S>,
}

impl<S: CustomSchema, C: ComparableSchema<S>> NamedSchemaVersions<S, C> {
    pub fn new() -> Self {
        Self {
            ordered_versions: Default::default(),
            custom_schema: Default::default(),
        }
    }

    pub fn from<F: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: IntoComparableSchema<C, S>>(
        from: F,
    ) -> Self {
        Self {
            ordered_versions: from
                .into_iter()
                .map(|(name, version)| (name.as_ref().to_string(), version.into_schema()))
                .collect(),
            custom_schema: Default::default(),
        }
    }

    pub fn register_version(
        mut self,
        name: impl AsRef<str>,
        version: impl IntoComparableSchema<C, S>,
    ) -> Self {
        self.ordered_versions
            .insert(name.as_ref().to_string(), version.into_schema());
        self
    }

    pub fn get_versions(&self) -> &IndexMap<String, C> {
        &self.ordered_versions
    }
}

/// Marker trait for [`SingleTypeSchema`] and [`TypeCollectionSchema`] which
/// includes named pointers to types, and can be used for comparisons of
/// different versions of the same schema.
pub trait ComparableSchema<S: CustomSchema>: Clone + VecSbor<S::DefaultCustomExtension> {
    fn encode_to_bytes(&self) -> Vec<u8> {
        vec_encode::<S::DefaultCustomExtension, Self>(self, BASIC_SBOR_V1_MAX_DEPTH).unwrap()
    }

    fn encode_to_hex(&self) -> String {
        hex::encode(&self.encode_to_bytes())
    }

    fn decode_from_bytes(bytes: &[u8]) -> Self {
        vec_decode_with_nice_error::<S::DefaultCustomExtension, Self>(
            bytes,
            BASIC_SBOR_V1_MAX_DEPTH,
        )
        .unwrap_or_else(|err| {
            panic!(
                "Could not SBOR decode bytes into {} with {}: {:?}",
                core::any::type_name::<Self>(),
                core::any::type_name::<S::DefaultCustomExtension>(),
                err,
            )
        })
    }

    fn decode_from_hex(hex: &str) -> Self {
        let bytes = hex::decode(hex)
            .unwrap_or_else(|err| panic!("Provided string was not valid hex: {err}"));
        Self::decode_from_bytes(&bytes)
    }

    fn compare_with<'s>(
        &'s self,
        compared: &'s Self,
        settings: &SchemaComparisonSettings,
    ) -> SchemaComparisonResult<'s, S>;
}

impl<S: CustomSchema> ComparableSchema<S> for SingleTypeSchema<S> {
    fn compare_with<'s>(
        &'s self,
        compared: &'s Self,
        settings: &SchemaComparisonSettings,
    ) -> SchemaComparisonResult<'s, S> {
        SchemaComparisonKernel::new(
            &self.schema.as_unique_version(),
            &compared.schema.as_unique_version(),
            settings,
        )
        .compare_using_fixed_type_roots(&[ComparisonTypeRoot {
            name: "root".to_string(),
            base_type_id: self.type_id,
            compared_type_id: compared.type_id,
        }])
    }
}

impl<S: CustomSchema> ComparableSchema<S> for TypeCollectionSchema<S> {
    fn compare_with<'s>(
        &'s self,
        compared: &'s Self,
        settings: &SchemaComparisonSettings,
    ) -> SchemaComparisonResult<'s, S> {
        SchemaComparisonKernel::new(
            &self.schema.as_unique_version(),
            &compared.schema.as_unique_version(),
            settings,
        )
        .compare_using_named_type_roots(&self.type_ids, &compared.type_ids)
    }
}

pub trait IntoComparableSchema<C: ComparableSchema<S>, S: CustomSchema> {
    fn into_schema(&self) -> C;
}

impl<S: CustomSchema> IntoComparableSchema<Self, S> for SingleTypeSchema<S> {
    fn into_schema(&self) -> Self {
        self.clone()
    }
}

impl<S: CustomSchema> IntoComparableSchema<Self, S> for TypeCollectionSchema<S> {
    fn into_schema(&self) -> Self {
        self.clone()
    }
}

impl<'a, C: ComparableSchema<S>, S: CustomSchema, T: IntoComparableSchema<C, S> + ?Sized>
    IntoComparableSchema<C, S> for &'a T
{
    fn into_schema(&self) -> C {
        <T as IntoComparableSchema<C, S>>::into_schema(*self)
    }
}

impl<C: ComparableSchema<S>, S: CustomSchema> IntoComparableSchema<C, S> for [u8] {
    fn into_schema(&self) -> C {
        C::decode_from_bytes(self)
    }
}

impl<C: ComparableSchema<S>, S: CustomSchema, const N: usize> IntoComparableSchema<C, S>
    for [u8; N]
{
    fn into_schema(&self) -> C {
        C::decode_from_bytes(self.as_slice())
    }
}

impl<C: ComparableSchema<S>, S: CustomSchema> IntoComparableSchema<C, S> for Vec<u8> {
    fn into_schema(&self) -> C {
        C::decode_from_bytes(self)
    }
}

impl<C: ComparableSchema<S>, S: CustomSchema> IntoComparableSchema<C, S> for String {
    fn into_schema(&self) -> C {
        C::decode_from_hex(self)
    }
}

impl<C: ComparableSchema<S>, S: CustomSchema> IntoComparableSchema<C, S> for str {
    fn into_schema(&self) -> C {
        C::decode_from_hex(self)
    }
}
