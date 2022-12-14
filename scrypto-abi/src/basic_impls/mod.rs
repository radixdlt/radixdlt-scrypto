macro_rules! well_known_schema {
    ($type:ty, $well_known_index:ident) => {
        impl<C: CustomTypeSchema> Schema<C> for $type {
            const SCHEMA_TYPE_REF: GlobalTypeRef =
                GlobalTypeRef::well_known(well_known::$well_known_index);
        }
    };
}

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

mod array;
mod boolean;
mod collection;
mod enums;
mod integer;
mod misc;
mod string;
mod tuple;
mod unit;

pub use array::*;
pub use boolean::*;
pub use collection::*;
pub use enums::*;
pub use integer::*;
pub use misc::*;
pub use string::*;
pub use tuple::*;
pub use unit::*;
