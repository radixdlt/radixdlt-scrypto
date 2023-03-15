use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, Parser},
    parse_quote
};


#[cfg(target_family = "unix")]
#[proc_macro_attribute]
pub fn trace_resources(attr: TokenStream, input: TokenStream) -> TokenStream {
    let arg = if let Ok(attrs) = syn::Ident::parse.parse(attr) {
        Some(attrs)
    } else {
        None
    };

    let output = if let Ok(mut item) = syn::Item::parse.parse(input.clone()) {
        match item {
            syn::Item::Fn(ref mut item_fn) => {
                let original_block = &mut item_fn.block;
                let fn_signature = item_fn.sig.ident.to_string();
                item_fn.block = if arg.is_some() {
                    Box::new( parse_quote! {{ 
                        QEMU_PLUGIN.with(|v| {
                            let stack = v.borrow().get_current_stack();
                            let space = std::iter::repeat(' ').take(4 * stack).collect::<String>();
                            println!("[rtrack]{}++enter: {}", space, #fn_signature);
                        });
                        QEMU_PLUGIN.with(|v| v.borrow_mut().start_counting(#fn_signature));
                        let ret = #original_block;
                        QEMU_PLUGIN.with(|v| {
                            let (stack, cnt) = v.borrow_mut().stop_counting();
                            let space = std::iter::repeat(' ').take(4 * stack).collect::<String>();
                            println!("[rtrack]{}--exit: {} {} {} {}", space, #fn_signature, stack, cnt, #arg);
                        });
                        ret
                    }} )
                } else {
                    Box::new( parse_quote! {{ 
                        QEMU_PLUGIN.with(|v| {
                            let stack = v.borrow().get_current_stack();
                            let space = std::iter::repeat(' ').take(4 * stack).collect::<String>();
                            println!("[rtrack]{}++enter: {}", space, #fn_signature);
                        });
                        QEMU_PLUGIN.with(|v| v.borrow_mut().start_counting(#fn_signature));
                        let ret = #original_block;
                        QEMU_PLUGIN.with(|v| {
                            let (stack, cnt) = v.borrow_mut().stop_counting();
                            let space = std::iter::repeat(' ').take(4 * stack).collect::<String>();
                            println!("[rtrack]{}--exit: {} {} {}", space, #fn_signature, stack, cnt);
                        });
                        ret
                    }} )
                };
                item.into_token_stream()
            }
            _ => syn::Error::new_spanned(item, "#[trace] is not supported for this item")
                .to_compile_error(),
        }

    } else {
        let input2 = proc_macro2::TokenStream::from(input);
        syn::Error::new_spanned(input2, "expected one of: `fn`, `impl`, `mod`").to_compile_error()
    };

    output.into()
}
