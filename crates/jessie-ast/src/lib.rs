use std::cell::Cell;

use jessie_rust_lex::{Lexer, Token};

pub mod diagnostics {
    use jessie_rust_lex::TokenKind;
    use jessie_session::Diagnostic;
    use jessie_span::Span;

    pub struct ExpectedBound {
        pub found: TokenKind,
        pub span: Span,
    }

    impl Diagnostic for ExpectedBound {
        fn diag(self, sess: &mut jessie_session::Session) {
            sess.new_err_sp(
                self.span,
                &format!("expected diagnostic, found {}", self.found),
            );
        }
    }

    pub struct ExpectedExpr {
        pub found: TokenKind,
        pub span: Span,
    }

    impl Diagnostic for ExpectedExpr {
        fn diag(self, sess: &mut jessie_session::Session) {
            sess.new_err_sp(
                self.span,
                &format!("expected expression, found {}", self.found),
            );
        }
    }

    pub struct ExpectedItem {
        pub found: TokenKind,
        pub span: Span,
    }

    impl Diagnostic for ExpectedItem {
        fn diag(self, sess: &mut jessie_session::Session) {
            sess.new_err_sp(self.span, &format!("expected item, found {}", self.found));
        }
    }

    pub struct ExpectedType {
        pub found: TokenKind,
        pub span: Span,
    }

    impl Diagnostic for ExpectedType {
        fn diag(self, sess: &mut jessie_session::Session) {
            sess.new_err_sp(self.span, &format!("expected type, found {}", self.found));
        }
    }

    pub struct ExpectedPattern {
        pub found: TokenKind,
        pub span: Span,
    }

    impl Diagnostic for ExpectedPattern {
        fn diag(self, sess: &mut jessie_session::Session) {
            sess.new_err_sp(
                self.span,
                &format!("expected pattern, found {}", self.found),
            );
        }
    }
    pub struct ExpectedStmt {
        pub found: TokenKind,
        pub span: Span,
    }

    impl Diagnostic for ExpectedStmt {
        fn diag(self, sess: &mut jessie_session::Session) {
            sess.new_err_sp(
                self.span,
                &format!("expected statement, found {}", self.found),
            );
        }
    }
}

pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub idx: Cell<usize>,
}

impl TokenStream {
    pub fn new(mut lexer: Lexer<'_>) -> Self {
        let mut tokens = vec![];
        loop {
            let tok = lexer.lex_token();
            tokens.push(tok);
            if tok.is_eof() {
                break;
            }
        }
        let idx = Cell::new(0);
        Self { tokens, idx }
    }

    pub fn peek(&self) -> Token {
        self.tokens
            .get(self.idx.get())
            .copied()
            .unwrap_or(self.tokens.last().copied().unwrap())
    }

    pub fn peek_n<const N: usize>(&self) -> [Token; N] {
        let mut idx = 0;
        [(); N].map(|_| {
            let out = self
                .tokens
                .get(self.idx.get() + idx)
                .copied()
                .unwrap_or(self.tokens.last().copied().unwrap());
            idx += 1;
            out
        })
    }

    pub fn next(&self) -> Token {
        let out = self.peek();
        self.idx.update(|x| x + 1);
        out
    }
}

pub mod ast;
pub mod parser;
