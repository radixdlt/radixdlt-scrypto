use proc_macro::{self, TokenStream};
use quote::quote;
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_describe(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    trace!("Describing {:?}", &ident);

    // TODO

    let output = quote! {
        impl scrypto::abi::Describe for #ident {
            fn describe() -> scrypto::abi::Type {
                scrypto::abi::Type::U32
            }
        }
    };

    print_compiled_code("#[derive(Describe)]", &output);

    output.into()
}
