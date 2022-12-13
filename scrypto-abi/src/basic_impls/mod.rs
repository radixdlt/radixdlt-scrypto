macro_rules! well_known_schema {
    ($type:ty, $well_known_index:ident) => {
        impl<X: ::sbor::CustomTypeId> Schema<X> for $type {
            const SCHEMA_TYPE_REF: TypeRef = TypeRef::well_known(well_known::$well_known_index);
        }
    };
}

macro_rules! use_same_schema {
    ($type:ident, $other_type:ty) => {
        impl<X: ::sbor::CustomTypeId> Schema<X> for $type {
            const SCHEMA_TYPE_REF: TypeRef = <$other_type>::SCHEMA_TYPE_REF;

            fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
                <$other_type>::get_local_type_data()
            }

            fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}

macro_rules! use_same_generic_schema {
    ($generic:ident, $type:ty, $other_type:ty) => {
        impl<X: ::sbor::CustomTypeId, $generic: Schema<X> + TypeId<X>> Schema<X> for $type {
            const SCHEMA_TYPE_REF: TypeRef = <$other_type>::SCHEMA_TYPE_REF;

            fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
                <$other_type>::get_local_type_data()
            }

            fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}

macro_rules! use_same_double_generic_schema {
    ($key_generic:ident, $value_generic:ident, $type:ty, $other_type:ty) => {
        impl<X: ::sbor::CustomTypeId, $key_generic: Schema<X>, $value_generic: Schema<X>> Schema<X> for $type {
            const SCHEMA_TYPE_REF: TypeRef = <$other_type>::SCHEMA_TYPE_REF;

            fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
                <$other_type>::get_local_type_data()
            }

            fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
                <$other_type>::add_all_dependencies(aggregator)
            }
        }
    };
}

mod array;
mod boolean;
mod collection;
mod integer;
mod misc;
mod enums;
mod string;
mod tuple;
mod unit;

pub use array::*;
pub use boolean::*;
pub use collection::*;
pub use integer::*;
pub use misc::*;
pub use enums::*;
pub use string::*;
pub use tuple::*;
pub use unit::*;
