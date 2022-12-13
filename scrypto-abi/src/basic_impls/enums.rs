use indexmap::indexmap;
use sbor::CustomTypeId;

use crate::v2::*;

impl<X: CustomTypeId, T: Schema<X>> Schema<X> for Option<T> {
    const SCHEMA_TYPE_REF: TypeRef = TypeRef::complex("Option", &[T::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Enum {
                variants: indexmap![
                    "Some".to_owned() => TypeRef::complex("Some", &[T::SCHEMA_TYPE_REF]),
                    "None".to_owned() => TypeRef::complex("None", &[]),
                ],
            },
            naming: TypeNaming::named("Set"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
        aggregator.add_child_type(TypeRef::complex("Some", &[T::SCHEMA_TYPE_REF]), || {
            Some(LocalTypeData::named_tuple("Some", vec![T::SCHEMA_TYPE_REF]))
        });
        aggregator.add_child_type(TypeRef::complex("None", &[]), || {
            Some(LocalTypeData::named_unit("None"))
        });
        aggregator.add_child_type_and_descendents::<T>();
    }
}

impl<X: CustomTypeId, T: Schema<X>, E: Schema<X>> Schema<X> for Result<T, E> {
    const SCHEMA_TYPE_REF: TypeRef =
        TypeRef::complex("Result", &[T::SCHEMA_TYPE_REF, E::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Enum {
                variants: indexmap![
                    "Ok".to_owned() => TypeRef::complex("Ok", &[T::SCHEMA_TYPE_REF]),
                    "Err".to_owned() => TypeRef::complex("Err", &[E::SCHEMA_TYPE_REF]),
                ],
            },
            naming: TypeNaming::named("Set"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
        aggregator.add_child_type(TypeRef::complex("Ok", &[T::SCHEMA_TYPE_REF]), || {
            Some(LocalTypeData::named_tuple("Ok", vec![T::SCHEMA_TYPE_REF]))
        });
        aggregator.add_child_type(TypeRef::complex("Err", &[E::SCHEMA_TYPE_REF]), || {
            Some(LocalTypeData::named_tuple("Err", vec![E::SCHEMA_TYPE_REF]))
        });
        aggregator.add_child_type_and_descendents::<T>();
        aggregator.add_child_type_and_descendents::<E>();
    }
}
