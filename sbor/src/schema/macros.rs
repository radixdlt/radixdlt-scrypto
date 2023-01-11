macro_rules! describe_basic_well_known_type {
    ($type:ty, $well_known_index:ident) => {
        impl<C: CustomTypeKind<GlobalTypeId>> Describe<C> for $type {
            const TYPE_ID: GlobalTypeId =
                GlobalTypeId::well_known(basic_well_known_types::$well_known_index);
        }
    };
}
pub(crate) use describe_basic_well_known_type;

macro_rules! wrapped_generic_describe {
    ($generic:ident, $type:ty, $other_type:ty) => {
        impl<C: CustomTypeKind<GlobalTypeId>, $generic: Describe<C>> Describe<C> for $type {
            const TYPE_ID: GlobalTypeId = <$other_type>::TYPE_ID;

            fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
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
                C: CustomTypeKind<GlobalTypeId>,
                $key_generic: Describe<C>,
                $value_generic: Describe<C>,
            > Describe<C> for $type
        {
            const TYPE_ID: GlobalTypeId = <$other_type>::TYPE_ID;

            fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
                <$other_type>::type_data()
            }

            fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}
pub(crate) use wrapped_double_generic_describe;
