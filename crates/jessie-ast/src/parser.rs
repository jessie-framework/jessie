use crate::ParseError;
use jessie_session::DefId;
use jessie_span::Span;
use jessie_tokenizer::{Token, TokenKind};

/// Parses tokens into an abstract-syntax-tree representation (AST).
pub struct Parser<'a> {
    tokens: Vec<Token>,
    sess: &'a mut jessie_session::Session,
    idx: usize,
    span: Span,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, sess: &'a mut jessie_session::Session, span: Span) -> Self {
        Self {
            tokens,
            sess,
            idx: 0,
            span,
        }
    }

    /// Checks two tokens ahead of the stream , skipping whitespace.
    fn peek_two(&mut self) -> (Token, Token) {
        let mut first_idx = 0;
        while let Some(&v) = self.tokens.get(self.idx + first_idx)
            && v.kind == TokenKind::Whitespace
        {
            first_idx += 1;
        }
        let mut second_idx = first_idx + 1;
        while let Some(&v) = self.tokens.get(self.idx + second_idx)
            && v.kind == TokenKind::Whitespace
        {
            second_idx += 1;
        }

        (
            if let Some(&v) = self.tokens.get(self.idx + first_idx) {
                v
            } else {
                Token {
                    span: self.span.shrink_to_hi(),
                    kind: TokenKind::EOF,
                }
            },
            if let Some(&v) = self.tokens.get(self.idx + second_idx) {
                v
            } else {
                Token {
                    span: self.span.shrink_to_hi(),
                    kind: TokenKind::EOF,
                }
            },
        )
    }

    /// Checks a token ahead of the stream, skipping whitespace.
    fn peek_tok(&mut self) -> Token {
        self.skip_ws();
        if let Some(&v) = self.tokens.get(self.idx) {
            v
        } else {
            Token {
                span: self.span.shrink_to_hi(),
                kind: TokenKind::EOF,
            }
        }
    }

    /// Consumes the next token in the stream, skipping whitespace.
    fn next_tok(&mut self) -> Token {
        self.skip_ws();
        self.idx += 1;
        if let Some(v) = self.tokens.get(self.idx - 1) {
            *v
        } else {
            Token {
                span: self.span.shrink_to_hi(),
                kind: TokenKind::EOF,
            }
        }
    }

    /// Checks for the next token in the stream, including whitespace.
    fn peek_tok_ws(&mut self) -> Token {
        if let Some(&v) = self.tokens.get(self.idx) {
            v
        } else {
            Token {
                span: self.span.shrink_to_hi(),
                kind: TokenKind::EOF,
            }
        }
    }

    /// Consumes the next token in the stream, including whitespace.
    fn next_tok_ws(&mut self) -> Token {
        self.idx += 1;
        if let Some(v) = self.tokens.get(self.idx - 1) {
            *v
        } else {
            Token {
                span: self.span.shrink_to_hi(),
                kind: TokenKind::EOF,
            }
        }
    }

    /// Skips the whitespace/comments in the stream.
    fn skip_ws(&mut self) {
        while let Some(&v) = self.tokens.get(self.idx)
            && (v.kind == TokenKind::Whitespace || v.kind == TokenKind::Comment)
        {
            self.idx += 1;
        }
    }

    /// Parses a document.
    pub fn parse_doc(&mut self) -> Document {
        let mut items = vec![];
        loop {
            let attrs = self.parse_attrs();
            let kw = self.next_tok();
            if kw.kind == TokenKind::EOF {
                break;
            }
            match self.sess.sm.span_str(kw.span) {
                "extends" => items.push(self.parse_extends(kw.span.lo(), attrs)),
                "entry" => items.push(self.parse_entry(kw.span.lo(), attrs)),
                "program" => items.push(self.parse_program(kw.span.lo(), attrs)),
                "import" => items.push(self.parse_import(kw.span.lo(), attrs)),
                err => {
                    self.sess.new_diag(ParseError::UnexpectedValue {
                        expected: "one of extends, main, program, import".into(),
                        found: err.to_string(),
                        sp: kw.span,
                    });
                }
            }
        }
        Document {
            items,
            span: self.span,
        }
    }

    /// Parses an `import` item.
    fn parse_import(&mut self, lo: u32, attrs: Vec<Attr>) -> Item {
        let importing = self.parse_path();
        let hi = importing.span.hi();
        Item {
            attrs,
            kind: ItemKind::Import(ImportStmt { importing }),
            span: Span::new(lo, hi),
        }
    }

    /// Consumes the next token in the stream if it is the same as `kind`. Returns the tokens span if so.
    fn next_if(&mut self, kind: TokenKind) -> Option<Span> {
        if self.peek_tok().kind == kind {
            return Some(self.next_tok().span);
        }
        None
    }

    /// Parses a `program` item.
    fn parse_program(&mut self, lo: u32, attrs: Vec<Attr>) -> Item {
        let name = self.must_ident();
        let mut vs_path = None;
        let mut fs_path = None;
        self.skip(TokenKind::LCurly);
        for _ in 0..2u8 {
            let ident = self.must_ident();
            self.skip(TokenKind::Colon);
            match self.sess.sm.span_str(ident.span()) {
                "vs" => {
                    if vs_path.is_some() {
                        self.sess.new_diag(ParseError::ProgramAlreadyHasParameter {
                            sp: ident.span(),
                            param: "vs",
                        });
                        return Item::err(ident.span());
                    }
                    vs_path = Some(self.parse_path());
                }
                "fs" => {
                    if fs_path.is_some() {
                        self.sess.new_diag(ParseError::ProgramAlreadyHasParameter {
                            sp: ident.span(),
                            param: "fs",
                        });
                        return Item::err(ident.span());
                    }
                    fs_path = Some(self.parse_path());
                }
                _ => return Item::err(ident.span()),
            }
            if self.peek_tok().kind == TokenKind::Comma {
                self.next_tok();
                continue;
            } else {
                break;
            }
        }
        self.next_if(TokenKind::Comma);
        let hi = self.skip(TokenKind::RCurly).hi();
        let mut missing_params = vec![];
        if vs_path.is_none() {
            missing_params.push("vs");
        }
        if fs_path.is_none() {
            missing_params.push("fs");
        }
        if let Some(vs_path) = vs_path
            && let Some(fs_path) = fs_path
        {
            return Item {
                attrs,
                kind: ItemKind::Program(ProgramBlock {
                    name,
                    vs_path,
                    fs_path,
                    did: self.sess.new_did(),
                }),
                span: Span::new(lo, hi),
            };
        }
        self.sess.new_diag(ParseError::MissingProgramParameters {
            sp: Span::new(lo, hi),
            missing_params,
        });
        Item::err(Span::new(lo, hi))
    }

    /// Parses a list of attributes before parsing the next item.
    fn parse_attrs(&mut self) -> Vec<Attr> {
        let mut out = vec![];
        while self.peek_tok().kind == TokenKind::At {
            self.next_tok();
            let name = self.must_ident();
            let inner = if let TokenKind::LParen = self.peek_tok().kind {
                self.next_tok();
                let (this, _tsp) = self.must_string();
                self.skip(TokenKind::RParen);
                Some(this)
            } else {
                None
            };
            out.push(Attr { name, inner });
        }
        out
    }

    /// Consumes a string token, returning the inner string and the span of the entire string token (including the quotes). Errors if the consumed token wasn't a string token.
    fn must_string(&mut self) -> (String, Span) {
        let next = self.next_tok();
        if next.kind == TokenKind::String {
            (
                self.sess
                    .sm
                    .span_str(Span::new(next.span.lo() + 1, next.span.hi() - 1))
                    .to_owned(),
                next.span,
            )
        } else {
            self.sess.new_diag(ParseError::UnexpectedToken {
                expected: TokenKind::String,
                found: next.kind,
                sp: next.span,
            });
            (self.sess.sm.span_str(next.span).to_owned(), next.span)
        }
    }

    /// Consumes an indentation, raises an error if the consumed token wasn't an ident token.
    fn must_ident(&mut self) -> Ident {
        let next = self.next_tok();
        if next.kind != TokenKind::Ident {
            self.sess.new_diag(ParseError::UnexpectedToken {
                expected: TokenKind::Ident,
                found: next.kind,
                sp: next.span,
            });
        }
        Ident(next.span)
    }

    /// Parses an `extends` item.
    fn parse_extends(&mut self, lo: u32, attrs: Vec<Attr>) -> Item {
        let extending = self.must_ident();
        let tree = self.parse_tree();
        let sp = tree.span;
        Item {
            kind: ItemKind::Extends(ExtendsBlock {
                span: Span::new(lo, sp.hi()),
                extending,
                tree,
                did: self.sess.new_did(),
            }),
            span: Span::new(lo, sp.hi()),
            attrs,
        }
    }

    /// Parses a `entry` item.
    fn parse_entry(&mut self, lo: u32, attrs: Vec<Attr>) -> Item {
        let tree = self.parse_tree();
        let sp = tree.span;
        Item {
            kind: ItemKind::Entry(EntryBlock {
                span: Span::new(lo, sp.hi()),
                tree,
                did: self.sess.new_did(),
            }),
            span: Span::new(lo, sp.hi()),
            attrs,
        }
    }

    /// Consumes the next token in the stream and checks if it is the same as `kind`. If not, it gives out a compile error.
    fn skip(&mut self, kind: TokenKind) -> Span {
        let next = self.next_tok();
        if next.kind != kind {
            self.sess.new_diag(ParseError::UnexpectedToken {
                expected: kind,
                found: next.kind,
                sp: next.span,
            });
        }
        next.span
    }

    /// Parses a tree, the structure that can be found inside the curly braces of an `extends` or `entry` item.
    fn parse_tree(&mut self) -> Tree {
        let mut elements = vec![];
        let lo = self.skip(TokenKind::LCurly).lo();
        loop {
            let peek = self.peek_tok();
            match peek.kind {
                TokenKind::Lt => {
                    self.next_tok();
                    elements.push(self.parse_tag());
                }
                TokenKind::RCurly => {
                    self.next_tok();
                    return Tree {
                        elements,
                        span: Span::new(lo, peek.span.hi()),
                    };
                }
                TokenKind::EOF => {
                    self.sess.new_diag(ParseError::UnexpectedToken {
                        expected: TokenKind::RCurly,
                        found: TokenKind::EOF,
                        sp: peek.span,
                    });
                    return Tree {
                        elements,
                        span: Span::new(lo, peek.span.hi()),
                    };
                }

                _ => elements.push(self.parse_text_element(peek)),
            }
        }
    }

    /// Parses a text element in a tree.
    fn parse_text_element(&mut self, tok: Token) -> Element {
        let mut params = vec![];

        let mut text = String::new();

        let mut peek = tok;

        loop {
            match peek.kind {
                TokenKind::Lt | TokenKind::RCurly => {
                    if !text.is_empty() {
                        params.push(FormatTextParam {
                            is_formatted: false,
                            text: text.clone(),
                        });
                        text.clear();
                    }
                    break;
                }
                TokenKind::Backslash => {
                    self.next_tok();
                    let escaped = self.next_tok();
                    text.push_str(self.sess.sm.span_str(escaped.span));
                }
                TokenKind::LCurly => {
                    if !text.is_empty() {
                        params.push(FormatTextParam {
                            is_formatted: false,
                            text: text.clone(),
                        });
                        text.clear();
                    }
                    self.skip(TokenKind::LCurly);
                    let text = self.must_ident();
                    self.skip(TokenKind::RCurly);
                    params.push(FormatTextParam {
                        is_formatted: true,
                        text: self.sess.sm.span_str(text.span()).into(),
                    });
                }
                TokenKind::Whitespace => {
                    self.next_tok_ws();
                    if !matches!(self.peek_tok_ws().kind, TokenKind::Lt | TokenKind::RCurly) {
                        text.push_str(self.sess.sm.span_str(peek.span));
                    }
                }
                TokenKind::EOF => {
                    self.sess.new_diag(ParseError::UnexpectedToken {
                        expected: TokenKind::RCurly,
                        found: TokenKind::EOF,
                        sp: peek.span,
                    });
                    break;
                }
                _ => {
                    self.next_tok_ws();
                    text.push_str(self.sess.sm.span_str(peek.span));
                }
            }
            peek = self.peek_tok_ws();
        }
        Element::Text(TextElement { params })
    }

    /// Parses an element in a tree, the structure that can be found inside the curly braces of an `extends` or `entry` item.
    fn parse_element(&mut self) -> Element {
        let peek = self.peek_tok();
        match peek.kind {
            TokenKind::Lt => {
                self.next_tok();
                self.parse_tag()
            }
            _ => self.parse_text_element(peek),
        }
    }

    /// Parses a tag item.
    fn parse_tag(&mut self) -> Element {
        let mut props = vec![];
        let mut children = vec![];
        let name = self.parse_path();
        loop {
            let peek = self.peek_tok();
            match peek.kind {
                TokenKind::Slash => {
                    self.next_tok();
                    let lo = name.span.lo();
                    return Element::Single(SingleElement {
                        props,
                        name,
                        span: Span::new(lo, self.skip(TokenKind::Gt).hi()),
                    });
                }
                TokenKind::Gt => {
                    self.next_tok();
                    while let (first, second) = self.peek_two()
                        && (first.kind, second.kind) != (TokenKind::Lt, TokenKind::Slash)
                    {
                        children.push(self.parse_element());
                    }
                }
                TokenKind::Lt => {
                    self.next_tok();
                    self.skip(TokenKind::Slash);
                    let path = self.parse_path();
                    if name.as_string(&self.sess.sm) == path.as_string(&self.sess.sm) {
                        let gt = self.skip(TokenKind::Gt);
                        let lo = name.span.lo();
                        return Element::Nested(NestedElement {
                            name,
                            span: Span::new(lo, gt.hi()),
                            children,
                            props,
                        });
                    }
                    self.sess
                        .new_diag(ParseError::ClosedElementThatWasntOpened {
                            sp: Span::new(peek.span.lo(), path.span.hi()),
                        });
                    return Element::Err;
                }
                TokenKind::Ident => {
                    props.push(self.parse_prop_not_single());
                    continue;
                }
                TokenKind::LCurly => {
                    props.push(self.parse_prop_single());
                    continue;
                }
                _ => {
                    self.sess.new_diag(ParseError::UnexpectedToken {
                        expected: TokenKind::Gt,
                        found: peek.kind,
                        sp: peek.span,
                    });
                    return Element::Err;
                }
            }
        }
    }

    /// Parses a single prop in tag. {it_looks_like_this}
    fn parse_prop_single(&mut self) -> Prop {
        let lo = self.skip(TokenKind::LCurly).lo();
        let ident = self.must_ident();
        let hi = self.skip(TokenKind::RCurly).hi();
        let sp = Span::new(lo, hi);
        Prop::Single(sp, ident)
    }

    /// Parses a non single prop in a tag. it_looks_like=this
    fn parse_prop_not_single(&mut self) -> Prop {
        let prop_name = self.must_ident();
        self.skip(TokenKind::Eq);
        let prop_val = self.must_ident();
        Prop::NotSingle {
            prop_name,
            prop_val,
        }
    }

    /// Parses a path. A path can be found at the right side of an `import` statement.
    fn parse_path(&mut self) -> Path {
        let mut qualifiers = vec![];
        let (lo, mut hi) = (self.peek_tok().span.lo(), self.peek_tok().span.hi());
        loop {
            let next = self.next_tok();
            match next.kind {
                TokenKind::At => {
                    let id = self.must_ident();
                    qualifiers.push(PathQ::WithAt(Span::new(next.span.lo(), id.span().hi())));
                }
                TokenKind::Ident => {
                    if self.peek_tok().kind == TokenKind::Dot {
                        self.next_tok();
                        let hi = self.must_ident().span().hi();
                        qualifiers.push(PathQ::FileName(Span::new(lo, hi)));
                    } else {
                        qualifiers.push(PathQ::WithoutAt(next.span));
                    }
                }
                TokenKind::Star => {
                    qualifiers.push(PathQ::WildCard(next.span));
                }
                TokenKind::LParen => {
                    let mut inner = vec![];
                    loop {
                        let peek = self.peek_tok();
                        match peek.kind {
                            TokenKind::At
                            | TokenKind::Ident
                            | TokenKind::Star
                            | TokenKind::LParen => {
                                inner.push(self.parse_path());
                                if self.peek_tok().kind == TokenKind::Comma {
                                    self.next_tok();
                                    continue;
                                } else {
                                    break;
                                }
                            }
                            TokenKind::RParen => {
                                break;
                            }
                            e => {
                                self.sess.new_diag(ParseError::UnexpectedValue {
                                    expected: "one of @, ident, * , (".into(),
                                    found: e.as_str().into(),
                                    sp: peek.span,
                                });
                                break;
                            }
                        }
                    }
                    qualifiers.push(PathQ::List(
                        Span::new(next.span.lo(), self.skip(TokenKind::RParen).hi()),
                        inner,
                    ));
                }
                e => {
                    self.sess.new_diag(ParseError::UnexpectedToken {
                        expected: TokenKind::Ident,
                        found: e,
                        sp: next.span,
                    });
                    break;
                }
            }
            hi = next.span.hi();
            if self.peek_tok().kind == TokenKind::Colon {
                hi = self.peek_tok().span.hi();
                self.next_tok();
                continue;
            } else {
                break;
            }
        }
        Path {
            qualifiers,
            span: Span::new(lo, hi),
        }
    }
}

/// A syntax representation of an entire .jessie file.
#[derive(Debug, Clone)]
pub struct Document<K = ItemKind> {
    pub items: Vec<Item<K>>,
    pub span: Span,
}

/// An attribute to an item. @looks_like_this("with an inner string")
#[derive(Debug, Clone)]
pub struct Attr {
    pub name: Ident,
    pub inner: Option<String>,
}

/// A file wide item. One of `extends`, `program`, `entry` or `import`.
#[derive(Debug, Clone)]
pub struct Item<K = ItemKind> {
    pub attrs: Vec<Attr>,
    pub kind: K,
    pub span: Span,
}

impl Item<ItemKind> {
    pub(crate) fn err(span: Span) -> Self {
        Self {
            attrs: vec![],
            kind: ItemKind::Err,
            span,
        }
    }
}

/// An indentation. Starts with either an ASCII alphabetical character or `_`. Consists of ASCII alphanumerical characters (A-Z , 0-9) or underscores.
/// it_looks_like_this
#[derive(Debug, Clone, Copy, Hash)]
pub struct Ident(Span);

impl Ident {
    #[inline]
    pub const fn span(self) -> Span {
        self.0
    }
}

/// Kind of an item.
#[derive(Debug, Clone)]
pub enum ItemKind {
    Entry(EntryBlock),
    Extends(ExtendsBlock),
    Program(ProgramBlock),
    Import(ImportStmt),
    Err,
}

impl ItemKind {
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err)
    }
}

/// An import statement. Brings an external item into the file scope.
#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub importing: Path,
}

/// A program block. Allows for importing of shaders into the Jessie langauge.
#[derive(Debug, Clone)]
pub struct ProgramBlock {
    pub name: Ident,
    pub vs_path: Path,
    pub fs_path: Path,
    pub did: DefId,
}

/// An `entry` block. Represents the abstract idea of the root of an application/window with optional external data provided.
#[derive(Debug, Clone)]
pub struct EntryBlock {
    pub span: Span,
    pub tree: Tree,
    pub did: DefId,
}

/// An `extends` block. Extends the definition of an exactly same named item in a `.rs` file.
#[derive(Debug, Clone)]
pub struct ExtendsBlock {
    pub span: Span,
    pub extending: Ident,
    pub tree: Tree,
    pub did: DefId,
}

/// A tree. Its the data structure that can be found inside an `entry` or an `extends` item. It represents the visual hierarchy of a component.
#[derive(Debug, Clone)]
pub struct Tree {
    pub elements: Vec<Element>,
    pub span: Span,
}

/// An element in a tree. Can be a simple text element(Text), or a tag with children(Nested), or a simple tag with no children(Single)
#[derive(Debug, Clone)]
pub enum Element {
    Nested(NestedElement),
    Single(SingleElement),
    Text(TextElement),
    Err,
}

/// A text element. Can be formatted using {curly_braces}
#[derive(Debug, Clone)]
pub struct TextElement {
    pub params: Vec<FormatTextParam>,
}

/// The building block of a text element. Can either be formatted, which means being delimited by curly braces, or not.
#[derive(Debug, Clone)]
pub struct FormatTextParam {
    pub is_formatted: bool,
    pub text: String,
}

/// A tag element in the tree with children.
#[derive(Debug, Clone)]
pub struct NestedElement {
    pub name: Path,
    pub span: Span,
    pub children: Vec<Element>,
    pub props: Vec<Prop>,
}

/// A tag element in the tree without children.
#[derive(Debug, Clone)]
pub struct SingleElement {
    pub props: Vec<Prop>,
    pub name: Path,
    pub span: Span,
}

/// A prop. They are values that can be set in a component.
#[derive(Debug, Clone)]
pub enum Prop {
    Single(Span, Ident), // a prop with the same name as its value. {likethis}
    NotSingle { prop_name: Ident, prop_val: Ident },
}

impl Prop {
    pub fn span(&self) -> Span {
        match self {
            Self::Single(sp, _) => *sp,
            Self::NotSingle {
                prop_name,
                prop_val,
            } => Span::new(prop_name.span().lo(), prop_val.span().hi()),
        }
    }
}

/// A path to a Jessie item.
#[derive(Debug, Clone)]
pub struct Path {
    pub qualifiers: Vec<PathQ>,
    pub span: Span,
}

impl Path {
    pub fn as_string(&self, sm: &jessie_span::SourceMap) -> String {
        let mut out = String::new();
        let mut iter = self.qualifiers.iter().peekable();
        while let Some(next) = iter.next() {
            out.push_str(sm.span_str(next.span()));
            if iter.peek().is_some() {
                out.push(':');
            }
        }
        out
    }
}

/// A path qualifier.
#[derive(Debug, Clone)]
pub enum PathQ {
    WithAt(Span),          // @likethis
    WithoutAt(Span),       // likethis
    FileName(Span),        // like.this
    WildCard(Span),        // * <- like that
    List(Span, Vec<Path>), // (like,this)
}

impl PathQ {
    pub fn span(&self) -> Span {
        match self {
            Self::WithAt(sp) => *sp,
            Self::WithoutAt(sp) => *sp,
            Self::WildCard(sp) => *sp,
            Self::FileName(sp) => *sp,
            Self::List(sp, _) => *sp,
        }
    }
}
