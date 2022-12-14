use crate::v2::*;

macro_rules! tuple_schema {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<C: CustomTypeSchema, $($name: Schema<C>),+> Schema<C> for ($($name,)+) {
            const SCHEMA_TYPE_REF: GlobalTypeRef = GlobalTypeRef::complex("Tuple", &[$($name::SCHEMA_TYPE_REF, )+]);

            fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
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

tuple_schema! { 1 0 T0 }
tuple_schema! { 2 0 T0 1 T1 }
tuple_schema! { 3 0 T0 1 T1 2 T2 }
tuple_schema! { 4 0 T0 1 T1 2 T2 3 T3 }
tuple_schema! { 5 0 T0 1 T1 2 T2 3 T3 4 T4 }
tuple_schema! { 6 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 }
tuple_schema! { 7 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 }
tuple_schema! { 8 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 }
tuple_schema! { 9 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 }
tuple_schema! { 10 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 }
