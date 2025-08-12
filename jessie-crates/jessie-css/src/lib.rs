use peekmore::{PeekMore, PeekMoreIterator};
use std::{io::repeat, str::Chars};
mod tests;

pub struct Tokenizer<'a> {
    process: PeekMoreIterator<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            process: input.chars().peekmore(),
        }
    }

    fn consume_token(&mut self) -> Result<CSSToken, CSSError> {
        // https://www.w3.org/TR/css-syntax-3/#consume-token

        // This section describes how to consume a token from a stream of code points. It will return a single token of any type.

        // Consume comments.
        self.consume_comments()?;

        match self.process.next() {
            Some(v) => {
                if Self::is_whitespace(v) {
                    // Consume as much whitespace as possible. Return a <whitespace-token>.
                    println!("whitespace found!");
                    self.consume_whitespace();
                    Ok(CSSToken::WhitespaceToken)
                } else if v == '\u{0022}' {
                    // Consume a string token and return it.
                    return self.consume_string_token(v);
                } else {
                    return Err(CSSError::ParseError);
                }
            }
            None => Ok(CSSToken::EOFToken),
        }
    }

    fn consume_string_token(&mut self, code_point: char) -> Result<CSSToken, CSSError> {
        // https://www.w3.org/TR/css-syntax-3/#consume-string-token
        //This algorithm may be called with an ending code point, which denotes the code point that ends the string. If an ending code point is not specified, the current input code point is used.

        //Initially create a <string-token> with its value set to the empty string.
        let mut out = String::new();
        while let Some(v) = self.process.peek() {
            match v {
                // ending code point :
                //newline
                '\u{000a}' => {
                    //This is a parse error. Reconsume the current input code point, create a <bad-string-token>, and return it.
                    self.process.next();
                    return Ok(CSSToken::BadStringToken);
                }

                //U+005C REVERSE SOLIDUS (\)
                '\u{005c}' => {
                    match self.process.peek_nth(1) {
                        //If the next input code point is EOF, do nothing.
                        //Otherwise, if the next input code point is a new line , consume it.
                        None => {}

                        Some('\n') => {
                            self.process.next();
                        }

                        v => {
                            if Self::is_valid_escape(Some('\u{005c}'), Some(*v.unwrap())) {
                                out.push(self.consume_escaped_code_point());
                            }
                        }
                    }
                }

                x => {
                    if *x == code_point {
                        self.process.next();
                        return Ok(CSSToken::StringToken { string: out });
                    }

                    out.push(*x);
                    self.process.next();
                    // Return the <string-token>.
                }
            }
        }
        // EOF :  This is a parse error. Return the <string-token>.
        Ok(CSSToken::StringToken { string: out })
    }

    fn peek_twin(&mut self) -> (Option<char>, Option<char>) {
        let peek = self.process.peek_amount(2);
        let first = peek[0];
        let second = peek[1];
        (first, second)
    }

    fn consume_escaped_code_point(&mut self) -> char {
        let next = self.process.next();
        match next {
            Some(v) => {
                if Self::is_hex_digit(v) {
                    let mut out = String::with_capacity(6);
                    out.push(v);
                    loop {
                        let peek = self.process.peek();
                        match peek {
                            Some(v) => {
                                if Self::is_hex_digit(*v) && out.len() != 6 {
                                    out.push(*v);
                                    self.process.next();
                                } else {
                                    if Self::is_whitespace(*v) {
                                        self.process.next();
                                    }
                                    let interpret = u32::from_str_radix(&out, 16).unwrap();
                                    if interpret == 0
                                        || Self::is_surrogate(interpret)
                                        || interpret > Self::max_allowed_code_point()
                                    {
                                        return '\u{fffd}';
                                    }
                                    return Self::code_point_to_char(&out);
                                }
                            }
                            None => return Self::code_point_to_char(&out),
                        }
                    }
                } else {
                    return v;
                }
            }
            None => {
                return '\u{fffd}';
            }
        }
    }

    fn max_allowed_code_point() -> u32 {
        0x10fff
    }

    fn is_surrogate(input: u32) -> bool {
        Self::is_leading_surrogate(input) || Self::is_trailing_surrogate(input)
    }

    fn is_leading_surrogate(input: u32) -> bool {
        0xd800 <= input && 0xdbff >= input
    }

    fn is_trailing_surrogate(input: u32) -> bool {
        0xdc00 <= input && 0xdfff >= input
    }

    fn code_point_to_char(input: &str) -> char {
        char::from_u32(u32::from_str_radix(input, 16).unwrap()).unwrap()
    }

    fn is_digit(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A code point between U+0030 DIGIT ZERO (0) and U+0039 DIGIT NINE (9) inclusive.
        input <= '\u{0039}' && input >= '\u{0030}'
    }

    fn is_hex_digit(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A digit, or a code point between U+0041 LATIN CAPITAL LETTER A (A) and U+0046 LATIN CAPITAL LETTER F (F) inclusive, or a code point between U+0061 LATIN SMALL LETTER A (a) and U+0066 LATIN SMALL LETTER F (f) inclusive.
        Self::is_digit(input)
            || (input >= '\u{0041}' && input <= '\u{0046}'
                || input >= '\u{0061}' && input <= '\u{0066}')
    }

    fn is_valid_escape(first: Option<char>, second: Option<char>) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#starts-with-a-valid-escape

        // If the first code point is not U+005C REVERSE SOLIDUS (\), return false.
        if first != Some('\u{005c}') {
            return false;
        }

        //Otherwise, if the second code point is a newline, return false.
        if second == Some('\n') {
            return false;
        }

        //Otherwise, return true.
        true
    }

    fn consume_whitespace(&mut self) {
        while !(self.process.peek().is_none() || Self::is_whitespace(*self.process.peek().unwrap()))
        {
            println!("consuming");
            println!("{:#?}", self.process.peek());
            self.process.next();
        }
    }

    fn is_whitespace(input: char) -> bool {
        input == '\u{000a}' || input == '\u{0009}' || input == '\u{0020}'
    }

    // fn consume_whitespace(&mut self) {
    //     while self.current().is_some() && self.current().unwrap().is_whitespace() {
    //         self.pos += 1;
    //         println!("{:#?}", self.current());
    //     }
    // }

    fn consume_comments(&mut self) -> Result<(), CSSError> {
        // https://www.w3.org/TR/css-syntax-3/#consume-comment
        // This section describes how to consume comments from a stream of code points. It returns nothing.
        // If the next two input code point are U+002F SOLIDUS (/) followed by a U+002A ASTERISK (*),
        let (mut first, mut second) = self.peek_twin();
        if !(first == Some('\u{002f}') && second == Some('\u{002a}')) {
            return Ok(());
        }

        self.process.next();
        self.process.next();

        loop {
            (first, second) = self.peek_twin();

            if first.is_none() {
                return Err(CSSError::ParseError);
            }

            if first == Some('\u{002a}') && second == Some('\u{002f}') {
                println!("i was true");
                self.process.next();
                self.process.next();
                self.consume_comments()?;
                return Ok(());
            }

            self.process.next();
        }
    }

    fn tokenize(&mut self) -> Result<Vec<CSSToken>, CSSError> {
        let mut out = vec![];
        loop {
            let tok = self.consume_token()?;
            if tok == CSSToken::EOFToken {
                out.push(CSSToken::EOFToken);
                break;
            }
            out.push(tok);
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CSSError {
    ParseError,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CSSToken {
    EOFToken,
    WhitespaceToken,
    StringToken { string: String },
    BadStringToken,
}
