macro_rules! describe_basic_well_known_type {
    ($type:ty, $well_known_index:ident, $well_known_type_data_method:ident) => {
        impl<C: CustomTypeKind<RustTypeId>> Describe<C> for $type {
            const TYPE_ID: RustTypeId =
                RustTypeId::WellKnown(basic_well_known_types::$well_known_index);

            fn type_data() -> TypeData<C, RustTypeId> {
                basic_well_known_types::$well_known_type_data_method()
            }
        }
    };
}
pub(crate) use describe_basic_well_known_type;

macro_rules! wrapped_generic_describe {
    ($generic:ident, $type:ty, $other_type:ty) => {
        impl<C: CustomTypeKind<RustTypeId>, $generic: Describe<C>> Describe<C> for $type {
            const TYPE_ID: RustTypeId = <$other_type>::TYPE_ID;

            fn type_data() -> TypeData<C, RustTypeId> {
                <$other_type>::type_data()
            }

            fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}
pub(crate) use wrapped_generic_describe;

macro_rules! wrapped_double_generic_describe {
    ($key_generic:ident, $value_generic:ident, $type:ty, $other_type:ty) => {
        impl<
                C: CustomTypeKind<RustTypeId>,
                $key_generic: Describe<C>,
                $value_generic: Describe<C>,
            > Describe<C> for $type
        {
            const TYPE_ID: RustTypeId = <$other_type>::TYPE_ID;

            fn type_data() -> TypeData<C, RustTypeId> {
                <$other_type>::type_data()
            }

            fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}
pub(crate) use wrapped_double_generic_describe;
