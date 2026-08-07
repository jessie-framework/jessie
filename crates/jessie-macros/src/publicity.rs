use proc_macro::{Delimiter, TokenTree};

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub(crate) struct Publicity(Cow<'static, str>);

impl Publicity {
    pub fn parse(
        stream: &mut std::iter::Peekable<impl Iterator<Item = proc_macro::TokenTree>>,
    ) -> Self {
        if let Some(peek) = stream.peek().cloned().map(|x| x.to_string())
            && peek.as_str() == "pub"
        {
            stream.next();
            if let Some(peek) = stream.peek()
                && let TokenTree::Group(group) = peek
                && group.delimiter() == Delimiter::Parenthesis
            {
                let inner = group.stream();
                match inner.to_string().as_str() {
                    "crate" => return Self("pub(crate)".into()),
                    "self" => return Self("pub(self)".into()),
                    "super" => return Self("pub(super)".into()),
                    v => return Self(format!("pub({})", v).into()),
                }
            }
            return Self("pub".into());
        }
        Self("".into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_unspecified(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_pub(&self) -> bool {
        self.0 == "pub"
    }

    pub fn is_pub_crate(&self) -> bool {
        self.0 == "pub(crate)"
    }

    pub fn is_pub_self(&self) -> bool {
        self.0 == "pub(self)"
    }

    pub fn is_pub_super(&self) -> bool {
        self.0 == "pub(super)"
    }

    pub fn is_pub_in(&self) -> bool {
        if let Some(inner) = self.0.get(4..self.0.len() - 2)
            && inner.starts_with("in")
        {
            return true;
        }
        false
    }

    fn pub_in(&self) -> Option<&str> {
        if self.is_pub_in() {
            return self.0.get(7..self.0.len() - 2);
        }
        None
    }
}
