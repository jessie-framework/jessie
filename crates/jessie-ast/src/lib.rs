use std::cell::Cell;

use jessie_rust_lex::{Lexer, Token};

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

    pub fn next(&self) -> Token {
        let out = self.peek();
        self.idx.update(|x| x + 1);
        out
    }
}

pub mod parser;
