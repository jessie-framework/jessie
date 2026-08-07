use std::iter::Peekable;

use proc_macro::{Delimiter, TokenStream, TokenTree};

use crate::publicity::Publicity;

/// A tree is a struct that is annotated with the `jessie::object` macro, assumed to be a tuple struct with annotated fields.
#[derive(Debug, Clone)]
pub(crate) struct Tree {
    pub object: Object,       // The object, the inner value of the attribute.
    pub publicity: Publicity, // Publicity of the struct.
    pub ty_name: String,      // The name of the struct.
    pub fields: Vec<Field>,   // List of attributes.
}

impl Tree {
    // Parses the TokenStream of an item to create a Tree.
    pub fn parse(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        attr_name: TokenStream,
    ) -> Option<Self> {
        let object = Object::parse(attr_name)?;
        let publicity = Publicity::parse(stream);
        stream.next();
        let ty_name = stream.next()?.to_string(); // FIXME : ideally we would want to have error handling and proper parsing instead of just parsing the next token
        if let Some(peek) = stream.peek()
            && let TokenTree::Group(group) = peek
            && group.delimiter() == Delimiter::Parenthesis
        {
            let mut inner = group.stream().into_iter().peekable();
            let fields = Field::parse_vec(&mut inner);
            return Some(Self {
                object,
                publicity,
                ty_name,
                fields,
            });
        }
        None
    }

    // Converts the tree into a stream of tokens.
    pub fn convert(self) -> TokenStream {
        match self.object.name.as_str() {
            "app" => crate::objects::app::app(self),
            _ => todo!(),
        }
    }

    pub fn get_field(&self, name: &str) -> Option<Field> {
        self.fields
            .iter()
            .find(|x| x.attrs.iter().find(|x| x.as_str() == name).is_some())
            .cloned()
    }

    pub fn get_fields(&self, name: &str) -> Vec<Field> {
        self.fields
            .iter()
            .filter(|x| x.attrs.iter().find(|x| x.as_str() == name).is_some())
            .cloned()
            .collect()
    }

    pub fn get_elements(&self) -> Vec<Field> {
        self.fields
            .iter()
            .filter(|x| x.is_element())
            .cloned()
            .collect()
    }
}

/// An object is the inner part of an annotation for a struct, using the `jessie::object` macro.
#[derive(Debug, Clone)]
pub(crate) struct Object {
    pub name: String, // Name of the object.
    stream: Option<TokenStream>,
}

impl Object {
    pub fn parse(input: TokenStream) -> Option<Self> {
        let mut iter = input.into_iter().peekable();
        let name = iter.next()?.to_string();
        let stream = if let Some(TokenTree::Group(group)) = iter.next()
            && group.delimiter() == Delimiter::Parenthesis
        {
            Some(group.stream())
        } else {
            None
        };
        Some(Self { name, stream })
    }

    pub fn inner_string(&self) -> Option<String> {
        let mut iter = self.stream.clone()?.into_iter();
        if let Some(TokenTree::Literal(literal)) = iter.next()
            && iter.next().is_none()
            && let str = literal.to_string()
            && str.starts_with("\"")
            && let Some(inner) = str.get(1..str.len() - 2)
        {
            return Some(inner.to_string());
        }
        None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub attrs: Vec<Attr>,     // List of attributes of the field.
    pub publicity: Publicity, // Publicity of the field.
    pub ty_name: String,      // Type name of the field.
}

impl Field {
    fn parse_vec(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Vec<Self> {
        let mut out = Vec::new();
        loop {
            out.push(Self::parse(stream));
            if let Some(peek) = stream.peek()
                && peek.to_string() == ","
            {
                continue;
            } else {
                break;
            }
        }
        out
    }

    fn parse(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let attrs = Attr::parse_vec(stream);
        let publicity = Publicity::parse(stream);
        let mut ty_name = String::new();
        while let Some(peek) = stream.peek()
            && let str = peek.to_string()
            && &str != ","
        {
            stream.next();
            ty_name.push_str(&str);
        }
        Self {
            attrs,
            publicity,
            ty_name,
        }
    }

    pub fn is_element(&self) -> bool {
        self.attrs
            .iter()
            .find(|attr| matches!(attr.as_str(), "div",))
            .is_some()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Attr(String);

impl Attr {
    fn parse(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<Self> {
        if let Some(hash) = stream.peek().map(|x| x.to_string())
            && hash == "#"
        {
            stream.next();
            if let Some(brackets) = stream.next()
                && let TokenTree::Group(brackets) = brackets
                && brackets.delimiter() == Delimiter::Bracket
            {
                return Some(Self(brackets.stream().to_string()));
            }
        }
        None
    }

    fn as_str(&self) -> &str {
        &self.0
    }

    fn parse_vec(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Vec<Attr> {
        let mut out = Vec::new();
        while let Some(v) = Self::parse(stream) {
            out.push(v);
        }
        out
    }
}
