use std::{
    array::IntoIter,
    iter::{Chain, Peekable},
    str::Chars,
};

use jessie_span::Span;

/// https://doc.rust-lang.org/reference/input-format.html
pub(crate) struct Normalizer<'a> {
    iter: Peekable<Chain<IntoIter<char, 3>, Peekable<Chars<'a>>>>,
    idx: u32,
}

impl<'a> Normalizer<'a> {
    pub fn new(src: &'a str, sp: Span) -> Self {
        let mut chars = src.chars().peekable();

        // If the first character in the sequence is U+FEFF (BYTE ORDER MARK), it is removed.
        if chars.peek() == Some(&'\u{feff}') {
            chars.next();
        }

        let mut idx = sp.lo();
        let a = chars.next();
        let b = chars.next();
        let c = chars.next();

        // if let Some(a) = a {
        //     idx += a.len_utf8() as u32;
        // }

        // if let Some(b) = b {
        //     idx += b.len_utf8() as u32;
        // }

        // if let Some(c) = c {
        //     idx += c.len_utf8() as u32;
        // }

        // If a shebang is present, it is removed from the input sequence (and is therefore ignored).
        let skip_shebang = match (a, b, c) {
            (Some('#'), Some('!'), Some(v)) if v != '[' => {
                while let Some(peek) = chars.peek()
                    && peek != &'\n'
                {
                    idx += peek.len_utf8() as u32;
                    chars.next();
                }
                ['\u{000d}'; 3].into_iter()
            }
            (Some('#'), Some('!'), Some('[')) => ['#', '!', '['].into_iter(),
            (Some(a), Some(b), Some(c)) => [a, b, c].into_iter(),
            (Some(a), Some(b), None) => [a, b, '\u{000d}'].into_iter(),
            (Some(a), None, None) => [a, '\u{000d}', '\u{000d}'].into_iter(),
            _ => ['\u{000d}'; 3].into_iter(),
        };

        let iter = skip_shebang.chain(chars).peekable();

        Self { iter, idx }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.iter.clone().peekable().peek().copied()
    }

    fn next_ch(&mut self) -> Option<char> {
        let out = self.iter.next()?;
        self.idx += out.len_utf8() as u32;
        Some(out)
    }

    pub fn idx(&self) -> u32 {
        self.idx
    }
}

impl<'a> Iterator for Normalizer<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        // Each pair of characters U+000D (CR) immediately followed by U+000A (LF) is replaced by a single U+000A (LF). This happens once, not repeatedly, so after the normalization, there can still exist U+000D (CR) immediately followed by U+000A (LF) in the input (e.g. if the raw input contained “CR CR LF LF”).

        // Other occurrences of the character U+000D (CR) are left in place (they are treated as whitespace).
        if self.peek() == Some('\u{000d}') {
            self.next_ch();
            if self.peek() == Some('\u{000a}') {
                self.next_ch();
                return Some('\u{000a}');
            } else {
                return Some('\u{000d}');
            }
        }
        self.next_ch()
    }
}
