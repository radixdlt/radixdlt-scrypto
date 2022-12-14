use sbor::rust::collections::*;
use sbor::*;

use crate::v2::*;

use_same_generic_schema!(T, Vec<T>, [T]);

impl<C: CustomTypeSchema, T: Schema<C> + TypeId<C::CustomTypeId>> Schema<C> for BTreeSet<T> {
    const SCHEMA_TYPE_REF: GlobalTypeRef = GlobalTypeRef::complex("Set", &[T::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Array {
                element_sbor_type_id: T::type_id().as_u8(),
                element_type: T::SCHEMA_TYPE_REF,
                length_validation: LengthValidation::none(),
            },
            naming: TypeNaming::named("Set"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
        aggregator.add_child_type_and_descendents::<T>();
    }
}

use_same_generic_schema!(T, HashSet<T>, BTreeSet<T>);
#[cfg(feature = "indexmap")]
use_same_generic_schema!(T, IndexSet<T>, BTreeSet<T>);

impl<C: CustomTypeSchema, K: Schema<C>, V: Schema<C>> Schema<C> for BTreeMap<K, V> {
    const SCHEMA_TYPE_REF: GlobalTypeRef =
        GlobalTypeRef::complex("Map", &[K::SCHEMA_TYPE_REF, V::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Array {
                element_sbor_type_id: <(K, V) as TypeId<C::CustomTypeId>>::type_id().as_u8(),
                element_type: <(K, V)>::SCHEMA_TYPE_REF,
                length_validation: LengthValidation::none(),
            },
            naming: TypeNaming::named("Map"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
        aggregator.add_child_type_and_descendents::<K>();
        aggregator.add_child_type_and_descendents::<V>();
    }
}

use_same_double_generic_schema!(K, V, HashMap<K, V>, BTreeMap<K, V>);
#[cfg(feature = "indexmap")]
use_same_double_generic_schema!(K, V, HashMap<K, V>, IndexMap<K, V>);
