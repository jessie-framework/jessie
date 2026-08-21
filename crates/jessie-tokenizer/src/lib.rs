use std::{iter::Peekable, str::Chars};

use jessie_span::Span;

pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
    idx: u32,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str, sp: Span) -> Self {
        Self {
            chars: src.chars().peekable(),
            idx: sp.lo(),
        }
    }

    fn peek_ch(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn next_ch(&mut self) -> Option<char> {
        let result = self.chars.next()?;
        self.idx += result.len_utf8() as u32;
        Some(result)
    }

    pub fn next_tok(&mut self) -> Token {
        let lo = self.idx;
        let kind = self.parse_kind();
        let hi = self.idx;
        let span = if kind == TokenKind::EOF {
            Span::nul()
        } else {
            Span::new(lo, hi)
        };
        Token { span, kind }
    }

    fn parse_kind(&mut self) -> TokenKind {
        match self.peek_ch() {
            Some(ch) if let Ok(out) = ch.try_into() => {
                self.next_ch();
                out
            }
            Some(ch) => match ch {
                ch if ch.is_ascii_alphabetic() || ch == '_' => self.parse_ident(),
                ch if ch.is_ascii_digit() => self.parse_digit(),
                ch if ch.is_whitespace() => self.parse_whitespace(),
                '#' => self.parse_comment(),
                '"' => self.parse_string(),
                _ => {
                    self.next_ch();
                    TokenKind::Unknown
                }
            },
            None => TokenKind::EOF,
        }
    }

    fn parse_comment(&mut self) -> TokenKind {
        self.next_ch();
        if self.peek_ch() == Some('|') {
            self.next_ch();
            loop {
                while let Some(ch) = self.peek_ch()
                    && ch != '|'
                {
                    self.next_ch();
                }
                self.next_ch();
                if self.peek_ch() == Some('#') {
                    self.next_ch();
                    break;
                }
                if self.peek_ch().is_none() {
                    return TokenKind::MalformedComment;
                }
                continue;
            }
        } else {
            while let Some(ch) = self.peek_ch()
                && ch != '\n'
            {
                self.next_ch();
            }
        }
        TokenKind::Comment
    }

    fn parse_string(&mut self) -> TokenKind {
        self.next_ch();
        while let Some(ch) = self.peek_ch()
            && ch != '"'
        {
            self.next_ch();
        }
        if self.next_ch().is_none() {
            TokenKind::MalformedString
        } else {
            TokenKind::String
        }
    }

    fn parse_whitespace(&mut self) -> TokenKind {
        while let Some(ch) = self.peek_ch()
            && ch.is_whitespace()
        {
            self.next_ch();
        }
        TokenKind::Whitespace
    }

    fn parse_digit(&mut self) -> TokenKind {
        while let Some(ch) = self.peek_ch()
            && ch.is_ascii_digit()
        {
            self.next_ch();
        }
        TokenKind::Digit
    }

    fn parse_ident(&mut self) -> TokenKind {
        while let Some(ch) = self.peek_ch()
            && (ch.is_ascii_alphanumeric() || ch == '_')
        {
            self.next_ch();
        }
        TokenKind::Ident
    }

    pub fn collect(&mut self) -> Vec<Token> {
        let mut out = vec![];
        loop {
            let next = self.next_tok();
            if next.kind == TokenKind::EOF {
                break;
            } else {
                out.push(next)
            }
        }
        out
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenKind {
    EOF,
    Unknown,
    Whitespace,
    Lt,
    Gt,
    Hash,
    Slash,
    Backslash,
    Bang,
    Tilde,
    Plus,
    Minus,
    Star,
    Percent,
    Caret,
    And,
    Or,
    At,
    Dot,
    Ident,
    LCurly,
    RCurly,
    LParen,
    RParen,
    LSquare,
    RSquare,
    Eq,
    Digit,
    String,
    MalformedString,
    Comment,
    MalformedComment,
    Comma,
    Colon,
    Semicolon,
}

/// Only implemented for values that take up a single Unicode code point and do not cause ambiguity
impl TryFrom<char> for TokenKind {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '@' => Ok(Self::At),
            '<' => Ok(Self::Lt),
            '>' => Ok(Self::Gt),
            '/' => Ok(Self::Slash),
            '\\' => Ok(Self::Backslash),
            '!' => Ok(Self::Bang),
            '~' => Ok(Self::Tilde),
            '+' => Ok(Self::Plus),
            '-' => Ok(Self::Minus),
            '*' => Ok(Self::Star),
            '%' => Ok(Self::Percent),
            '^' => Ok(Self::Caret),
            '&' => Ok(Self::And),
            '|' => Ok(Self::Or),
            '.' => Ok(Self::Dot),
            '{' => Ok(Self::LCurly),
            '}' => Ok(Self::RCurly),
            '(' => Ok(Self::LParen),
            ')' => Ok(Self::RParen),
            '[' => Ok(Self::LSquare),
            ']' => Ok(Self::RSquare),
            '=' => Ok(Self::Eq),
            ',' => Ok(Self::Comma),
            ':' => Ok(Self::Colon),
            ';' => Ok(Self::Semicolon),
            _ => Err(()),
        }
    }
}
