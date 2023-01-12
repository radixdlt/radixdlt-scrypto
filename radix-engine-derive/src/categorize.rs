use proc_macro2::TokenStream;
use syn::Result;

pub fn handle_categorize(input: TokenStream) -> Result<TokenStream> {
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::str::FromStr;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_categorize_struct() {
        let input = TokenStream::from_str("pub struct MyStruct { }").unwrap();
        let output = handle_categorize(input).unwrap();

        assert_code_eq(output, quote! {});
    }

    #[test]
    fn test_categorize_enum() {
        let input = TokenStream::from_str("enum MyEnum<T: Bound> { A { named: T }, B(String), C }")
            .unwrap();
        let output = handle_categorize(input).unwrap();

        assert_code_eq(output, quote! {});
    }
}
