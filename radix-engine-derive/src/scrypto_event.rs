use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, spanned::Spanned, Error, Result};

pub fn handle_scrypto_event(input: TokenStream) -> Result<TokenStream> {
    let item = parse2::<syn::Item>(input)?;

    let ident = match item {
        syn::Item::Struct(struct_item) => Ok(struct_item.ident),
        syn::Item::Enum(enum_item) => Ok(enum_item.ident),
        _ => Err(Error::new(
            item.span(),
            "An event type can either be a struct or an enum",
        )),
    }?;
    let ident_string = ident.to_string();

    // TODO: Assuming that ScryptoEvent is already imported. Do we want to always use the full path
    // in the re-interface crate?
    let derive = quote! {
        impl ScryptoEvent for #ident {
            fn event_name() -> &'static str {
                #ident_string
            }
        }
    };
    Ok(derive)
}
