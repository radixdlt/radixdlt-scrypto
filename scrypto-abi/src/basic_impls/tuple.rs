use crate::v2::*;
use sbor::*;

macro_rules! tuple_schema {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomTypeId, $($name: Schema<X>),+> Schema<X> for ($($name,)+) {
            const SCHEMA_TYPE_REF: TypeRef = TypeRef::complex("Tuple", &[$($name::SCHEMA_TYPE_REF, )+]);

            fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> {
                Some(LocalTypeData {
                    schema: TypeSchema::Tuple {
                        element_types: vec![
                            $($name::SCHEMA_TYPE_REF,)+
                        ],
                    },
                    naming: TypeNaming::named("Tuple"),
                })
            }
        }
    };
}

tuple_schema! { 1 0 A }
tuple_schema! { 2 0 A 1 B }
tuple_schema! { 3 0 A 1 B 2 C }
tuple_schema! { 4 0 A 1 B 2 C 3 D }
tuple_schema! { 5 0 A 1 B 2 C 3 D 4 E }
tuple_schema! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
tuple_schema! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
tuple_schema! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
tuple_schema! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
tuple_schema! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }
