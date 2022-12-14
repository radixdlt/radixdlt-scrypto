use indexmap::indexmap;

use crate::v2::*;

impl<C: CustomTypeSchema, T: Schema<C>> Schema<C> for Option<T> {
    const SCHEMA_TYPE_REF: GlobalTypeRef = GlobalTypeRef::complex("Option", &[T::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Enum {
                variants: indexmap![
                    "Some".to_owned() => GlobalTypeRef::complex("Some", &[T::SCHEMA_TYPE_REF]),
                    "None".to_owned() => GlobalTypeRef::complex("None", &[]),
                ],
            },
            naming: TypeNaming::named("Set"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
        aggregator.add_child_type(
            GlobalTypeRef::complex("Some", &[T::SCHEMA_TYPE_REF]),
            || Some(LocalTypeData::named_tuple("Some", vec![T::SCHEMA_TYPE_REF])),
        );
        aggregator.add_child_type(GlobalTypeRef::complex("None", &[]), || {
            Some(LocalTypeData::named_unit("None"))
        });
        aggregator.add_child_type_and_descendents::<T>();
    }
}

impl<C: CustomTypeSchema, T: Schema<C>, E: Schema<C>> Schema<C> for Result<T, E> {
    const SCHEMA_TYPE_REF: GlobalTypeRef =
        GlobalTypeRef::complex("Result", &[T::SCHEMA_TYPE_REF, E::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Enum {
                variants: indexmap![
                    "Ok".to_owned() => GlobalTypeRef::complex("Ok", &[T::SCHEMA_TYPE_REF]),
                    "Err".to_owned() => GlobalTypeRef::complex("Err", &[E::SCHEMA_TYPE_REF]),
                ],
            },
            naming: TypeNaming::named("Set"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
        aggregator.add_child_type(GlobalTypeRef::complex("Ok", &[T::SCHEMA_TYPE_REF]), || {
            Some(LocalTypeData::named_tuple("Ok", vec![T::SCHEMA_TYPE_REF]))
        });
        aggregator.add_child_type(GlobalTypeRef::complex("Err", &[E::SCHEMA_TYPE_REF]), || {
            Some(LocalTypeData::named_tuple("Err", vec![E::SCHEMA_TYPE_REF]))
        });
        aggregator.add_child_type_and_descendents::<T>();
        aggregator.add_child_type_and_descendents::<E>();
    }
}
