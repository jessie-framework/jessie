use proc_macro::TokenStream;

use crate::tree::Tree;

pub(crate) trait Parser {
    fn parse(self, attr_name: TokenStream) -> Tree;
}

impl Parser for TokenStream {
    fn parse(self, attr_name: TokenStream) -> Tree {
        Tree::parse(&mut self.into_iter().peekable(), attr_name).unwrap()
    }
}
