use sbor::*;
use sbor::rust::collections::*;

use crate::v2::*;

use_same_generic_schema!(T, Vec<T>, [T]);

impl<X: CustomTypeId, T: Schema<X> + TypeId<X>> Schema<X> for BTreeSet<T> {
    const SCHEMA_TYPE_REF: TypeRef = TypeRef::complex("Set", &[T::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Array {
                element_sbor_type_id: T::type_id().as_u8(),
                element_type: T::SCHEMA_TYPE_REF,
                length_validation: LengthValidation::none()
            },
            naming: TypeNaming::named("Set"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
        aggregator.attempt_add_schema_and_descendents::<T>();
    }
}

use_same_generic_schema!(T, HashSet<T>, BTreeSet<T>);
#[cfg(feature = "indexmap")]
use_same_generic_schema!(T, IndexSet<T>, BTreeSet<T>);

impl<X: CustomTypeId, K: Schema<X>, V: Schema<X>> Schema<X> for BTreeMap<K, V>
{
    const SCHEMA_TYPE_REF: TypeRef = TypeRef::complex("Map", &[K::SCHEMA_TYPE_REF, V::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Array {
                element_sbor_type_id: <(K, V) as TypeId::<X>>::type_id().as_u8(),
                element_type: <(K, V)>::SCHEMA_TYPE_REF,
                length_validation: LengthValidation::none()
            },
            naming: TypeNaming::named("Map"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
        aggregator.attempt_add_schema_and_descendents::<K>();
        aggregator.attempt_add_schema_and_descendents::<V>();
    }
}

use_same_double_generic_schema!(K, V, HashMap<K, V>, BTreeMap<K, V>);
#[cfg(feature = "indexmap")]
use_same_double_generic_schema!(K, V, HashMap<K, V>, IndexMap<K, V>);
