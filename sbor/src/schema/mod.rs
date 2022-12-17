mod schema;
mod schema_aggregator;
mod type_ref;
mod type_schema;
mod well_known_type_schemas;

pub use schema::*;
pub use schema_aggregator::*;
pub use type_ref::*;
pub use type_schema::*;
pub use well_known_type_schemas::*;

macro_rules! well_known_basic_schema {
    ($type:ty, $well_known_index:ident) => {
        impl<C: CustomTypeSchema> Schema<C> for $type {
            const SCHEMA_TYPE_REF: GlobalTypeRef =
                GlobalTypeRef::well_known(well_known_basic_schemas::$well_known_index);
        }
    };
}
pub(crate) use well_known_basic_schema;

macro_rules! use_same_generic_schema {
    ($generic:ident, $type:ty, $other_type:ty) => {
        impl<C: CustomTypeSchema, $generic: Schema<C> + TypeId<C::CustomTypeId>> Schema<C>
            for $type
        {
            const SCHEMA_TYPE_REF: GlobalTypeRef = <$other_type>::SCHEMA_TYPE_REF;

            fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
                <$other_type>::get_local_type_data()
            }

            fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}
pub(crate) use use_same_generic_schema;

macro_rules! use_same_double_generic_schema {
    ($key_generic:ident, $value_generic:ident, $type:ty, $other_type:ty) => {
        impl<C: CustomTypeSchema, $key_generic: Schema<C>, $value_generic: Schema<C>> Schema<C>
            for $type
        {
            const SCHEMA_TYPE_REF: GlobalTypeRef = <$other_type>::SCHEMA_TYPE_REF;

            fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
                <$other_type>::get_local_type_data()
            }

            fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}
pub(crate) use use_same_double_generic_schema;
