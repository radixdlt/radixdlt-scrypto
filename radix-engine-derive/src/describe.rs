use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_describe(input: TokenStream) -> Result<TokenStream> {
    sbor_derive_common::describe::handle_describe(
        input,
        Some("radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use sbor_derive_common::utils::get_code_hash_const_array_token_stream;
    use std::str::FromStr;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_describe_struct() {
        let input = TokenStream::from_str("pub struct MyStruct { }").unwrap();
        let code_hash = get_code_hash_const_array_token_stream(&input);
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> > for MyStruct {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(MyStruct),
                        &[],
                        &#code_hash
                    );
                    fn type_data() -> Option<::sbor::TypeData<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>, ::sbor::GlobalTypeId>> {
                        Some(::sbor::TypeData::named_fields_tuple(
                            stringify!(MyStruct),
                            ::sbor::rust::vec![],
                        ))
                    }
                    fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> >) {}
                }
            },
        );
    }

    #[test]
    fn test_describe_enum() {
        let input = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let code_hash = get_code_hash_const_array_token_stream(&input);
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl<T: Bound>
                    ::sbor::Describe<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> > for MyEnum<T>
                where
                    T: ::sbor::Describe<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> >,
                    T: ::sbor::Categorize<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>::CustomValueKind >
                {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(MyEnum),
                        &[<T>::TYPE_ID,],
                        &#code_hash
                    );
                    fn type_data() -> Option<::sbor::TypeData<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>, ::sbor::GlobalTypeId>> {
                        use ::sbor::rust::borrow::ToOwned;
                        Some(::sbor::TypeData::named_enum(
                            stringify!(MyEnum),
                            :: sbor :: rust :: collections :: btree_map :: btreemap ! [
                                0u8 => :: sbor :: TypeData :: named_fields_tuple ("A", :: sbor :: rust :: vec ! [("named", < T as :: sbor :: Describe < radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> >> :: TYPE_ID) ,] ,) ,
                                1u8 => :: sbor :: TypeData :: named_tuple ("B", :: sbor :: rust :: vec ! [< String as :: sbor :: Describe < radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> >> :: TYPE_ID ,] ,) ,
                                2u8 => :: sbor :: TypeData :: named_unit ("C") ,
                            ],
                        ))
                    }
                    fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> >) {
                        aggregator.add_child_type_and_descendents::<T>();
                        aggregator.add_child_type_and_descendents::<String>();
                    }
                }
            },
        );
    }
}
