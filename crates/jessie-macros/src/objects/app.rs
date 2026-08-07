use proc_macro::TokenStream;
use std::str::FromStr;

use crate::tree::Tree;

pub(crate) fn app(input: Tree) -> TokenStream {
    let mut out = TokenStream::new();

    let _config_file_dir = input.object.inner_string(); // TODO : maybe we can have a config file with special configurations for apps and stuff? idk

    let ty_name = &input.ty_name;
    let publicity = &input.publicity.as_str();

    let _style = input.get_field("style");

    let _elements = input.get_elements();

    out.extend(TokenStream::from_str(&format!(
        "
                    {publicity} struct {ty_name};   
            "
    )));

    if let Some(extra_state) = input.get_field("extra_state") {
        let extra_state_ty = extra_state.ty_name;
        out.extend(TokenStream::from_str(&format!(
            "
                    impl {ty_name} {{
                        pub fn run(extra_state : &mut {extra_state_ty}) -> Self {{
                            todo!()
                        }}
                    }}
            "
        )));
    } else {
        out.extend(TokenStream::from_str(&format!(
            "
                    impl {ty_name} {{
                        pub fn run() -> Self {{
                            todo!()
                        }}
                    }}
            "
        )));
    }

    out
}
