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

        //Consume the next input code point.
        let next = self.process.next();

        if let Some(v) = next {
            if Self::is_whitespace(v) {
                // Consume as much whitespace as possible. Return a <whitespace-token>.
                self.consume_whitespace();
                return Ok(CSSToken::WhitespaceToken);
            } else if v == '\u{0022}' {
                // Consume a string token and return it.
                return self.consume_string_token(v);
            } else {
                return Err(CSSError::ParseError);
            }
        } else {
            Ok(CSSToken::EOFToken)
        }
    }

    fn consume_string_token(&mut self, code_point: char) -> Result<CSSToken, CSSError> {
        // https://www.w3.org/TR/css-syntax-3/#consume-string-token
        //This algorithm may be called with an ending code point, which denotes the code point that ends the string. If an ending code point is not specified, the current input code point is used.

        //Initially create a <string-token> with its value set to the empty string.
        let mut out = CSSToken::StringToken { string: "".into() };
        while let Some(v) = self.process.next() {
            match v {
                // ending code point :
                code_point => {
                    // Return the <string-token>.
                    return Ok(out);
                }

                //newline
                '\u{000a}' => {
                    //This is a parse error. Reconsume the current input code point, create a <bad-string-token>, and return it.
                    self.process.next();
                    return Ok(CSSToken::BadStringToken);
                }

                //U+005C REVERSE SOLIDUS (\)
                '\u{005c}' => {
                    match self.process.next() {
                        //If the next input code point is EOF, do nothing.
                        //Otherwise, if the next input code point is a new line , consume it.
                        None | Some('\n') => {}

                        v => if Self::is_valid_escape(Some('\u{005c}'), v) {},
                    }
                }
            }
        }
        // EOF :  This is a parse error. Return the <string-token>.
        Ok(out)
    }

    fn peek_twin(&mut self) -> (Option<char>, Option<char>) {
        let peek = self.process.peek_amount(2);
        let first = peek[0];
        let second = peek[1];
        (first, second)
    }

    fn consume_escaped_code_point(&mut self, input: Option<char>) -> char {
        if let Some(v) = input {
            if Self::is_hex_digit(v) {
                let mut count: u8 = 1;
                let mut code_point = String::new();
                while count == 5 {
                    if let Some(v) = self.process.next() {
                        if Self::is_hex_digit(v) {
                            code_point.push(v);
                        } else {
                            return Self::code_point_to_char(&code_point);
                        }
                    }
                    count += 1;
                }

                return Self::code_point_to_char(&code_point);
            } else {
                return v;
            }
        }
        unreachable!()
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
        while self.consume_token() != Ok(CSSToken::WhitespaceToken) {}
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
            println!("no comment to be seen here");
            return Ok(());
        }

        self.process.next();
        self.process.next();

        loop {
            (first, second) = self.peek_twin();

            if first.is_none() || second.is_none() {
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

    fn tokenize(&mut self) -> Vec<CSSToken> {
        let mut out = vec![];
        while let Ok(v) = self.consume_token() {
            if v == CSSToken::EOFToken {
                out.push(v);
                break;
            }
            out.push(v);
        }
        out
    }
}

#[cfg(test)]
mod tests2 {
    use std::hint::assert_unchecked;

    use super::*;

    #[test]
    fn test_peek_twin() {
        assert_eq!(Tokenizer::new("ok").peek_twin(), (Some('o'), Some('k')))
    }

    #[test]
    fn test_comment_consuming() {
        assert_eq!(
            Tokenizer::new("/*this is a comment*/").tokenize(),
            vec![CSSToken::EOFToken]
        )
    }

    // #[test]
    // fn test_whitespace_token() {
    //     assert_eq!(
    //         Tokenizer::new("        ").tokenize(),
    //         vec![CSSToken::WhitespaceToken, CSSToken::EOFToken]
    //     )
    // }

    // #[test]
    // fn test_comment_and_whitespace() {
    //     assert_eq!(
    //         Tokenizer::new("/* this is a comment and the following is whitespace*/     \n\t")
    //             .tokenize(),
    //         vec![CSSToken::WhitespaceToken, CSSToken::EOFToken]
    //     )
    // }

    #[test]
    fn test_code_point_to_char() {
        assert_eq!(Tokenizer::code_point_to_char("1F338"), '\u{1f338}')
    }

    #[test]
    fn test_string_token() {
        assert_eq!(
            Tokenizer::new("\"hello world!\"").tokenize(),
            vec![CSSToken::StringToken {
                string: "hello world!".into()
            }]
        )
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
