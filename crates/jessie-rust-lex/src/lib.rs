use jessie_session::Session;
use jessie_span::{SourceMap, Span};
mod normalizer;
use normalizer::Normalizer;
use unicode_xid::UnicodeXID;

use crate::TokenKind::DocComment;

pub struct Lexer<'a> {
    iter: Normalizer<'a>,
    sess: &'a mut Session,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str, sp: Span, sess: &'a mut Session) -> Self {
        Self {
            iter: Normalizer::new(src, sp),
            sess,
        }
    }

    pub fn lex_token(&mut self) -> Token {
        if let Some(doc) = self.skip_whitespace() {
            return doc;
        }
        let lo = self.iter.idx();
        let peek = self.iter.peek();
        match peek {
            Some('\'') => self.lex_generic_tick_token(lo),
            Some('\"') => self.lex_generic_quote_token(lo),
            Some('r') => self.lex_generic_r_token(lo),
            Some('b') => self.lex_generic_b_token(lo),
            Some('c') => self.lex_generic_c_token(lo),
            Some(v) if v.is_xid_start() || v == '_' => {
                let kind = self.lex_ident_or_keyword();
                let hi = self.iter.idx();
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
            Some(v) if v.is_ascii_digit() => self.lex_generic_number_token(lo),
            Some('.') => self.lex_dot(lo),
            Some('<') => self.lex_lt(lo),
            Some('>') => self.lex_gt(lo),
            Some('!') => self.lex_bang(lo),
            Some('%') => self.lex_modulo(lo),
            Some('&') => self.lex_ampersand(lo),
            Some('*') => self.lex_star(lo),
            Some('+') => self.lex_plus(lo),
            Some('-') => self.lex_minus(lo),
            Some(':') => self.lex_colon(lo),
            Some('=') => self.lex_eq(lo),
            Some('^') => self.lex_caret(lo),
            Some('|') => self.lex_pipe(lo),
            Some('#') => self.lex_hash(lo),
            Some('$') => self.lex_dollar(lo),
            Some('(') => self.lex_lparen(lo),
            Some(')') => self.lex_rparen(lo),
            Some(',') => self.lex_comma(lo),
            Some(';') => self.lex_semi(lo),
            Some('?') => self.lex_mark(lo),
            Some('@') => self.lex_at(lo),
            Some('[') => self.lex_lsquare(lo),
            Some(']') => self.lex_rsquare(lo),
            Some('{') => self.lex_lcurly(lo),
            Some('}') => self.lex_rcurly(lo),
            Some('~') => self.lex_tilde(lo),
            Some(v) => {
                self.iter.next();
                let span = Span::new(lo, self.iter.idx());
                self.sess.new_err_sp(span, &format!("unexpected token {v}"));
                Token {
                    span,
                    kind: TokenKind::Err,
                }
            }
            None => Token {
                span: Span::new(lo, self.iter.idx()),
                kind: TokenKind::EOF,
            },
        }
    }
    fn lex_tilde(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::Tilde,
        }
    }

    fn lex_rcurly(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::RCurly,
        }
    }

    fn lex_lcurly(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::LCurly,
        }
    }

    fn lex_rsquare(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::RSquare,
        }
    }

    fn lex_lsquare(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::LSquare,
        }
    }

    fn lex_at(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::At,
        }
    }

    fn lex_mark(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::Mark,
        }
    }

    fn lex_semi(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::Semi,
        }
    }

    fn lex_comma(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::Comma,
        }
    }

    fn lex_rparen(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::RParen,
        }
    }

    fn lex_lparen(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::LParen,
        }
    }

    fn lex_dollar(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::Dollar,
        }
    }

    fn lex_hash(&mut self, lo: u32) -> Token {
        self.iter.next();
        let span = Span::new(lo, self.iter.idx());
        Token {
            span,
            kind: TokenKind::Hash,
        }
    }

    fn lex_pipe(&mut self, lo: u32) -> Token {
        self.iter.next();
        let kind = match self.iter.peek() {
            Some('|') => {
                self.iter.next();
                TokenKind::PipePipe
            }
            Some('=') => {
                self.iter.next();
                TokenKind::PipeEq
            }
            _ => TokenKind::Pipe,
        };
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_caret(&mut self, lo: u32) -> Token {
        self.iter.next();
        let kind = match self.iter.peek() {
            Some('=') => {
                self.iter.next();
                TokenKind::CaretEq
            }
            _ => TokenKind::Caret,
        };
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_eq(&mut self, lo: u32) -> Token {
        self.iter.next();
        let kind = match self.iter.peek() {
            Some('=') => {
                self.iter.next();
                TokenKind::EqEq
            }
            Some('>') => {
                self.iter.next();
                TokenKind::FatArrow
            }
            _ => TokenKind::Eq,
        };
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_colon(&mut self, lo: u32) -> Token {
        self.iter.next();
        let kind = match self.iter.peek() {
            Some(':') => {
                self.iter.next();
                TokenKind::ColonColon
            }
            _ => TokenKind::Colon,
        };
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_minus(&mut self, lo: u32) -> Token {
        self.iter.next();
        let kind = match self.iter.peek() {
            Some('=') => {
                self.iter.next();
                TokenKind::MinusEq
            }
            Some('>') => {
                self.iter.next();
                TokenKind::Arr
            }
            _ => TokenKind::Minus,
        };
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_plus(&mut self, lo: u32) -> Token {
        self.iter.next();
        let mut kind = TokenKind::Plus;
        if let Some(peek) = self.iter.peek()
            && peek == '='
        {
            kind = TokenKind::AddEq;
            self.iter.next();
        }
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_star(&mut self, lo: u32) -> Token {
        self.iter.next();
        let mut kind = TokenKind::Star;
        if let Some(peek) = self.iter.peek()
            && peek == '='
        {
            kind = TokenKind::MulEq;
            self.iter.next();
        }
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_ampersand(&mut self, lo: u32) -> Token {
        let amp_count = self.skip_multi('&', 2);
        match (amp_count, self.iter.peek()) {
            (2, _) => {
                let span = Span::new(lo, self.iter.idx());
                Token {
                    span,
                    kind: TokenKind::AndAnd,
                }
            }
            (1, Some('=')) => {
                self.iter.next();
                let span = Span::new(lo, self.iter.idx());
                Token {
                    span,
                    kind: TokenKind::AndEq,
                }
            }
            (1, _) => {
                let span = Span::new(lo, self.iter.idx());
                Token {
                    span,
                    kind: TokenKind::And,
                }
            }
            _ => unreachable!(),
        }
    }

    fn lex_modulo(&mut self, lo: u32) -> Token {
        self.iter.next();
        let mut kind = TokenKind::Mod;
        if let Some(peek) = self.iter.peek()
            && peek == '='
        {
            kind = TokenKind::ModEq;
            self.iter.next();
        }
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn skip_multi(&mut self, ch: char, max: u8) -> u8 {
        let mut out = 0u8;
        for _ in 0..max {
            if let Some(peek) = self.iter.peek()
                && peek == ch
            {
                self.iter.next();
                out += 1;
            }
        }
        out
    }

    fn lex_bang(&mut self, lo: u32) -> Token {
        self.iter.next();
        let mut kind = TokenKind::Bang;
        if let Some(peek) = self.iter.peek()
            && peek == '='
        {
            kind = TokenKind::Ne;
            self.iter.next();
        }
        let span = Span::new(lo, self.iter.idx());
        Token { span, kind }
    }

    fn lex_gt(&mut self, lo: u32) -> Token {
        let gt_count = self.skip_multi('>', 2);
        let span = Span::new(lo, self.iter.idx());
        match (gt_count, self.iter.peek()) {
            (2, Some('=')) => {
                self.iter.next();
                Token {
                    span,
                    kind: TokenKind::GtGtEq,
                }
            }
            (2, _) => Token {
                span,
                kind: TokenKind::GtGt,
            },
            (1, Some('=')) => {
                self.iter.next();
                Token {
                    span,
                    kind: TokenKind::GtEq,
                }
            }
            (1, _) => Token {
                span,
                kind: TokenKind::Gt,
            },
            _ => unreachable!(),
        }
    }

    fn lex_lt(&mut self, lo: u32) -> Token {
        let lt_count = self.skip_multi('<', 2);
        let span = Span::new(lo, self.iter.idx());
        match (lt_count, self.iter.peek()) {
            (2, Some('=')) => {
                self.iter.next();
                Token {
                    span,
                    kind: TokenKind::LtLtEq,
                }
            }
            (2, _) => Token {
                span,
                kind: TokenKind::LtLt,
            },
            (1, Some('-')) => {
                self.iter.next();
                Token {
                    span,
                    kind: TokenKind::Larr,
                }
            }
            (1, Some('=')) => {
                self.iter.next();
                Token {
                    span,
                    kind: TokenKind::LtEq,
                }
            }
            (1, _) => Token {
                span,
                kind: TokenKind::Lt,
            },
            _ => unreachable!(),
        }
    }

    fn lex_dot(&mut self, lo: u32) -> Token {
        let dot_count = self.skip_multi('.', 3);
        let span = Span::new(lo, self.iter.idx());
        match (dot_count, self.iter.peek()) {
            (3, _) => Token {
                span,
                kind: TokenKind::DotDotDot,
            },
            (2, Some('=')) => {
                self.iter.next();
                Token {
                    span,
                    kind: TokenKind::DotDotEq,
                }
            }
            (2, _) => Token {
                span,
                kind: TokenKind::DotDot,
            },
            (1, _) => Token {
                span,
                kind: TokenKind::Dot,
            },
            _ => unreachable!(),
        }
    }

    fn lex_generic_number_token(&mut self, lo: u32) -> Token {
        let kind = self.lex_generic_digit_token();
        if let TokenKind::Literal(LitKind::Integer(IntegerKind::Dec(has_underscore)), _) = kind
            && let Some('.') = self.iter.peek()
        {
            self.iter.next();
            match self.iter.peek() {
                Some(v) if v == '.' || v == '_' || v.is_xid_start() => {
                    let hi = self.iter.idx();
                    let float_kind = match has_underscore {
                        HasUnderscore::Yes => FloatKind::NotReserved,
                        HasUnderscore::No => FloatKind::Reserved,
                    };
                    return Token {
                        span: Span::new(lo, hi),
                        kind: TokenKind::Literal(LitKind::Float(float_kind), self.skip_suffix()),
                    };
                }
                Some(v) if v.is_ascii_digit() => {
                    let has_underscore_rhs = self.skip_dec_digit();
                    let suffix = self.skip_suffix();
                    let float_kind = match (suffix.is_some(), has_underscore, has_underscore_rhs) {
                        (false, HasUnderscore::No, HasUnderscore::No) => FloatKind::Reserved,
                        _ => FloatKind::NotReserved,
                    };
                    let hi = self.iter.idx();
                    return Token {
                        span: Span::new(lo, hi),
                        kind: TokenKind::Literal(LitKind::Float(float_kind), self.skip_suffix()),
                    };
                }
                _ => {
                    let float_kind = match kind.has_suffix() {
                        true => FloatKind::NotReserved,
                        false => FloatKind::Reserved,
                    };
                    let hi = self.iter.idx();
                    return Token {
                        span: Span::new(lo, hi),
                        kind: TokenKind::Literal(LitKind::Float(float_kind), None),
                    };
                }
            }
        }
        let hi = self.iter.idx();
        Token {
            span: Span::new(lo, hi),
            kind,
        }
    }

    fn lex_generic_digit_token(&mut self) -> TokenKind {
        if let Some(peek) = self.iter.peek()
            && peek == '0'
        {
            self.iter.next();
            if let Some(peek) = self.iter.peek() {
                match peek {
                    'b' => {
                        self.iter.next();
                        self.skip_bin_digit();
                        return TokenKind::Literal(
                            LitKind::Integer(IntegerKind::Oct),
                            self.skip_suffix(),
                        );
                    }
                    'o' => {
                        self.iter.next();
                        self.skip_oct_digit();
                        return TokenKind::Literal(
                            LitKind::Integer(IntegerKind::Bin),
                            self.skip_suffix(),
                        );
                    }
                    'x' => {
                        self.iter.next();
                        self.skip_hex_digit();
                        return TokenKind::Literal(
                            LitKind::Integer(IntegerKind::Hex),
                            self.skip_suffix(),
                        );
                    }
                    _ => {}
                }
            }
        }
        let has_underscore = self.skip_dec_digit();
        TokenKind::Literal(
            LitKind::Integer(IntegerKind::Dec(has_underscore)),
            self.skip_suffix(),
        )
    }

    fn skip_dec_digit(&mut self) -> HasUnderscore {
        let lo = self.iter.idx();
        let next = self.iter.next();
        let hi = self.iter.idx();
        match next {
            Some(v) if !v.is_ascii_digit() => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    &format!("expected an ASCII value between 0 through 9, found {v}"),
                );
            }
            None => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    "expected an ASCII value between 0 through 9 , found end of file",
                );
            }
            _ => {}
        }
        let mut has_underscore = HasUnderscore::No;
        while let Some(peek) = self.iter.peek()
            && (peek.is_ascii_digit() || peek == '_')
        {
            if peek == '_' {
                has_underscore = HasUnderscore::Yes;
            }
            self.iter.next();
        }
        has_underscore
    }

    fn skip_bin_digit(&mut self) -> HasUnderscore {
        let lo = self.iter.idx();
        let next = self.iter.next();
        let hi = self.iter.idx();
        match next {
            Some(v) if !('0'..='1').contains(&v) => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    &format!("expected an ASCII value between 0 through 1, found {v}"),
                );
            }
            None => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    "expected an ASCII value between 0 through 1 , found end of file",
                );
            }
            _ => {}
        }
        let mut has_underscore = HasUnderscore::No;
        while let Some(peek) = self.iter.peek()
            && (('0'..='1').contains(&peek) || peek == '_')
        {
            if peek == '_' {
                has_underscore = HasUnderscore::Yes;
            }
            self.iter.next();
        }
        has_underscore
    }

    fn skip_oct_digit(&mut self) -> HasUnderscore {
        let lo = self.iter.idx();
        let next = self.iter.next();
        let hi = self.iter.idx();
        match next {
            Some(v) if !('0'..='7').contains(&v) => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    &format!("expected an ASCII value between 0 through 7, found {v}"),
                );
            }
            None => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    "expected an ASCII value between 0 through 7 , found end of file",
                );
            }
            _ => {}
        }
        let mut has_underscore = HasUnderscore::No;
        while let Some(peek) = self.iter.peek()
            && (('0'..='7').contains(&peek) || peek == '_')
        {
            if peek == '_' {
                has_underscore = HasUnderscore::Yes;
            }
            self.iter.next();
        }
        has_underscore
    }

    fn skip_hex_digit(&mut self) -> HasUnderscore {
        let lo = self.iter.idx();
        let next = self.iter.next();
        let hi = self.iter.idx();
        match next {
            Some(v) if !v.is_ascii_hexdigit() => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    &format!("expected an ASCII hexadecimal value, found {v}"),
                );
            }
            None => {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    "expected an ASCII hexadecimal value, found end of file",
                );
            }
            _ => {}
        }
        let mut has_underscore = HasUnderscore::No;
        while let Some(peek) = self.iter.peek()
            && (peek.is_ascii_hexdigit() || peek == '_')
        {
            if peek == '_' {
                has_underscore = HasUnderscore::Yes;
            }
            self.iter.next();
        }
        has_underscore
    }

    fn lex_generic_tick_token(&mut self, lo: u32) -> Token {
        let kind = self.lex_tick(true, true, true, true);
        let hi = self.iter.idx();
        Token {
            span: Span::new(lo, hi),
            kind,
        }
    }

    fn lex_generic_quote_token(&mut self, lo: u32) -> Token {
        let kind = self.lex_string_literal(true, true, true, true);
        let hi = self.iter.idx();
        Token {
            span: Span::new(lo, hi),
            kind,
        }
    }

    fn lex_generic_r_token(&mut self, lo: u32) -> Token {
        self.iter.next();
        let hash_count = self.collect_hash_count();
        if self.iter.peek() == Some('"') {
            let kind = self.lex_raw_string(hash_count);
            let hi = self.iter.idx();
            return Token {
                span: Span::new(lo, hi),
                kind,
            };
        }
        if hash_count == 1
            && let Some(peek) = self.iter.peek()
            && (peek.is_xid_start() || peek == '_')
        {
            self.lex_ident_or_keyword();
            let hi = self.iter.idx();
            return Token {
                span: Span::new(lo, hi),
                kind: TokenKind::RawIdent,
            };
        }
        if let Some(peek) = self.iter.peek()
            && !(peek.is_xid_start() || peek == '_')
        {
            let hi = self.iter.idx();
            self.sess
                .new_err_sp(Span::new(lo, hi), "raw literal must have a right hand side");
            return Token {
                span: Span::new(lo, hi),
                kind: TokenKind::Err,
            };
        }
        let kind = self.lex_xid_continue();
        let hi = self.iter.idx();
        Token {
            span: Span::new(lo, hi),
            kind,
        }
    }

    fn lex_generic_b_token(&mut self, lo: u32) -> Token {
        self.iter.next();
        match self.iter.peek() {
            Some('\'') => {
                let mut kind = self.lex_tick(true, false, false, true);
                let hi = self.iter.idx();
                if matches!(kind, TokenKind::Literal(LitKind::Char, _)) {
                    kind = TokenKind::Literal(LitKind::Byte, kind.get_suffix());
                }
                if matches!(kind, TokenKind::Lifetime) {
                    self.sess
                        .new_err_sp(Span::new(lo, hi), "missing \' after byte sequence");
                }
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
            Some('\"') => {
                let mut kind = self.lex_string_literal(true, false, false, true);
                let hi = self.iter.idx();
                if matches!(kind, TokenKind::Literal(LitKind::String, _)) {
                    kind = TokenKind::Literal(LitKind::ByteString, kind.get_suffix());
                }
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
            Some('r') => {
                self.iter.next();
                let hash_count = self.collect_hash_count();
                if hash_count > 0 || self.iter.peek() == Some('\"') {
                    let mut kind = self.lex_raw_string(hash_count);
                    let hi = self.iter.idx();
                    if let TokenKind::Literal(LitKind::RawString(c), _) = kind
                        && c == hash_count
                    {
                        kind = TokenKind::Literal(
                            LitKind::RawByteString(hash_count),
                            kind.get_suffix(),
                        );
                    }
                    return Token {
                        span: Span::new(lo, hi),
                        kind,
                    };
                }
                let kind = self.lex_xid_continue();
                let hi = self.iter.idx();
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
            _ => {
                let kind = self.lex_xid_continue();
                let hi = self.iter.idx();
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
        }
    }

    fn lex_generic_c_token(&mut self, lo: u32) -> Token {
        self.iter.next();
        match self.iter.peek() {
            Some('\"') => {
                let mut kind = self.lex_string_literal(true, false, false, true);
                let hi = self.iter.idx();
                if matches!(kind, TokenKind::Literal(LitKind::String, _)) {
                    kind = TokenKind::Literal(LitKind::CString, kind.get_suffix());
                }
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
            Some('r') => {
                self.iter.next();
                let hash_count = self.collect_hash_count();
                if hash_count > 0 || self.iter.peek() == Some('\"') {
                    let mut kind = self.lex_raw_string(hash_count);
                    let hi = self.iter.idx();
                    if let TokenKind::Literal(LitKind::RawString(c), _) = kind
                        && c == hash_count
                    {
                        kind =
                            TokenKind::Literal(LitKind::RawCString(hash_count), kind.get_suffix());
                    }
                    return Token {
                        span: Span::new(lo, hi),
                        kind,
                    };
                }
                let kind = self.lex_xid_continue();
                let hi = self.iter.idx();
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
            _ => {
                let kind = self.lex_xid_continue();
                let hi = self.iter.idx();
                Token {
                    span: Span::new(lo, hi),
                    kind,
                }
            }
        }
    }

    fn skip_whitespace(&mut self) -> Option<Token> {
        while let Some(peek) = self.iter.peek()
            && (peek.is_whitespace() || peek == '/')
        {
            let lo = self.iter.idx();
            if peek == '/'
                && let Some(kind) = self.lex_comments()
            {
                return Some(Token {
                    span: Span::new(lo, self.iter.idx()),
                    kind,
                });
            }
            self.iter.next();
        }
        None
    }

    fn lex_ident_or_keyword(&mut self) -> TokenKind {
        if let Some(peek) = self.iter.peek()
            && (peek.is_xid_start() || peek == '_')
        {
            self.iter.next();
        }
        while let Some(peek) = self.iter.peek()
            && peek.is_xid_continue()
        {
            self.iter.next();
        }
        TokenKind::Ident
    }

    fn lex_xid_continue(&mut self) -> TokenKind {
        while let Some(peek) = self.iter.peek()
            && peek.is_xid_continue()
        {
            self.iter.next();
        }
        TokenKind::Ident
    }

    fn skip_suffix(&mut self) -> Option<Suffix> {
        let lo = self.iter.idx();
        let has_suffix = if let Some(peek) = self.iter.peek()
            && (peek.is_xid_start() || peek == '_')
        {
            true
        } else {
            false
        };
        self.lex_ident_or_keyword();
        let hi = self.iter.idx();
        match has_suffix {
            true => Some(Suffix::new(Span::new(lo, hi))),
            false => None,
        }
    }

    fn skip_until_newline(&mut self) {
        while let Some(peek) = self.iter.peek()
            && peek != '\n'
        {
            self.iter.next();
        }
    }

    fn skip_until_block_comment_end(&mut self) -> Option<TokenKind> {
        loop {
            let next = self.iter.next();
            let peek = self.iter.peek();

            if next.is_none() {
                return Some(TokenKind::MalformedComment);
            }

            if matches!((next, peek), (Some('*'), Some('/'))) {
                self.iter.next();
                return None;
            }
        }
    }

    fn skip_ch(&mut self, ch: char) {
        let next = self.iter.next();
        if Some(ch) != next {
            self.sess.new_err_sp(
                Span::new(self.iter.idx(), self.iter.idx()),
                &format!(
                    "expected {ch}, found {}",
                    match next {
                        Some(v) => format!("{v}"),
                        _ => "end of file".into(),
                    }
                ),
            );
        }
    }

    fn lex_tick(
        &mut self,
        allow_quote_escaping: bool,
        allow_ascii_escaping: bool,
        allow_unicode_escaping: bool,
        allow_byte_escaping: bool,
    ) -> TokenKind {
        self.iter.next();
        match self.iter.peek() {
            Some('\\') => {
                self.iter.next();
                match self.iter.peek() {
                    Some('\'') | Some('\"') if allow_quote_escaping => {
                        self.iter.next();
                        self.skip_ch('\'');
                        TokenKind::Literal(LitKind::Char, self.skip_suffix())
                    }
                    Some('n') | Some('r') | Some('t') | Some('\\') | Some('0')
                        if allow_ascii_escaping || allow_byte_escaping =>
                    {
                        self.iter.next();
                        self.skip_ch('\'');
                        TokenKind::Literal(LitKind::Char, self.skip_suffix())
                    }
                    Some('x') if allow_ascii_escaping || allow_byte_escaping => {
                        self.skip_ascii_escape(allow_ascii_escaping);
                        self.skip_ch('\'');
                        TokenKind::Literal(LitKind::Char, self.skip_suffix())
                    }
                    Some('u') if allow_unicode_escaping => {
                        let lo = self.iter.idx() - 1;
                        self.iter.next();
                        self.skip_ch('{');
                        for _ in 0..6 {
                            let first = self.iter.next();
                            let second = self.iter.peek();
                            if second == Some('}') {
                                break;
                            }
                            let hi = self.iter.idx();
                            if let Some(v) = first
                                && v.is_ascii_hexdigit()
                            {
                                continue;
                            } else {
                                self.sess.new_err_sp(
                                    Span::new(lo, hi),
                                    &format!(
                                        "expected a hexadecimal digit, found {}",
                                        match first {
                                            Some(v) => format!("{v}"),
                                            _ => "end of file".to_string(),
                                        }
                                    ),
                                );
                                return TokenKind::Err;
                            }
                        }
                        self.skip_ch('}');
                        self.skip_ch('\'');
                        TokenKind::Literal(LitKind::Char, self.skip_suffix())
                    }
                    err => {
                        let idx = self.iter.idx();
                        self.sess.new_err_sp(
                            Span::new(idx, idx),
                            &format!(
                                "invalid escape \"{}\"",
                                match err {
                                    Some(v) => format!("{v}"),
                                    _ => "end of file".to_string(),
                                }
                            ),
                        );
                        TokenKind::Err
                    }
                }
            }
            Some(v) if !v.is_xid_start() => {
                self.iter.next();
                self.skip_ch('\'');
                TokenKind::Literal(LitKind::Char, self.skip_suffix())
            }
            Some(v) if v.is_xid_start() => {
                self.iter.next();
                match (v, self.iter.peek()) {
                    ('r', Some('#')) => {
                        self.iter.next();
                        self.skip_suffix();
                        TokenKind::RawLifetime
                    }
                    (_, Some('\'')) => {
                        self.iter.next();
                        TokenKind::Literal(LitKind::Char, self.skip_suffix())
                    }
                    _ => TokenKind::Lifetime,
                }
            }
            None => {
                let idx = self.iter.idx();
                self.sess.new_err_sp(
                    Span::new(idx, idx),
                    "expected a character or lifetime sequence, found end of file",
                );
                TokenKind::Err
            }
            _ => unreachable!(),
        }
    }

    fn collect_hash_count(&mut self) -> u8 {
        let mut hash_count = 0u8;
        while self.iter.peek() == Some('#') {
            self.iter.next();
            hash_count = match hash_count.checked_add(1) {
                Some(v) => v,
                None => {
                    self.sess.new_err_sp(
                        Span::new(self.iter.idx(), self.iter.idx()),
                        "more than 255 hash symbols found",
                    );
                    u8::MAX
                }
            };
        }
        hash_count
    }

    fn lex_raw_string(&mut self, hash_count: u8) -> TokenKind {
        self.skip_ch('\"');
        loop {
            let peek = self.iter.peek();
            match peek {
                Some('\"') => {
                    self.iter.next();
                    let mut hash_count_rhs = 0u8;
                    if hash_count_rhs == hash_count {
                        return TokenKind::Literal(
                            LitKind::RawString(hash_count),
                            self.skip_suffix(),
                        );
                    }
                    while let Some(peek) = self.iter.peek()
                        && peek == '#'
                    {
                        self.iter.next();
                        hash_count_rhs += 1;
                        if hash_count_rhs == hash_count {
                            return TokenKind::Literal(
                                LitKind::RawString(hash_count),
                                self.skip_suffix(),
                            );
                        }
                    }
                    continue;
                }
                Some(_) => {
                    self.iter.next();
                }
                None => {
                    let idx = self.iter.idx();
                    self.sess.new_err_sp(
                        Span::new(idx, idx),
                        &format!(
                            "expected \" followed by {hash_count} # symbols, found end of file"
                        ),
                    );
                    return TokenKind::Literal(LitKind::MalformedString, self.skip_suffix());
                }
            }
        }
    }

    fn skip_ascii_escape(&mut self, is_ascii: bool) {
        self.iter.next();
        let lo = self.iter.idx();
        let first = self.iter.next();
        let second = self.iter.next();
        if let Some(first) = first
            && let Some(second) = second
            && first.is_ascii_hexdigit()
            && second.is_ascii_hexdigit()
        {
            if is_ascii && !('0'..='7').contains(&first) {
                let hi = self.iter.idx();
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    &format!(
                        "expected an octal digit followed by a hexadecimal digit, found {first} and {second}",
                    ),
                );
            }
        } else {
            let hi = self.iter.idx();
            self.sess.new_err_sp(
                Span::new(lo, hi),
                &format!(
                    "expected two hexadecimal digits, found {} and {}",
                    match first {
                        Some(v) => format!("{v}"),
                        _ => "end of file".into(),
                    },
                    match second {
                        Some(v) => format!("{v}"),
                        _ => "end of file".into(),
                    }
                ),
            );
        }
    }

    fn skip_unicode_escape(&mut self) {
        self.iter.next();
        let lo = self.iter.idx();
        self.skip_ch('{');
        for _ in 0..6 {
            let first = self.iter.next();
            let second = self.iter.peek();
            if second == Some('}') {
                break;
            }
            let hi = self.iter.idx();
            if let Some(v) = first
                && v.is_ascii_hexdigit()
            {
                continue;
            } else {
                self.sess.new_err_sp(
                    Span::new(lo, hi),
                    &format!(
                        "expected a hexadecimal digit, found {}",
                        match first {
                            Some(v) => format!("{v}"),
                            _ => "end of file".into(),
                        }
                    ),
                );
                return;
            }
        }
        self.skip_ch('}');
    }

    fn lex_string_literal(
        &mut self,
        allow_quote_escaping: bool,
        allow_ascii_escaping: bool,
        allow_unicode_escaping: bool,
        allow_byte_escaping: bool,
    ) -> TokenKind {
        self.iter.next();
        loop {
            let peek = self.iter.peek();

            match peek {
                Some('\\') => {
                    self.iter.next();
                    match self.iter.peek() {
                        Some('\'') | Some('\"') if allow_quote_escaping => {
                            self.iter.next();
                            continue;
                        }
                        Some('n') | Some('r') | Some('t') | Some('\\') | Some('0')
                            if allow_ascii_escaping || allow_byte_escaping =>
                        {
                            self.iter.next();
                            continue;
                        }
                        Some('x') if allow_ascii_escaping || allow_byte_escaping => {
                            self.skip_ascii_escape(allow_ascii_escaping);
                            continue;
                        }
                        Some('u') if allow_unicode_escaping => {
                            self.skip_unicode_escape();
                            continue;
                        }
                        Some('\u{000a}') => {
                            self.iter.next();
                            continue;
                        }
                        _ => {
                            let idx = self.iter.idx();
                            self.sess
                                .new_err_sp(Span::new(idx, idx), "invalid character escape");
                        }
                    }
                }
                Some('\"') => {
                    break;
                }
                Some(_) => {
                    self.iter.next();
                    continue;
                }
                None => {
                    let idx = self.iter.idx();
                    self.sess
                        .new_err_sp(Span::new(idx, idx), "expected \", found end of file");
                    return TokenKind::Literal(LitKind::MalformedString, self.skip_suffix());
                }
            }
        }
        self.skip_ch('\"');
        TokenKind::Literal(LitKind::String, self.skip_suffix())
    }

    fn lex_comments(&mut self) -> Option<TokenKind> {
        if self.iter.peek() == Some('/') {
            self.iter.next();
            if self.iter.peek() == Some('/') {
                self.iter.next();
                let peek = self.iter.peek();
                self.skip_until_newline();
                match peek {
                    Some('/') => {
                        return Some(DocComment(CommentKind::Line, AttrStyle::Outer));
                    }
                    Some('!') => {
                        return Some(DocComment(CommentKind::Line, AttrStyle::Inner));
                    }
                    _ => return None,
                }
            }
            if self.iter.peek() == Some('*') {
                self.iter.next();
                let peek = self.iter.peek();
                self.skip_until_block_comment_end();
                match peek {
                    Some('*') => {
                        return Some(DocComment(CommentKind::Block, AttrStyle::Outer));
                    }
                    Some('!') => {
                        return Some(DocComment(CommentKind::Block, AttrStyle::Inner));
                    }
                    _ => return None,
                }
            }
            if self.iter.peek() == Some('=') {
                self.iter.next();
                return Some(TokenKind::DivEq);
            }
            return Some(TokenKind::Slash);
        }
        None
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    #[inline]
    pub const fn is_ident(self) -> bool {
        matches!(self.kind, TokenKind::Ident)
    }

    #[inline]
    pub fn is_strict_keyword(self, sm: &SourceMap) -> bool {
        matches!(
            sm.span_str(self.span),
            "_" | "as"
                | "async"
                | "await"
                | "break"
                | "const"
                | "continue"
                | "crate"
                | "dyn"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
        )
    }

    #[inline]
    pub fn is_reserved_keyword(self, sm: &SourceMap) -> bool {
        matches!(
            sm.span_str(self.span),
            "abstract"
                | "become"
                | "box"
                | "do"
                | "final"
                | "gen"
                | "macro"
                | "override"
                | "priv"
                | "try"
                | "typeof"
                | "unsized"
                | "virtual"
                | "yield"
        )
    }

    #[inline]
    pub fn is_weak_keyword(self, sm: &SourceMap) -> bool {
        matches!(
            sm.span_str(self.span),
            "'static" | "macro_rules" | "raw" | "safe" | "union"
        )
    }

    #[inline]
    pub fn is_keyword(self, sm: &SourceMap) -> bool {
        self.is_strict_keyword(sm) || self.is_reserved_keyword(sm) || self.is_weak_keyword(sm)
    }

    #[inline]
    pub fn has_suffix(self) -> bool {
        self.kind.has_suffix()
    }

    #[inline]
    pub fn get_suffix(self) -> Option<Suffix> {
        self.kind.get_suffix()
    }

    #[inline]
    pub fn is_eof(self) -> bool {
        self.kind.is_eof()
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum TokenKind {
    /// ...
    DotDotDot,
    /// ..=
    DotDotEq,
    /// ..
    DotDot,
    /// .
    Dot,
    /// <<=
    LtLtEq,
    /// <<
    LtLt,
    /// <=
    LtEq,
    /// <-
    Larr,
    /// <
    Lt,
    /// >>=
    GtGtEq,
    /// >>
    GtGt,
    /// >=
    GtEq,
    /// >
    Gt,
    /// !=
    Ne,
    /// !
    Bang,
    /// ident
    Ident,
    /// %=
    ModEq,
    /// %
    Mod,
    /// &&
    AndAnd,
    /// &=
    AndEq,
    /// &
    And,
    /// *
    Star,
    /// *=
    MulEq,
    /// +
    Plus,
    /// +=
    AddEq,
    /// -
    Minus,
    /// -=
    MinusEq,
    /// ->
    Arr,
    /// r#ident
    RawIdent,
    /// 'r#lifetimename
    RawLifetime,
    /// 'lifetime
    Lifetime,
    /// /
    Slash,
    /// /=
    DivEq,
    /// :
    Colon,
    /// ::
    ColonColon,
    /// ==
    EqEq,
    /// =>
    FatArrow,
    /// =
    Eq,
    /// ^
    Caret,
    /// ^=
    CaretEq,
    /// |
    Pipe,
    /// ||
    PipePipe,
    /// |=
    PipeEq,
    /// #
    Hash,
    /// $
    Dollar,
    /// (
    LParen,
    /// )
    RParen,
    /// ,
    Comma,
    /// ;
    Semi,
    /// ?
    Mark,
    /// @
    At,
    /// [
    LSquare,
    /// ]
    RSquare,
    /// {
    LCurly,
    /// }
    RCurly,
    /// ~
    Tilde,
    /// /// Doc comment
    DocComment(CommentKind, AttrStyle),
    /// (malformed comment)
    MalformedComment,
    /// literal ('a',"abc",r#"abc"#,b'b',b"abc",br#"abc"#,c"abc",cr#"abc"#,1.0f32,0u8,"malformed string)
    Literal(LitKind, Option<Suffix>),
    /// unknown
    Err,
    /// end of file (EOF)
    EOF,
}

impl TokenKind {
    #[inline]
    pub fn has_suffix(self) -> bool {
        matches!(self, TokenKind::Literal(_, Some(_)))
    }

    #[inline]
    pub fn get_suffix(self) -> Option<Suffix> {
        match self {
            TokenKind::Literal(_, suffix) => suffix,
            _ => None,
        }
    }

    #[inline]
    pub fn is_eof(self) -> bool {
        matches!(self, TokenKind::EOF)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LitKind {
    /// 'a'
    Char,
    /// "abc"
    String,
    /// r#"abc"#
    RawString(u8),
    /// b'b'
    Byte,
    /// b"abc"
    ByteString,
    /// br#"abc"#
    RawByteString(u8),
    /// c"abc"
    CString,
    /// cr#"abc"#
    RawCString(u8),
    /// 1.0f32
    Float(FloatKind),
    /// 0u8
    Integer(IntegerKind),
    /// "malformed string
    MalformedString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatKind {
    Reserved,
    NotReserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegerKind {
    Dec(HasUnderscore),
    Bin,
    Oct,
    Hex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HasUnderscore {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct Suffix(Span);

impl Suffix {
    pub(crate) fn new(sp: Span) -> Self {
        Self(sp)
    }
    pub fn span(self) -> Span {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentKind {
    Line,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttrStyle {
    Inner,
    Outer,
}
