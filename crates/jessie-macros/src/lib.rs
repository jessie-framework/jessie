use proc_macro::TokenStream as TS;

use crate::parser::Parser;

mod objects;
mod parser;
mod publicity;
mod tree;

#[proc_macro_attribute]
pub fn object(obj_name: TS, item: TS) -> TS {
    let tree = item.parse(obj_name);
    tree.convert()
}
