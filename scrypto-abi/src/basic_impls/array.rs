use sbor::{SborTypeId, TypeId};

use crate::v2::*;

impl<C: CustomTypeSchema, T: Schema<C> + TypeId<C::CustomTypeId>> Schema<C> for [T] {
    const SCHEMA_TYPE_REF: GlobalTypeRef = if T::IS_U8 {
        GlobalTypeRef::well_known(well_known::BYTES_INDEX)
    } else {
        GlobalTypeRef::complex("Array", &[T::SCHEMA_TYPE_REF])
    };

    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        if T::IS_U8 {
            None
        } else {
            Some(LocalTypeData {
                schema: TypeSchema::Array {
                    element_sbor_type_id: T::type_id().as_u8(),
                    element_type: T::SCHEMA_TYPE_REF,
                    length_validation: LengthValidation::none(),
                },
                naming: TypeNaming::named("Array"),
            })
        }
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
        aggregator.add_child_type_and_descendents::<T>();
    }
}

impl<C: CustomTypeSchema, T: Schema<C> + TypeId<C::CustomTypeId>, const N: usize> Schema<C>
    for [T; N]
{
    const SCHEMA_TYPE_REF: GlobalTypeRef =
        GlobalTypeRef::complex_sized("Array", &[T::SCHEMA_TYPE_REF], N);

    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        let size = N
            .try_into()
            .expect("The array length is too large for a u32 for the SBOR schema");
        let type_name = if T::type_id() == SborTypeId::U8 {
            "Bytes"
        } else {
            "Array"
        };
        Some(LocalTypeData {
            schema: TypeSchema::Array {
                element_sbor_type_id: T::type_id().as_u8(),
                element_type: T::SCHEMA_TYPE_REF,
                length_validation: LengthValidation {
                    min: Some(size),
                    max: Some(size),
                },
            },
            naming: TypeNaming::named(type_name),
        })
    }

    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
        aggregator.add_child_type_and_descendents::<T>();
    }
}
