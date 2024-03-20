use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::*;

pub fn handle_catch_unwind(metadata: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
    let rtn_transformer = parse2::<Path>(metadata)?;

    if let Ok(mut item_fn) = parse2::<ItemFn>(input.clone()) {
        process_function(&mut item_fn.attrs, &mut item_fn.block, &rtn_transformer);
        Ok(quote! { #item_fn })
    } else if let Ok(mut item_impl) = parse2::<ItemImpl>(input) {
        for item in item_impl.items.iter_mut() {
            if let ImplItem::Method(method) = item {
                process_function(&mut method.attrs, &mut method.block, &rtn_transformer)
            }
        }
        Ok(quote! { #item_impl })
    } else {
        Err(Error::new(
            Span::call_site(),
            "Only functions and impls are supported by `catch_unwind`.",
        ))
    }
}

fn process_function(attrs: &mut Vec<Attribute>, block: &mut Block, rtn_transformer: &Path) {
    if let Some((index, _)) = attrs.iter().enumerate().find(|(_, attribute)| {
        (&(*attribute).clone())
            .into_token_stream()
            .to_string()
            .contains("catch_unwind_ignore")
    }) {
        attrs.remove(index);
        return;
    }

    let transformed_block: Block = parse_quote! { {
        #rtn_transformer(::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(crate::utils::fn_once(|| #block))))
    } };
    *block = transformed_block;
}
