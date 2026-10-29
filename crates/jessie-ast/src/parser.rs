use jessie_rust_lex::{TokenKind, diagnostics::UnexpectedToken};
use jessie_session::Session;
use jessie_span::Span;

use crate::{
    TokenStream,
    ast::{
        Block, Bound, Document, Expr, ExprKind, GenericArg, Ident, Item, ItemKind, Lifetime, Lit,
        Mutability, Pattern, Stmt, StmtKind, TraitBoundModifier, Ty, TyKind, TyPath, Unsafeness,
    },
    diagnostics::{
        ExpectedBound, ExpectedExpr, ExpectedItem, ExpectedLit, ExpectedPattern, ExpectedStmt,
        ExpectedType,
    },
};

pub struct Parser<'a> {
    ts: TokenStream,
    current_node_id: u32,
    sess: &'a mut Session,
}

impl<'a> Parser<'a> {
    pub fn new(ts: TokenStream, sess: &'a mut Session) -> Self {
        Self {
            ts,
            sess,
            current_node_id: 0,
        }
    }

    fn new_node_id(&mut self) -> crate::ast::NodeId {
        self.current_node_id += 1;
        crate::ast::NodeId {
            private_use_as_methods_instead: self.current_node_id,
        }
    }

    fn must(&mut self, kind: TokenKind) {
        let next = self.ts.next();
        if next.kind != kind {
            self.sess
                .new_err_sp(next.span, &format!("expected {kind}, found {}", next.kind));
        }
    }
}

impl<'a> Parser<'a> {
    pub fn parse_ty(&mut self) -> Ty {
        let id = self.new_node_id();
        let lo = self.ts.peek().span.lo();
        let kind = match self.ts.peek().kind {
            TokenKind::Impl => self.parse_impl_ty(),
            TokenKind::Dyn => self.parse_dyn_ty(),
            TokenKind::Ident => self.parse_typath_ty(),
            TokenKind::Bang => self.parse_never_ty(),
            TokenKind::LParen => self.parse_tuple_ty(),
            TokenKind::And => self.parse_ref_ty(),
            TokenKind::Star => self.parse_ptr_ty(),
            TokenKind::LSquare => self.parse_array_ty(),
            TokenKind::Lt => self.parse_qpath_ty(),
            TokenKind::For | TokenKind::Unsafe | TokenKind::Extern | TokenKind::Fn => {
                self.parse_fnptr_ty()
            }
            e => {
                self.sess.new_diag(ExpectedType {
                    found: e,
                    span: self.ts.peek().span,
                });
                TyKind::Err
            }
        };
        let hi = self.ts.peek().span.lo() - 1;
        let span = Span::new(lo, hi);
        Ty { kind, span, id }
    }

    fn parse_fnptr_ty(&mut self) -> TyKind {
        let _modifier = if self.ts.peek().kind == TokenKind::For {
            self.ts.next();
            let args = self.parse_generic_args();
            Some(TraitBoundModifier::For(args))
        } else {
            None
        };
        let _unsafeness = if self.ts.peek().kind == TokenKind::Unsafe {
            self.ts.next();
            Unsafeness::Unsafe
        } else {
            Unsafeness::Default
        };
        TyKind::Err // TODO
    }

    fn parse_qpath_ty(&mut self) -> TyKind {
        self.must(TokenKind::Lt);
        let ty = Box::new(self.parse_ty());
        self.must(TokenKind::As);
        let path1 = self.parse_ty_path();
        self.must(TokenKind::Gt);
        let path2 = self.parse_ty_path();
        TyKind::QPath(crate::ast::QSelf { ty, path: path1 }, path2)
    }

    fn parse_array_ty(&mut self) -> TyKind {
        self.must(TokenKind::LSquare);
        let ty = Box::new(self.parse_ty());
        match self.ts.peek().kind {
            TokenKind::Semi => {
                self.ts.next();
                let expr = Box::new(self.parse_expr());
                self.must(TokenKind::RSquare);
                TyKind::Array(ty, expr)
            }
            TokenKind::RSquare => {
                self.ts.next();
                TyKind::Slice(ty)
            }
            e => {
                self.sess.new_diag(UnexpectedToken {
                    expected: TokenKind::RSquare,
                    found: e,
                    span: self.ts.peek().span,
                });
                TyKind::Slice(ty)
            }
        }
    }

    fn parse_ptr_ty(&mut self) -> TyKind {
        self.must(TokenKind::Star);
        let mutability = match self.ts.peek().kind {
            TokenKind::Mut => {
                self.ts.next();
                Mutability::Mut
            }
            TokenKind::Const => {
                self.ts.next();
                Mutability::Not
            }
            e => {
                self.sess.new_diag(UnexpectedToken {
                    expected: TokenKind::Mut,
                    found: e,
                    span: self.ts.peek().span,
                });
                Mutability::Not
            }
        };
        let ty = Box::new(self.parse_ty());
        TyKind::RawPtr(crate::ast::MutTy { mutability, ty })
    }

    fn parse_ref_ty(&mut self) -> TyKind {
        self.must(TokenKind::And);
        let lifetime = match self.ts.peek().kind {
            TokenKind::Lifetime | TokenKind::RawLifetime => Some(self.parse_lifetime()),
            _ => None,
        };
        let mutability = match self.ts.peek().kind {
            TokenKind::Mut => {
                self.ts.next();
                Mutability::Mut
            }
            _ => Mutability::Not,
        };
        let ty = Box::new(self.parse_ty());
        TyKind::Ref(lifetime, crate::ast::MutTy { mutability, ty })
    }

    fn parse_tuple_ty(&mut self) -> TyKind {
        self.must(TokenKind::LParen);
        let mut out = vec![];
        loop {
            if self.ts.peek().kind == TokenKind::RParen {
                break;
            }
            out.push(self.parse_ty());
            if self.ts.peek().kind == TokenKind::Comma {
                self.ts.next();
                continue;
            } else {
                break;
            }
        }
        self.must(TokenKind::RParen);
        TyKind::Tuple(out)
    }

    fn parse_never_ty(&mut self) -> TyKind {
        self.must(TokenKind::Bang);
        TyKind::Never
    }

    fn parse_typath_ty(&mut self) -> TyKind {
        let path = self.parse_ty_path();
        TyKind::TyPath(path)
    }

    fn parse_impl_ty(&mut self) -> TyKind {
        self.ts.next();
        let bounds = self.parse_bounds();
        TyKind::Impl(bounds)
    }

    fn parse_dyn_ty(&mut self) -> TyKind {
        self.ts.next();
        let bounds = self.parse_bounds();
        TyKind::TraitObject(bounds, crate::ast::TraitObjectSyntax::Dyn)
    }

    fn parse_bounds(&mut self) -> Vec<Bound> {
        let mut out = vec![];
        loop {
            match self.ts.peek().kind {
                TokenKind::Lifetime | TokenKind::RawLifetime => {
                    out.push(Bound::Lifetime(self.parse_lifetime()));
                }
                TokenKind::Mark => {
                    self.ts.next();
                    out.push(Bound::TraitBound(crate::ast::TraitBound {
                        modifier: Some(TraitBoundModifier::Qm),
                        path: self.parse_ty_path(),
                    }));
                }
                TokenKind::For => {
                    self.ts.next();
                    out.push(Bound::TraitBound(crate::ast::TraitBound {
                        modifier: Some(TraitBoundModifier::For(self.parse_generic_args())),
                        path: self.parse_ty_path(),
                    }));
                }
                TokenKind::Use => {
                    self.ts.next();
                    out.push(Bound::UseBound(crate::ast::UseBound {
                        args: self.parse_generic_args(),
                    }));
                }
                TokenKind::Ident | TokenKind::RawIdent => {
                    out.push(Bound::TraitBound(crate::ast::TraitBound {
                        modifier: None,
                        path: self.parse_ty_path(),
                    }));
                }
                e => {
                    self.sess.new_diag(ExpectedBound {
                        found: e,
                        span: self.ts.peek().span,
                    });
                    return out;
                }
            }

            if self.ts.peek().kind == TokenKind::Plus {
                self.ts.next();
                continue;
            } else {
                break;
            }
        }
        out
    }

    fn parse_generic_args(&mut self) -> Vec<GenericArg> {
        let mut out = vec![];
        self.must(TokenKind::Lt);
        loop {
            if self.ts.peek().kind == TokenKind::Gt {
                break;
            }
            let peek = self.ts.peek();
            match peek.kind {
                TokenKind::Lifetime | TokenKind::RawLifetime => {
                    out.push(GenericArg::Lt(self.parse_lifetime()));
                }
                TokenKind::Impl | TokenKind::Dyn | TokenKind::Ident => {
                    out.push(GenericArg::Ty(self.parse_ty()));
                }
                _ => {
                    out.push(GenericArg::Const(self.parse_expr()));
                }
            }
            if self.ts.peek().kind == TokenKind::Comma {
                self.ts.next();
                continue;
            } else {
                break;
            }
        }
        self.must(TokenKind::Gt);
        out
    }

    fn parse_pattern(&mut self) -> Pattern {
        let pat = match self.ts.peek().kind {
            TokenKind::Ident | TokenKind::RawIdent => {
                let ident = self.parse_ident();
                Pattern::Ident(ident)
            }

            e => {
                self.sess.new_diag(ExpectedPattern {
                    found: e,
                    span: self.ts.peek().span,
                });

                Pattern::Err
            }
        };
        match self.ts.peek().kind {
            TokenKind::Colon => {
                self.ts.next();
                let ty = Box::new(self.parse_ty());
                Pattern::TyBinding(Box::new(pat), ty)
            }
            _ => pat,
        }
    }

    fn parse_let_stmt(&mut self) -> StmtKind {
        self.must(TokenKind::Let);
        let pat = self.parse_pattern();

        let expr = match self.ts.peek().kind {
            TokenKind::Eq => {
                self.ts.next();
                Some(self.parse_expr())
            }
            _ => None,
        };

        StmtKind::Let(crate::ast::StmtLet { pat, expr })
    }

    fn parse_stmt(&mut self) -> Stmt {
        let id = self.new_node_id();
        let lo = self.ts.peek().span.lo();
        let kind = match self.ts.peek().kind {
            TokenKind::Semi => {
                self.ts.next();
                StmtKind::Semi
            }
            TokenKind::Let => self.parse_let_stmt(),
            e => {
                self.sess.new_diag(ExpectedStmt {
                    found: e,
                    span: self.ts.peek().span,
                });
                StmtKind::Err
            }
        };
        let hi = self.ts.peek().span.hi() - 1;
        let span = Span::new(lo, hi);
        Stmt { span, kind, id }
    }

    fn parse_item(&mut self) -> Item {
        let id = self.new_node_id();
        let lo = self.ts.peek().span.lo();
        let kind = match self.ts.peek().kind {
            TokenKind::Fn => self.parse_fn_item(),
            e => {
                self.ts.next();
                self.sess.new_diag(ExpectedItem {
                    found: e,
                    span: self.ts.peek().span,
                });
                ItemKind::Err
            }
        };
        let hi = self.ts.peek().span.hi() - 1;
        let span = Span::new(lo, hi);
        Item { id, kind, span }
    }

    pub fn parse_document(&mut self) -> Document {
        let mut items = vec![];
        while self.ts.peek().kind != TokenKind::EOF {
            items.push(self.parse_item());
        }
        Document { items }
    }

    fn parse_fn_item(&mut self) -> ItemKind {
        self.must(TokenKind::Fn);
        let unsafeness = Unsafeness::Default; // todo
        let name = self.parse_ident();
        let generics = None; // todo
        let params = self.parse_fn_params();
        let ret_ty = match self.ts.peek().kind {
            TokenKind::Arr => {
                self.ts.next();
                Some(self.parse_ty())
            }
            _ => None,
        };
        let block = self.parse_block();
        ItemKind::Fn(crate::ast::ItemFn {
            unsafeness,
            name,
            generics,
            params,
            ret_ty,
            block,
        })
    }

    fn parse_block(&mut self) -> Block {
        self.must(TokenKind::LCurly);
        let mut stmts = vec![];
        loop {
            match self.ts.peek().kind {
                TokenKind::EOF | TokenKind::RCurly => {
                    break;
                }
                _ => {}
            }
            stmts.push(self.parse_stmt());
        }
        self.must(TokenKind::RCurly);

        Block { stmts }
    }

    fn parse_fn_params(&mut self) -> Vec<Pattern> {
        let mut out = vec![];
        self.must(TokenKind::LParen);
        loop {
            if self.ts.peek().kind == TokenKind::RParen {
                break;
            }
            out.push(self.parse_pattern());

            if self.ts.peek().kind == TokenKind::Comma {
                self.ts.next();
                continue;
            } else {
                break;
            }
        }
        self.must(TokenKind::RParen);
        out
    }

    fn parse_ident(&mut self) -> Ident {
        let peek = self.ts.peek();
        match peek.kind {
            TokenKind::Ident => {
                self.ts.next();
                Ident {
                    span: peek.span,
                    src: self.sess.sm.new_sym_from_span(peek.span),
                }
            }
            TokenKind::RawIdent => {
                self.ts.next();
                Ident {
                    span: peek.span,
                    src: self
                        .sess
                        .sm
                        .new_sym_from_span(Span::new(peek.span.lo() + 2, peek.span.hi())),
                }
            }
            e => {
                self.sess.new_diag(UnexpectedToken {
                    expected: TokenKind::Ident,
                    found: e,
                    span: peek.span,
                });
                Ident {
                    span: peek.span,
                    src: self.sess.sm.new_sym_from_span(peek.span),
                }
            }
        }
    }

    fn parse_ty_path(&mut self) -> TyPath {
        let is_root = match self.ts.peek().kind {
            TokenKind::ColonColon => {
                self.ts.next();
                true
            }
            _ => false,
        };
        let mut segments = vec![];

        while let TokenKind::Ident = self.ts.peek().kind {
            let ident = self.parse_ident();

            let [tok1, tok2] = self.ts.peek_n::<2>();

            let generics = match (tok1.kind, tok2.kind) {
                (TokenKind::ColonColon, TokenKind::Lt) => {
                    self.ts.next();
                    Some(self.parse_generic_args())
                }
                (TokenKind::Lt, _) => Some(self.parse_generic_args()),
                _ => None,
            };

            segments.push(crate::ast::TyPathSegment { ident, generics });

            match self.ts.peek().kind {
                TokenKind::ColonColon => {
                    self.ts.next();
                    continue;
                }
                _ => break,
            }
        }
        TyPath { is_root, segments }
    }

    fn parse_lit(&mut self) -> Lit {
        let peek = self.ts.peek();
        match peek.kind {
            TokenKind::True | TokenKind::False => {
                self.ts.next();
                Lit {
                    kind: jessie_rust_lex::LitKind::Bool,
                    symbol: self.sess.sm.new_sym_from_span(peek.span),
                    suffix: None,
                }
            }
            TokenKind::Literal(litkind, suffix) => {
                self.ts.next();
                Lit {
                    kind: litkind,
                    symbol: self.sess.sm.new_sym_from_span(peek.span),
                    suffix: suffix.map(|x| self.sess.sm.new_sym_from_span(x.span())),
                }
            }
            e => {
                self.sess.new_diag(ExpectedLit {
                    found: e,
                    span: peek.span,
                });
                Lit {
                    kind: jessie_rust_lex::LitKind::Err,
                    symbol: self.sess.sm.new_sym_from_span(peek.span),
                    suffix: None,
                }
            }
        }
    }

    fn parse_lifetime(&mut self) -> Lifetime {
        let peek = self.ts.peek();
        match peek.kind {
            TokenKind::Lifetime => {
                self.ts.next();
                Lifetime {
                    span: peek.span,
                    name: self.sess.sm.span_str(peek.span).to_owned(),
                }
            }
            TokenKind::RawLifetime => {
                self.ts.next();
                Lifetime {
                    span: Span::new(peek.span.lo() + 2, peek.span.hi()),
                    name: self
                        .sess
                        .sm
                        .span_str(Span::new(peek.span.lo() + 2, peek.span.hi()))
                        .to_owned(),
                }
            }
            e => {
                self.sess.new_diag(UnexpectedToken {
                    span: peek.span,
                    expected: TokenKind::Lifetime,
                    found: e,
                });
                self.ts.next();
                Lifetime {
                    span: peek.span,
                    name: self.sess.sm.span_str(peek.span).to_owned(),
                }
            }
        }
    }

    fn parse_lit_expr(&mut self) -> ExprKind {
        ExprKind::Lit(self.parse_lit())
    }

    fn parse_expr(&mut self) -> Expr {
        let id = self.new_node_id();
        let lo = self.ts.peek().span.lo();
        let kind = match self.ts.peek().kind {
            TokenKind::Literal(_, _) => self.parse_lit_expr(),
            e => {
                self.ts.next();
                self.sess.new_diag(ExpectedExpr {
                    found: e,
                    span: self.ts.peek().span,
                });
                ExprKind::Err
            }
        };
        let hi = self.ts.peek().span.lo() - 1;
        let span = Span::new(lo, hi);
        Expr { span, kind, id }
    }
}
