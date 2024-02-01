#[macro_export]
macro_rules! token_stream_from_str {
    ($string: expr) => {
        <proc_macro2::TokenStream as std::str::FromStr>::from_str($string)
            .expect("Obtained from schema, must be valid!")
    };
}

#[macro_export]
macro_rules! ident {
    ($ident: expr) => {
        syn::Ident::new($ident, proc_macro2::Span::call_site())
    };
}
