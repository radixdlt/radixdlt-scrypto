#[cfg(not(panic = "unwind"))]
compile_error!("The `catch_unwind` crate requires that `panic = \"unwind\"` in the profile");

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::*;

#[proc_macro_attribute]
pub fn catch_unwind(metadata: TokenStream, input: TokenStream) -> TokenStream {
    handle_catch_unwind(metadata.into(), input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn ignore(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

fn handle_catch_unwind(metadata: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
    let rtn_transformer = parse2::<Path>(metadata)?;

    if let Ok(item_fn) = parse2::<ItemFn>(input.clone()) {
        let impl_item_fn = ImplItemFn {
            attrs: item_fn.attrs,
            vis: item_fn.vis,
            sig: item_fn.sig,
            block: item_fn.block.as_ref().to_owned(),
            defaultness: None,
        };
        let processed = process_function(impl_item_fn, &rtn_transformer);
        Ok(quote! { #processed })
    } else if let Ok(mut item_impl) = parse2::<ItemImpl>(input) {
        for item in item_impl.items.iter_mut() {
            let ImplItem::Fn(impl_item_fn) = item else { continue };
            *impl_item_fn = process_function(impl_item_fn.clone(), &rtn_transformer);
        }
        Ok(quote! { #item_impl })
    } else {
        Err(Error::new(
            Span::call_site(),
            "Only functions and impls are supported by `catch_unwind`.",
        ))
    }
}

fn process_function(mut item_fn: ImplItemFn, rtn_transformer: &Path) -> ImplItemFn {
    if let Some((index, _)) = item_fn.attrs.iter().enumerate().find(|(_, attribute)| {
        (&(*attribute).clone())
            .into_token_stream()
            .to_string()
            .contains("catch_unwind_ignore")
    }) {
        item_fn.attrs.remove(index);
        return item_fn;
    }

    let block = item_fn.block;
    let block: Block = parse_quote! { {
        #rtn_transformer(::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(crate::utils::fn_once(|| #block))))
    } };
    item_fn.block = block;
    item_fn
}
