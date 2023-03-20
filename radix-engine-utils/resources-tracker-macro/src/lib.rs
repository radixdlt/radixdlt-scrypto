#![allow(unused_imports)]

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    parse::{Parse, Parser},
    parse_quote,
    FnArg, Pat::Lit, Pat::Ident, Type::Path, Type::Reference
};

#[cfg(target_family = "unix")]
#[cfg(feature = "resource_tracker")]
use radix_engine_utils::QEMU_PLUGIN;
#[cfg(feature = "resource_tracker")]
use radix_engine_utils::data_analyzer::*;

#[cfg(not(feature = "resource_tracker"))]
#[proc_macro_attribute]
pub fn trace_resources(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[cfg(target_family = "unix")]
#[cfg(feature = "resource_tracker")]
#[proc_macro_attribute]
pub fn trace_resources(attr: TokenStream, input: TokenStream) -> TokenStream {

    let arg_name = if let Ok(ast) = syn::parse(attr.clone()) 
    {
        let a: syn::Ident = ast;
        Some(a)
    } else {
        //println!("args: no specified");
        None
    };

    // let arg = if let Ok(attrs) = syn::Ident::parse.parse(attr) {
    //     quote!{ Some(OutputParam::Literal( #attrs )) }
    // } else {
    //     quote!{ None }
    // };

    let output = if let Ok(mut item) = syn::Item::parse.parse(input.clone()) {
        match item {
            syn::Item::Fn(ref mut item_fn) => {
                let original_block = &mut item_fn.block;
                let fn_signature = item_fn.sig.ident.to_string();

                let mut aarg = quote!{ None };
                if arg_name.is_some() {
                    for fn_arg in item_fn.sig.inputs.iter() {
                        match fn_arg {
                            FnArg::Typed(v) => {
                                match v.pat.as_ref() {
                                    Lit(pl) => {
                                        println!("pat lit");
                                        aarg = quote!{ Some( OutputParam::Literal(#pl)) };
                                        break;
                                    }
                                    Ident(pi) => {
                                        if pi.ident == arg_name.clone().unwrap() {
                                            println!("found variable in fnsig: {}", pi.ident);
                                            match v.ty.as_ref() {
                                                Path(tp) => { // u8..i64
                                                    if let Some(p) = tp.path.segments.last() {
                                                        println!("ty path: {}", p.ident);
                                                        if p.ident == "u8" || p.ident == "u16" || p.ident == "u32" || p.ident == "u64" {
                                                            aarg = quote!{ Some(OutputParam::NumberU64( #pi as u64 )) };
                                                            break;
                                                        } else if p.ident == "i8" || p.ident == "i16" || p.ident == "i32" || p.ident == "i64" {
                                                            aarg = quote!{ Some(OutputParam::NumberI64( #pi as i64 )) };
                                                            break;
                                                        } else {
                                                            panic!("Not supported arg type: {}", p.ident);
                                                        }
                                                    }
                                                }
                                                Reference(tr) => { //&str
                                                    //tr.elem.as_ref().
                                                    //println!("ty path: {:?}", tr.elem);
                                                    match tr.elem.as_ref() {
                                                        Path(tp) => {
                                                            if let Some(p) = tp.path.segments.last() {
                                                                if p.ident == "str" {
                                                                    println!("ty str: {}", pi.ident);
                                                                    aarg = quote!{ Some( OutputParam::Literal(#pi.into())) };
                                                                    break;
                                                                } else {
                                                                    panic!("Not supported arg type: {}", p.ident);
                                                                }
                                                            }
                                                        }
                                                        _ => ()
                                                    }
                                                }
                                                _ => ()
                                            }
                                        }
                                    } 
                                    _ => ()
                                }
                            },
                            _ => ()
                        }
                    }
                };


                item_fn.block = Box::new( parse_quote! {{ 
                    use radix_engine_utils::{QEMU_PLUGIN, OutputParam};
                    QEMU_PLUGIN.with(|v| {
                        // let stack = v.borrow().get_current_stack();
                        // let spaces = [' '; 40];
                        // //let space = std::iter::repeat(' ').take(4 * stack).collect::<String>();
                        // println!("[rtrack]{}++enter: {} {} {}", spaces[], #fn_signature, stack + 1, #arg);
                        v.borrow_mut().start_counting(#fn_signature, #aarg);
                    });
                    let ret = #original_block;
                    QEMU_PLUGIN.with(|v| {
                        let (stack, cnt) = v.borrow_mut().stop_counting(#fn_signature, #aarg);
                        //let space = std::iter::repeat(' ').take(4 * stack).collect::<String>();
                        //println!("[rtrack]{}--exit: {} {} {} {}", space, #fn_signature, stack, cnt, #arg);
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
