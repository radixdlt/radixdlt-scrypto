#![allow(dead_code)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    parse_quote
};


#[proc_macro]
pub fn init_trace_resources(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = if input.is_empty() {
        quote! {
            std::thread_local! {
                static QEMU_PLUGIN: std::cell::RefCell<QemuPluginInterface> = std::cell::RefCell::new(QemuPluginInterface::new(true));
            }
        }
    } else {
        let item: syn::ExprLit = syn::parse(input).expect("Failed to parse input.");
        //let port: Result<u16, &str> = 
        /*match item.lit {
            syn::Lit::Int(v) => if let Ok(value) = v.base10_parse::<u16>() {
                quote! {
                    std::thread_local! {
                        static QEMU_PLUGIN: std::cell::RefCell<QemuPluginInterface> = std::cell::RefCell::new(QemuPluginInterface::new(#value, true));
                    }
                }
            } else {
                syn::Error::new_spanned(v, "aa").to_compile_error()
            },
            _ => syn::Error::new_spanned(item, "aa").to_compile_error()
        }*/

        syn::Error::new_spanned(item, "aa").to_compile_error()
    };

    output.into()
}


#[proc_macro_attribute]
pub fn trace_resources(_attr: TokenStream, input: TokenStream) -> TokenStream {

    let output = if let Ok(mut item) = syn::Item::parse.parse(input.clone()) {
        match item {
            syn::Item::Fn(ref mut item_fn) => {
                let original_block = &mut item_fn.block;
                let fn_signature = item_fn.sig.ident.to_string();
                let new_block = Box::new( parse_quote! {{ 
                    QEMU_PLUGIN.with(|v| {
                        let stack = v.borrow().get_current_stack();
                        let space = std::iter::repeat(' ').take(2 * stack).collect::<String>();
                        println!("{}++enter: {}", space, #fn_signature);
                    });
                    QEMU_PLUGIN.with(|v| v.borrow_mut().start_counting(#fn_signature));
                    let ret = #original_block;
                    QEMU_PLUGIN.with(|v| {
                        let (stack, cnt) = v.borrow_mut().stop_counting();
                        let space = std::iter::repeat(' ').take(2 * stack).collect::<String>();
                        println!("{}--exit: {} {} {}", space, #fn_signature, stack, cnt);
                    });
                    ret
                }} );
                item_fn.block = new_block;
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