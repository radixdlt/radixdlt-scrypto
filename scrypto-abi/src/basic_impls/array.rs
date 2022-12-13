use sbor::{CustomTypeId, TypeId};

use crate::v2::*;

impl<X: CustomTypeId, T: Schema<X> + TypeId<X>> Schema<X> for [T] {
    const SCHEMA_TYPE_REF: TypeRef = TypeRef::complex("Array", &[T::SCHEMA_TYPE_REF]);

    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
        Some(LocalTypeData {
            schema: TypeSchema::Array {
                element_sbor_type_id: T::type_id().as_u8(),
                element_type: T::SCHEMA_TYPE_REF,
                length_validation: LengthValidation::none(),
            },
            naming: TypeNaming::named("Array"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
        aggregator.add_child_type_and_descendents::<T>();
    }
}

impl<X: CustomTypeId, T: Schema<X> + TypeId<X>, const N: usize> Schema<X> for [T; N] {
    const SCHEMA_TYPE_REF: TypeRef = TypeRef::complex_sized("Array", &[T::SCHEMA_TYPE_REF], N);

    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
        let size = N
            .try_into()
            .expect("The array length is too large for a u32 for the SBOR schema");
        Some(LocalTypeData {
            schema: TypeSchema::Array {
                element_sbor_type_id: T::type_id().as_u8(),
                element_type: T::SCHEMA_TYPE_REF,
                length_validation: LengthValidation {
                    min: Some(size),
                    max: Some(size),
                },
            },
            naming: TypeNaming::named("Array"),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<X>) {
        aggregator.add_child_type_and_descendents::<T>();
    }
}
