use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_sbor(
    input: TokenStream,
    context_custom_value_kind: Option<&'static str>,
    context_custom_type_kind: Option<&'static str>,
) -> Result<TokenStream> {
    trace!("handle_sbor() starts");

    let categorize =
        crate::categorize::handle_categorize(input.clone(), context_custom_value_kind.clone())?;
    let encode = crate::encode::handle_encode(input.clone(), context_custom_value_kind.clone())?;
    let decode = crate::decode::handle_decode(input.clone(), context_custom_value_kind.clone())?;
    let describe = crate::describe::handle_describe(input, context_custom_type_kind)?;

    let output = quote! {
        #categorize

        #encode

        #decode

        #describe
    };

    trace!("handle_sbor() finishes");
    Ok(output)
}
