extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, PatType, Receiver};

#[proc_macro_attribute]
pub fn tx(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let vis = &input_fn.vis;
    let block = &input_fn.block;
    let fn_name = &input_fn.sig.ident;
    let fn_args = &input_fn.sig.inputs;
    let fn_return = &input_fn.sig.output;

    let arg_list: Vec<_> = fn_args
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(PatType { pat, .. }) => quote! { #pat },

            FnArg::Receiver(Receiver {
                reference,
                mutability,
                ..
            }) => {
                if reference.is_some() && mutability.is_some() {
                    quote!(self)
                } else if reference.is_some() {
                    quote!(&self)
                } else {
                    quote!(self)
                }
            }
        })
        .collect();

    let wrapped_fn_name = quote::format_ident!("{}_inner", fn_name);
    let gen = quote! {
        #vis async fn #wrapped_fn_name(#fn_args) #fn_return {
            #block
        }

        #vis async fn #fn_name(#fn_args) #fn_return {
            session.start_transaction().await?;
            match Self::#wrapped_fn_name(#(#arg_list),*).await {
                Ok(result) => {
                    session.commit_transaction().await?;
                    Ok(result)
                },
                Err(e) => {
                    session.abort_transaction().await?;
                    Err(e)
                }
            }
        }
    };

    TokenStream::from(gen)
}
