use proc_macro::TokenStream;
use quote::{ToTokens, quote};
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
                let code_print_arg = if arg.is_some() {
                      quote!{ #arg }
                } else {
                      quote!{ "" }
                };
                item_fn.block = Box::new( parse_quote! {{ 
                    let mut space = String::new();
                    QEMU_PLUGIN.with(|v| {
                        let stack = v.borrow().get_current_stack();
                        space = std::iter::repeat(' ').take(4 * stack).collect::<String>();
                        println!("[rtrack]{}++enter: {} {} {}", space, #fn_signature, stack + 1, #code_print_arg);
                        v.borrow_mut().start_counting(#fn_signature);
                    });
                    let ret = #original_block;
                    QEMU_PLUGIN.with(|v| {
                        let (stack, cnt) = v.borrow_mut().stop_counting();
                        println!("[rtrack]{}--exit: {} {} {} {}", space, #fn_signature, stack, cnt, #code_print_arg);
                    });
                    ret
                }} );
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
