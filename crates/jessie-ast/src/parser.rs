use jessie_rust_lex::TokenKind;
use jessie_session::Session;
use jessie_span::Span;

use crate::TokenStream;

pub struct Parser<'a> {
    ts: TokenStream,
    sess: &'a mut Session,
}

impl<'a> Parser<'a> {
    pub fn new(ts: TokenStream, sess: &'a mut Session) -> Self {
        Self { ts, sess }
    }

    fn must(&mut self, kind: TokenKind) {
        let next = self.ts.next();
        if next.kind != kind {
            self.sess
                .new_err_sp(next.span, &format!("expected {kind}, found {}", next.kind));
        }
    }
    fn must_ident(&mut self) -> String {
        let peek = self.ts.peek();
        match peek.kind {
            TokenKind::Ident => {
                self.ts.next();
                self.sess.sm.span_str(peek.span).to_string()
            }
            TokenKind::RawIdent => {
                self.ts.next();
                self.sess
                    .sm
                    .span_str(Span::new(peek.span.lo() + 2, peek.span.hi()))
                    .to_string()
            }
            e => {
                self.sess
                    .new_err_sp(peek.span, &format!("expected ident, found {e}"));
                String::new()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub items: Vec<Item>,
}

impl<'a> Parser<'a> {
    pub fn parse_document(&mut self) -> Document {
        let mut items = vec![];
        loop {
            if self.ts.peek().is_eof() {
                break;
            }
            items.push(self.parse_item());
        }
        Document { items }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub span: Span,
    pub kind: ItemKind,
}

impl<'a> Parser<'a> {
    pub fn parse_item(&mut self) -> Item {
        let lo = self.ts.peek().span.lo();
        let peek = self.ts.peek();
        let kind = match self.sess.sm.span_str(peek.span) {
            "uniform" => self.parse_uniform_stmt(),
            e => {
                self.sess
                    .new_err_sp(peek.span, &format!("expected keyword, found {e}"));
                ItemKind::Err
            }
        };
        let hi = self.ts.peek().span.hi();
        let span = Span::new(lo, hi);
        Item { span, kind }
    }
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Uniform(UniformStmt),
    Err,
}

#[derive(Debug, Clone)]
pub struct UniformStmt {
    pub name: String,
    pub ty: Ty,
}

impl<'a> Parser<'a> {
    pub fn parse_uniform_stmt(&mut self) -> ItemKind {
        self.ts.next();
        let name = self.must_ident();
        self.must(TokenKind::Colon);
        let ty = self.parse_ty();
        self.must(TokenKind::Semi);
        ItemKind::Uniform(UniformStmt { name, ty })
    }
}

#[derive(Debug, Clone)]
pub enum Ty {
    Never,
    Ptr(Mutability, Box<Self>),
    Ref(Option<Lt>, Mutability, Box<Self>),
    Tuple(Vec<Self>),
    Path(TyPath),
    Err,
}

#[derive(Debug, Clone)]
pub struct TyPath {
    pub is_root: bool,
    pub path: Vec<String>,
    pub generics: Option<Vec<Generic>>,
}

#[derive(Debug, Clone)]
pub enum Generic {
    Ty(Ty),
    Lt(Lt),
}

#[derive(Debug, Clone)]
pub struct Lt(String);

impl<'a> Parser<'a> {
    pub fn parse_lt(&mut self) -> Option<Lt> {
        let peek = self.ts.peek();
        match peek.kind {
            TokenKind::Lifetime => {
                self.ts.next();
                Some(Lt(self
                    .sess
                    .sm
                    .span_str(Span::new(peek.span.lo() + 1, peek.span.hi()))
                    .to_string()))
            }
            TokenKind::RawLifetime => {
                self.ts.next();
                Some(Lt(self
                    .sess
                    .sm
                    .span_str(Span::new(peek.span.lo() + 3, peek.span.hi()))
                    .to_string()))
            }
            _ => None,
        }
    }
}

impl Lt {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum Mutability {
    Mut,
    Const,
}

impl<'a> Parser<'a> {
    pub fn parse_ty(&mut self) -> Ty {
        let peek = self.ts.peek();
        use TokenKind as K;
        match peek.kind {
            K::Bang => self.parse_never_ty(),
            K::Star => self.parse_ptr_ty(),
            K::And => self.parse_ref_ty(),
            K::LParen => self.parse_tuple_ty(),
            K::Ident | K::RawIdent | K::ColonColon => self.parse_ty_path(),
            e => {
                self.sess
                    .new_err_sp(peek.span, &format!("expected type, found {e}"));
                Ty::Err
            }
        }
    }

    fn parse_never_ty(&self) -> Ty {
        self.ts.next();
        Ty::Never
    }

    fn parse_ptr_ty(&mut self) -> Ty {
        self.ts.next();
        let mutability = match self.sess.sm.span_str(self.ts.peek().span) {
            "mut" => {
                self.ts.next();
                Mutability::Mut
            }
            "const" => {
                self.ts.next();
                Mutability::Const
            }
            e => {
                self.sess.new_err_sp(
                    self.ts.peek().span,
                    &format!("expected one of mut or const , found {e}"),
                );
                Mutability::Const
            }
        };
        Ty::Ptr(mutability, Box::new(self.parse_ty()))
    }

    fn parse_ref_ty(&mut self) -> Ty {
        self.ts.next();
        let lt = self.parse_lt();
        let mutability = if self.ts.peek().span.len() == 3
            && self.sess.sm.span_str(self.ts.peek().span) == "mut"
        {
            self.ts.next();
            Mutability::Mut
        } else {
            Mutability::Const
        };
        Ty::Ref(lt, mutability, Box::new(self.parse_ty()))
    }

    fn parse_tuple_ty(&mut self) -> Ty {
        self.ts.next();
        let mut tys = vec![];
        loop {
            let peek = self.ts.peek();
            match peek.kind {
                TokenKind::RParen => {
                    self.ts.next();
                    return Ty::Tuple(tys);
                }
                _ => {
                    tys.push(self.parse_ty());
                    match self.ts.peek().kind {
                        TokenKind::Comma => {
                            self.ts.next();
                            continue;
                        }
                        TokenKind::RParen => {
                            continue;
                        }
                        e => {
                            self.sess.new_err_sp(
                                self.ts.peek().span,
                                &format!("expected one of , or ( , found {e}"),
                            );
                            return Ty::Tuple(tys);
                        }
                    }
                }
            }
        }
    }

    fn parse_ty_path_generics(&mut self) -> Option<Vec<Generic>> {
        if self.ts.peek().kind != TokenKind::Lt {
            return None;
        }
        self.ts.next();
        let mut out = vec![];
        loop {
            let peek = self.ts.peek();
            match peek.kind {
                TokenKind::Gt => {
                    self.ts.next();
                    return Some(out);
                }
                TokenKind::Lifetime | TokenKind::RawLifetime => {
                    out.push(Generic::Lt(self.parse_lt().unwrap()));
                    match self.ts.peek().kind {
                        TokenKind::Comma => {
                            self.ts.next();
                            continue;
                        }
                        TokenKind::Gt => {
                            continue;
                        }
                        e => {
                            self.sess.new_err_sp(
                                self.ts.peek().span,
                                &format!("expected one of , or < , found {e}"),
                            );
                            return Some(out);
                        }
                    }
                }
                _ => {
                    out.push(Generic::Ty(self.parse_ty()));
                    match self.ts.peek().kind {
                        TokenKind::Comma => {
                            self.ts.next();
                            continue;
                        }
                        TokenKind::Gt => {
                            continue;
                        }
                        e => {
                            self.sess.new_err_sp(
                                self.ts.peek().span,
                                &format!("expected one of , or < , found {e}"),
                            );
                            return Some(out);
                        }
                    }
                }
            }
        }
    }

    fn parse_ty_path(&mut self) -> Ty {
        let is_root = if self.ts.peek().kind == TokenKind::ColonColon {
            self.ts.next();
            true
        } else {
            false
        };

        let mut path = vec![];

        let peek = self.ts.peek();
        match peek.kind {
            TokenKind::RawIdent => {
                self.ts.next();
                path.push(
                    self.sess
                        .sm
                        .span_str(Span::new(peek.span.lo() + 2, peek.span.hi()))
                        .to_string(),
                );
            }
            TokenKind::Ident => {
                self.ts.next();
                path.push(self.sess.sm.span_str(peek.span).to_string());
            }
            e => {
                self.sess
                    .new_err_sp(peek.span, &format!("expected ident, found {e}"));
            }
        }

        if self.ts.peek().kind != TokenKind::ColonColon {
            let generics = self.parse_ty_path_generics();
            return Ty::Path(TyPath {
                is_root,
                path,
                generics,
            });
        }

        loop {
            let peek = self.ts.peek();
            match peek.kind {
                TokenKind::RawIdent => {
                    self.ts.next();
                    path.push(
                        self.sess
                            .sm
                            .span_str(Span::new(peek.span.lo() + 2, peek.span.hi()))
                            .to_string(),
                    );
                }
                TokenKind::Ident => {
                    self.ts.next();
                    path.push(self.sess.sm.span_str(peek.span).to_string());
                }
                e => {
                    self.sess
                        .new_err_sp(peek.span, &format!("expected ident, found {e}"));
                }
            }
            if self.ts.peek().kind == TokenKind::ColonColon {
                self.ts.next();
                continue;
            } else {
                break;
            }
        }
        let generics = self.parse_ty_path_generics();
        Ty::Path(TyPath {
            is_root,
            path,
            generics,
        })
    }
}
