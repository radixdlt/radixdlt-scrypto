use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn extract_attributes(
    attrs: &[Attribute],
    name: &str,
) -> Option<BTreeMap<String, Option<String>>> {
    for attr in attrs {
        if !attr.path.is_ident(name) {
            continue;
        }

        let mut fields = BTreeMap::new();
        if let Ok(meta) = attr.parse_meta() {
            if let Meta::List(MetaList { nested, .. }) = meta {
                nested.into_iter().for_each(|m| match m {
                    NestedMeta::Meta(m) => match m {
                        Meta::NameValue(name_value) => {
                            if let Some(ident) = name_value.path.get_ident() {
                                if let Lit::Str(s) = name_value.lit {
                                    fields.insert(ident.to_string(), Some(s.value()));
                                }
                            }
                        }
                        Meta::Path(path) => {
                            if let Some(ident) = path.get_ident() {
                                fields.insert(ident.to_string(), None);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                })
            }
        }
        return Some(fields);
    }

    None
}

pub fn is_mutable(f: &Field) -> bool {
    extract_attributes(&f.attrs, "mutable").is_some()
}

pub fn handle_non_fungible_data(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_non_fungible_data() starts");

    let DeriveInput { ident, data, .. } = parse2(input)?;
    trace!("Processing: {}", ident.to_string());

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let mutable_fields: Punctuated<String, Comma> = named
                    .iter()
                    .filter(|f| is_mutable(f))
                    .filter_map(|f| f.ident.as_ref().map(|f| f.to_string()))
                    .collect();

                quote! {
                    impl ::scrypto::prelude::NonFungibleData for #ident {
                        const MUTABLE_FIELDS: &'static [&'static str] = &[#mutable_fields];
                    }
                }
            }
            syn::Fields::Unnamed(_) => {
                return Err(Error::new(
                    Span::call_site(),
                    "Struct with unnamed fields is not supported!",
                ));
            }
            syn::Fields::Unit => {
                return Err(Error::new(
                    Span::call_site(),
                    "Struct with no fields is not supported!",
                ));
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            return Err(Error::new(
                Span::call_site(),
                "Enum or union can not be used as non-fungible data presently!",
            ));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("NonFungibleData", &output);

    trace!("handle_non_fungible_data() finishes");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_non_fungible() {
        let input = TokenStream::from_str(
            "pub struct MyStruct { pub field_1: u32, #[mutable] pub field_2: String, }",
        )
        .unwrap();
        let output = handle_non_fungible_data(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::scrypto::prelude::NonFungibleData for MyStruct {
                    const MUTABLE_FIELDS : & 'static [& 'static str] = & ["field_2"] ;
                }
            },
        );
    }
}
