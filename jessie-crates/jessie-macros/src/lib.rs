use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn app(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_body = parse_macro_input!(item as ItemFn);

    let vis = &fn_body.vis;

    let fn_name = &fn_body.sig.ident;

    let output = quote! {
        include!(concat!(env!("OUT_DIR"),"/appinfo.rs"));

        #fn_body

        #vis fn main() {
            jessie_lib::run(APP_INFO,#fn_name());

        }
    };

    output.into()
}
