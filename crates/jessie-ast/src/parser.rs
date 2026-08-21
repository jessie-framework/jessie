use crate::ParseError;
use jessie_ast_lowering::{DefId, LowerAst};
use jessie_span::{SourceMap, Span};
use jessie_tokenizer::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    sm: &'a mut SourceMap,
    idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, sm: &'a mut SourceMap) -> Self {
        Self { tokens, sm, idx: 0 }
    }

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
                    span: Span::nul(),
                    kind: TokenKind::EOF,
                }
            },
            if let Some(&v) = self.tokens.get(self.idx + second_idx) {
                v
            } else {
                Token {
                    span: Span::nul(),
                    kind: TokenKind::EOF,
                }
            },
        )
    }

    fn peek_tok(&mut self) -> Token {
        self.skip_ws();
        if let Some(&v) = self.tokens.get(self.idx) {
            v
        } else {
            Token {
                span: Span::nul(),
                kind: TokenKind::EOF,
            }
        }
    }

    fn next_tok(&mut self) -> Token {
        self.skip_ws();
        self.idx += 1;
        if let Some(v) = self.tokens.get(self.idx - 1) {
            *v
        } else {
            Token {
                span: Span::nul(),
                kind: TokenKind::EOF,
            }
        }
    }

    fn peek_tok_ws(&mut self) -> Token {
        if let Some(&v) = self.tokens.get(self.idx) {
            v
        } else {
            Token {
                span: Span::nul(),
                kind: TokenKind::EOF,
            }
        }
    }

    fn next_tok_ws(&mut self) -> Token {
        self.idx += 1;
        if let Some(v) = self.tokens.get(self.idx - 1) {
            *v
        } else {
            Token {
                span: Span::nul(),
                kind: TokenKind::EOF,
            }
        }
    }

    fn skip_ws(&mut self) {
        while let Some(&v) = self.tokens.get(self.idx)
            && (v.kind == TokenKind::Whitespace || v.kind == TokenKind::Comment)
        {
            self.idx += 1;
        }
    }

    pub fn parse_doc(&mut self) -> Result<Document> {
        let mut items = vec![];
        let mut lo = None;
        let hi;
        loop {
            let attrs = self.parse_attrs()?;
            let kw = self.next_tok();
            if lo.is_none() {
                lo = Some(kw.span.lo());
            }
            if kw.kind == TokenKind::EOF {
                hi = Some(kw.span.hi());
                break;
            }
            match self.sm.span_str(kw.span) {
                "extends" => items.push(self.parse_extends(kw.span.lo(), attrs)?),
                "main" => items.push(self.parse_main(kw.span.lo(), attrs)?),
                "program" => items.push(self.parse_program(kw.span.lo(), attrs)?),
                "import" => items.push(self.parse_import(kw.span.lo(), attrs)?),
                _ => return Err(ParseError::UnexpectedValue),
            }
        }
        if let Some(lo) = lo
            && let Some(hi) = hi
        {
            let span = Span::new(lo, hi);
            Ok(Document { items, span })
        } else {
            Err(ParseError::UnexpectedValue)
        }
    }

    fn parse_import(&mut self, lo: u32, attrs: Vec<Attr>) -> Result<Item> {
        let (importing, sp) = self.must_string()?;
        Ok(Item {
            attrs,
            kind: ItemKind::Import(ImportStmt { importing }),
            span: Span::new(lo, sp.hi()),
        })
    }

    fn next_if(&mut self, kind: TokenKind) -> Option<Span> {
        if self.peek_tok().kind == kind {
            return Some(self.next_tok().span);
        }
        None
    }

    fn must_digit(&mut self) -> Result<(u128, Span)> {
        let next = self.next_tok();
        if let TokenKind::Digit = next.kind {
            return Ok((
                self.sm
                    .span_str(next.span)
                    .parse::<u128>()
                    .map_err(|_| ParseError::NumberConvertError { sp: next.span })?,
                next.span,
            ));
        }
        Err(ParseError::UnexpectedToken {
            expected: TokenKind::Digit,
            found: next.kind,
        })
    }

    fn parse_program(&mut self, lo: u32, attrs: Vec<Attr>) -> Result<Item> {
        let name = self.must_ident()?.0;
        let mut vs_path = None;
        let mut fs_path = None;
        let mut stride = None;
        let mut stream = None;
        self.skip(TokenKind::LCurly)?;
        for _ in 0..4u8 {
            let (ident, sp) = self.must_ident()?;
            self.skip(TokenKind::Colon)?;
            match ident.as_str() {
                "vs" => {
                    if vs_path.is_some() {
                        return Err(ParseError::ProgramAlreadyHasParameter { sp });
                    }
                    let str = self.must_string()?.0;
                    vs_path = Some(str);
                }
                "fs" => {
                    if fs_path.is_some() {
                        return Err(ParseError::ProgramAlreadyHasParameter { sp });
                    }
                    let str = self.must_string()?.0;
                    fs_path = Some(str);
                }
                "stride" => {
                    if stride.is_some() {
                        return Err(ParseError::ProgramAlreadyHasParameter { sp });
                    }
                    stride = Some(self.must_digit()?.0);
                }
                "stream" => {
                    if stream.is_some() {
                        return Err(ParseError::ProgramAlreadyHasParameter { sp });
                    }
                    stream = Some(self.parse_ident_list()?);
                }
                _ => return Err(ParseError::UnrecognizedProgramField { sp }),
            }
        }
        self.next_if(TokenKind::Comma);
        let hi = self.skip(TokenKind::RCurly)?.hi();
        if let Some(vs_path) = vs_path
            && let Some(fs_path) = fs_path
            && let Some(stride) = stride
            && let Some(stream) = stream
        {
            return Ok(Item {
                attrs,
                kind: ItemKind::Program(ProgramBlock {
                    name,
                    vs_path,
                    fs_path,
                    stride,
                    stream,
                }),
                span: Span::new(lo, hi),
            });
        }
        Err(ParseError::MissingProgramParameters {
            sp: Span::new(lo, hi),
        })
    }

    fn parse_ident_list(&mut self) -> Result<Vec<String>> {
        self.skip(TokenKind::LSquare)?;
        let mut out = vec![];
        loop {
            let peek = self.peek_tok();
            match peek.kind {
                TokenKind::Ident => {
                    out.push(self.must_ident()?.0);
                    let peek = self.peek_tok();
                    match peek.kind {
                        TokenKind::Comma => {
                            self.next_tok();
                            continue;
                        }
                        TokenKind::RSquare => {
                            return Ok(out);
                        }
                        e => {
                            return Err(ParseError::UnexpectedToken {
                                expected: TokenKind::Comma,
                                found: e,
                            });
                        }
                    }
                }
                TokenKind::RSquare => {
                    self.next_tok();
                    return Ok(out);
                }
                e => {
                    return Err(ParseError::UnexpectedToken {
                        expected: TokenKind::Ident,
                        found: e,
                    });
                }
            }
        }
    }

    fn parse_attrs(&mut self) -> Result<Vec<Attr>> {
        let mut out = vec![];
        while self.peek_tok().kind == TokenKind::At {
            self.next_tok();
            let (name, _sp) = self.must_ident()?;
            let inner = if let TokenKind::LParen = self.peek_tok().kind {
                self.next_tok();
                let (this, _tsp) = self.must_string()?;
                self.skip(TokenKind::RParen)?;
                Some(this)
            } else {
                None
            };
            out.push(Attr { name, inner });
        }
        Ok(out)
    }

    fn must_string(&mut self) -> Result<(String, Span)> {
        let next = self.next_tok();
        if next.kind == TokenKind::String {
            return Ok((
                self.sm
                    .span_str(Span::new(next.span.lo() + 1, next.span.hi() - 1))
                    .to_owned(),
                next.span,
            ));
        }
        Err(ParseError::UnexpectedToken {
            expected: TokenKind::String,
            found: next.kind,
        })
    }

    fn must_ident(&mut self) -> Result<(String, Span)> {
        let next = self.next_tok();
        if next.kind == TokenKind::Ident {
            return Ok((self.sm.span_str(next.span).to_owned(), next.span));
        }
        Err(ParseError::UnexpectedToken {
            expected: TokenKind::Ident,
            found: next.kind,
        })
    }

    fn parse_extends(&mut self, lo: u32, attrs: Vec<Attr>) -> Result<Item> {
        let extending = self.must_ident()?.0;
        let tree = self.parse_tree()?;
        let sp = tree.span;
        Ok(Item {
            kind: ItemKind::Extends(ExtendsBlock {
                span: Span::new(lo, sp.hi()),
                extending,
                tree,
            }),
            span: Span::new(lo, sp.hi()),
            attrs,
        })
    }

    fn parse_main(&mut self, lo: u32, attrs: Vec<Attr>) -> Result<Item> {
        let tree = self.parse_tree()?;
        let sp = tree.span;
        Ok(Item {
            kind: ItemKind::Main(MainBlock {
                span: Span::new(lo, sp.hi()),
                tree,
            }),
            span: Span::new(lo, sp.hi()),
            attrs,
        })
    }

    fn skip(&mut self, kind: TokenKind) -> Result<Span> {
        let next = self.next_tok();
        if next.kind == kind {
            Ok(next.span)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: kind,
                found: next.kind,
            })
        }
    }

    fn parse_tree(&mut self) -> Result<Tree> {
        let mut elements = vec![];
        let lo = self.skip(TokenKind::LCurly)?.lo();
        loop {
            let peek = self.peek_tok();
            match peek.kind {
                TokenKind::Lt => {
                    self.next_tok();
                    elements.push(self.parse_tag()?);
                }
                TokenKind::RCurly => {
                    self.next_tok();
                    return Ok(Tree {
                        elements,
                        span: Span::new(lo, peek.span.hi()),
                    });
                }
                _ => elements.push(self.parse_text_element(peek)?),
            }
        }
    }

    fn parse_text_element(&mut self, tok: Token) -> Result<Element> {
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
                    text.push_str(self.sm.span_str(escaped.span));
                }
                TokenKind::LCurly => {
                    if !text.is_empty() {
                        params.push(FormatTextParam {
                            is_formatted: false,
                            text: text.clone(),
                        });
                        text.clear();
                    }
                    self.skip(TokenKind::LCurly)?;
                    let (text, _) = self.must_ident()?;
                    self.skip(TokenKind::RCurly)?;
                    params.push(FormatTextParam {
                        is_formatted: true,
                        text,
                    });
                }
                TokenKind::Whitespace => {
                    self.next_tok_ws();
                    if !matches!(self.peek_tok_ws().kind, TokenKind::Lt | TokenKind::RCurly) {
                        text.push_str(self.sm.span_str(peek.span));
                    }
                }
                _ => {
                    self.next_tok_ws();
                    text.push_str(self.sm.span_str(peek.span));
                }
            }
            peek = self.peek_tok_ws();
        }
        Ok(Element::Text(TextElement { params }))
    }

    fn parse_element(&mut self) -> Result<Element> {
        let peek = self.peek_tok();
        match peek.kind {
            TokenKind::Lt => {
                self.next_tok();
                self.parse_tag()
            }
            _ => self.parse_text_element(peek),
        }
    }

    fn parse_tag(&mut self) -> Result<Element> {
        let mut props = vec![];
        let mut children = vec![];
        let (name, sp) = self.must_ident()?;
        loop {
            let next = self.next_tok();
            match next.kind {
                TokenKind::Slash => {
                    return Ok(Element::Single(SingleElement {
                        props,
                        name,
                        span: Span::new(sp.lo(), self.skip(TokenKind::Gt)?.hi()),
                    }));
                }
                TokenKind::Gt => {
                    while let (first, second) = self.peek_two()
                        && (first.kind, second.kind) != (TokenKind::Lt, TokenKind::Slash)
                    {
                        children.push(self.parse_element()?);
                    }
                }
                TokenKind::Lt => {
                    self.skip(TokenKind::Slash)?;
                    let (ident, ident_sp) = self.must_ident()?;
                    if name == ident {
                        let gt = self.skip(TokenKind::Gt)?;
                        return Ok(Element::Nested(NestedElement {
                            name,
                            span: Span::new(sp.lo(), gt.hi()),
                            children,
                            props,
                        }));
                    }
                    return Err(ParseError::ClosedElementThatWasntOpened { sp: ident_sp });
                }
                TokenKind::Ident => {
                    props.push(self.parse_prop(self.sm.span_str(next.span).to_owned())?);
                    continue;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: TokenKind::Gt,
                        found: next.kind,
                    });
                }
            }
        }
    }

    fn parse_prop(&mut self, name: String) -> Result<Prop> {
        let prop_name = name;
        self.skip(TokenKind::Eq)?;
        let prop_val = self.must_ident()?.0;
        Ok(Prop {
            prop_name,
            prop_val,
        })
    }
}

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document<K = ItemKind> {
    pub items: Vec<Item<K>>,
    pub span: Span,
}

impl<'a> LowerAst<'a> for Document<ItemKind> {
    type LowerTy = Document<LowerItemKind>;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        Document {
            items: self.items.into_iter().map(|v| v.lower(ctx)).collect(),
            span: self.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub name: String,
    pub inner: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item<K = ItemKind> {
    pub attrs: Vec<Attr>,
    pub kind: K,
    pub span: Span,
}

impl<'a> LowerAst<'a> for Item<ItemKind> {
    type LowerTy = Item<LowerItemKind>;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        Item {
            attrs: self.attrs,
            kind: self.kind.lower(ctx),
            span: self.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Main(MainBlock),
    Extends(ExtendsBlock),
    Program(ProgramBlock),
    Import(ImportStmt),
}

impl<'a> LowerAst<'a> for ItemKind {
    type LowerTy = LowerItemKind;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        match self {
            Self::Main(main_block) => LowerItemKind::Main(main_block.lower(ctx)),
            Self::Extends(extends_block) => LowerItemKind::Extends(extends_block.lower(ctx)),
            Self::Program(program_block) => LowerItemKind::Program(program_block.lower(ctx)),
            Self::Import(import_block) => LowerItemKind::Import(import_block),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerItemKind {
    Main(LowerMainBlock),
    Extends(LowerExtendsBlock),
    Program(LowerProgramBlock),
    Import(ImportStmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStmt {
    pub importing: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramBlock {
    pub name: String,
    pub vs_path: String,
    pub fs_path: String,
    pub stride: u128,
    pub stream: Vec<String>,
}

impl<'a> LowerAst<'a> for ProgramBlock {
    type LowerTy = LowerProgramBlock;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        LowerProgramBlock {
            name: self.name,
            vs_path: self.vs_path,
            fs_path: self.fs_path,
            stride: self.stride,
            stream: self.stream,
            did: ctx.new_did(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerProgramBlock {
    pub name: String,
    pub vs_path: String,
    pub fs_path: String,
    pub stride: u128,
    pub stream: Vec<String>,
    pub did: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainBlock {
    pub span: Span,
    pub tree: Tree,
}

impl<'a> LowerAst<'a> for MainBlock {
    type LowerTy = LowerMainBlock;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        LowerMainBlock {
            span: self.span,
            tree: self.tree,
            did: ctx.new_did(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerMainBlock {
    pub span: Span,
    pub tree: Tree,
    pub did: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendsBlock {
    pub span: Span,
    pub extending: String,
    pub tree: Tree,
}

impl<'a> LowerAst<'a> for ExtendsBlock {
    type LowerTy = LowerExtendsBlock;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        LowerExtendsBlock {
            span: self.span,
            extending: self.extending,
            tree: self.tree,
            did: ctx.new_did(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerExtendsBlock {
    pub span: Span,
    pub extending: String,
    pub tree: Tree,
    pub did: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ty {
    pub span: Span,
    pub string: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub elements: Vec<Element>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Nested(NestedElement),
    Single(SingleElement),
    Text(TextElement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextElement {
    pub params: Vec<FormatTextParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatTextParam {
    pub is_formatted: bool,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedElement {
    pub name: String,
    pub span: Span,
    pub children: Vec<Element>,
    pub props: Vec<Prop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleElement {
    pub props: Vec<Prop>,
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub prop_name: String,
    pub prop_val: String,
}
