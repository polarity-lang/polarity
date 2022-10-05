extern crate proc_macro;

use quote::{quote, ToTokens};

mod codegen;
mod parser;
mod syntax;

#[proc_macro_attribute]
pub fn trace(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let f: syn::ItemFn = syn::parse(input).unwrap();

    let mut format = parser::parse(attr.into()).unwrap();
    let code_head = format.to_token_stream();
    format.ret = Some(quote! { &__tracer_res });
    format.ret_type = match &f.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, typ) => Some(*typ.clone()),
    };
    let code_tail = format.to_token_stream();

    let f_sig = &f.sig;
    let mut f_attrs = proc_macro2::TokenStream::new();
    f_attrs.extend(f.attrs.iter().map(|attr| attr.to_token_stream()));
    let f_vis = &f.vis;
    let f_block = &f.block;

    quote! {
        #f_attrs #f_vis #f_sig {
            if tracer::enabled() {
                #code_head
            }
            let __tracer_res = #f_block;
            if tracer::enabled() {
                #code_tail
            }
            __tracer_res
        }
    }
    .into()
}
