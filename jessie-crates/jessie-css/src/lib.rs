use peekmore::{PeekMore, PeekMoreIterator};
use std::str::Chars;
mod tests;

pub struct Tokenizer<'a> {
    process: PutBackPeekMore<'a>,
    parse_error: bool,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            process: PutBackPeekMore::new(input),
            parse_error: false,
        }
    }

    fn consume_token(&mut self) -> CSSToken {
        // https://www.w3.org/TR/css-syntax-3/#consume-token

        // This section describes how to consume a token from a stream of code points. It will return a single token of any type.

        // Consume comments.
        self.consume_comments();

        match self.process.peek() {
            Some(v) => {
                let v = *v;
                if Self::is_whitespace(v) {
                    // Consume as much whitespace as possible. Return a <whitespace-token>.
                    self.process.next();
                    self.consume_whitespace();
                    return CSSToken::WhitespaceToken;
                }
                if v == '\u{0022}' {
                    self.process.next();
                    // Consume a string token and return it.
                    return self.consume_string_token(v);
                }
                if v == '\u{0023}' {
                    self.process.next();
                    let (first, second) = self.peek_twin();

                    if Self::is_ident_code_point(first) || Self::is_valid_escape(first, second) {
                        let mut flag = HashTokenFlag::Unrestricted;
                        if self.would_start_ident_sequence() {
                            flag = HashTokenFlag::Id;
                        }
                        let value = self.consume_ident_sequence();
                        return CSSToken::HashToken { flag, value };
                    }
                    return CSSToken::DelimToken { value: v };
                }
                if v == '\u{0027}' {
                    self.process.next();
                    return self.consume_string_token(v);
                }
                if v == '\u{0028}' {
                    self.process.next();
                    return CSSToken::LParenToken;
                }
                if v == '\u{0029}' {
                    self.process.next();
                    return CSSToken::RParenToken;
                }
                if v == '\u{002b}' {
                    if self.would_start_number() {
                        self.process.put_back(v);
                        return self.consume_numeric_token();
                    }
                    return CSSToken::DelimToken { value: v };
                }
                if v == '\u{002c}' {
                    return CSSToken::CommaToken;
                }
                if v == '\u{002d}' {
                    if self.would_start_number() {
                        self.process.put_back(v);
                        return self.consume_numeric_token();
                    }
                    if self.peek_twin() == (Some('\u{002d}'), Some('\u{003e}')) {
                        self.process.next();
                        self.process.next();
                        return CSSToken::CDCToken;
                    }
                    if self.would_start_ident_sequence() {
                        self.process.put_back(v);
                        return self.consume_ident_like_token();
                    }
                    return CSSToken::DelimToken { value: v };
                }
                if v == '\u{002e}' {
                    if self.would_start_number() {
                        self.process.put_back(v);
                        return self.consume_numeric_token();
                    }
                    return CSSToken::DelimToken { value: v };
                }
                if v == '\u{003a}' {
                    return CSSToken::ColonToken;
                }
                if v == '\u{003b}' {
                    return CSSToken::SemicolonToken;
                }
                if v == '\u{003c}' {
                    if *self.process.peek_amount(3)
                        == [Some('\u{0021}'), Some('\u{002d}'), Some('\u{002d}')]
                    {
                        self.process.next();
                        self.process.next();
                        self.process.next();
                        return CSSToken::CDOToken;
                    }
                    return CSSToken::DelimToken { value: v };
                }
                if v == '\u{0040}' {
                    if self.would_start_ident_sequence() {
                        return CSSToken::AtKeywordToken {
                            value: self.consume_ident_sequence(),
                        };
                    }
                    return CSSToken::DelimToken { value: v };
                }
                if v == '\u{005b}' {
                    return CSSToken::LeftSquareBracketToken;
                }
                if v == '\u{005c}' {
                    if let Some(&second) = self.process.peek()
                        && Self::is_valid_escape(Some(v), Some(second))
                    {
                        self.process.put_back(v);
                        return self.consume_ident_like_token();
                    }

                    self.parse_error();
                    return CSSToken::DelimToken { value: v };
                }
                return CSSToken::WhitespaceToken;
            }
            None => CSSToken::EOFToken,
        }
    }

    fn consume_ident_like_token(&mut self) -> CSSToken {
        let string = self.consume_ident_sequence();
        if &string.to_lowercase() == "url" && self.process.peek() == Some(&'\u{0028}') {
            self.process.next();

            while let (Some(first), Some(second)) = self.peek_twin() {
                if !(Self::is_whitespace(first) && Self::is_whitespace(second)) {
                    break;
                }
                self.process.next();
            }

            if let (Some(first), Some(second)) = self.peek_twin() {
                if (first == '\u{0022}' || first == '\u{0027}')
                    || (Self::is_whitespace(first)
                        && (second == '\u{0022}' || second == '\u{0027}'))
                {
                    return CSSToken::FunctionToken { value: string };
                } else {
                    return self.consume_url_token();
                }
            }
        }
        if self.process.peek() == Some(&'\u{0028}') {
            self.process.next();
            return CSSToken::FunctionToken { value: string };
        }
        CSSToken::IdentToken { value: string }
    }

    fn consume_url_token(&mut self) -> CSSToken {
        let mut url_token_val = String::new();
        self.consume_whitespace();
        loop {
            let next = self.process.next();
            if next == Some('\u{0029}') {
                return CSSToken::URLToken {
                    value: url_token_val,
                };
            }

            if next.is_none() {
                self.parse_error();
                return CSSToken::URLToken {
                    value: url_token_val,
                };
            }

            if Self::is_whitespace(next.unwrap()) {
                self.consume_whitespace();
                let peek = self.process.peek();
                if matches!(peek, None | Some(&'\u{0029}')) {
                    if peek.is_none() {
                        self.parse_error();
                    }
                    self.process.next();
                    return CSSToken::URLToken {
                        value: url_token_val,
                    };
                }
                self.consume_remnants_of_bad_url();
                return CSSToken::BadURLToken;
            }
            if let Some(v) = next
                && ((v == '\u{0022}')
                    || (v == '\u{0027}')
                    || (v == '\u{0028}')
                    || Self::is_none_printable_code_point(v))
            {
                self.parse_error();
                self.consume_remnants_of_bad_url();
                return CSSToken::BadURLToken;
            }

            if next == Some('\u{005c}') {
                if let Some(&v) = self.process.peek()
                    && Self::is_valid_escape(next, Some(v))
                {
                    url_token_val.push(self.consume_escaped_code_point());
                }
                self.parse_error();
                self.consume_remnants_of_bad_url();
                return CSSToken::BadURLToken;
            }

            if let Some(v) = next {
                url_token_val.push(v);
            }
        }
    }

    fn is_none_printable_code_point(input: char) -> bool {
        ('\u{0000}' <= input && '\u{0008}' >= input)
            || (input == '\u{000b}')
            || ('\u{000e}' <= input && '\u{001f}' >= input)
            || (input == '\u{007f}')
    }

    fn consume_remnants_of_bad_url(&mut self) {
        loop {
            let next = self.process.next();
            if next == Some('\u{0029}') {
                return;
            }
            if let Some(&peek) = self.process.peek()
                && Self::is_valid_escape(next, Some(peek))
            {
                self.consume_escaped_code_point();
            }
        }
    }

    fn consume_numeric_token(&mut self) -> CSSToken {
        let number = self.consume_number();
        if self.would_start_ident_sequence() {
            return CSSToken::DimensionToken {
                flag: number.r#type,
                value: number.value,
                unit: self.consume_ident_sequence(),
            };
        }
        if self.process.peek() == Some(&'\u{0025}') {
            self.process.next();
            return CSSToken::PercentageToken {
                flag: number.r#type,
                value: number.value,
            };
        }
        CSSToken::NumberToken {
            flag: number.r#type,
            value: number.value,
        }
    }

    fn consume_number(&mut self) -> Number {
        let mut r#type = NumberType::Integer;
        let mut repr = String::new();

        if let Some(&v) = self.process.peek()
            && (v == '\u{002b}' || v == '\u{002d}')
        {
            self.process.next();
            repr.push(v);
        }

        while let Some(&v) = self.process.peek() {
            if !Self::is_digit(v) {
                break;
            }
            self.process.next();
            repr.push(v);
        }

        if let (Some(first), Some(second)) = self.peek_twin()
            && (first == '\u{002e}' && Self::is_digit(second))
        {
            self.process.next();
            self.process.next();
            repr.push(first);
            repr.push(second);
            r#type = NumberType::Number;
            while let Some(&v) = self.process.peek() {
                if !Self::is_digit(v) {
                    break;
                }
                self.process.next();
                repr.push(v);
            }
        }

        if let &[Some(first), Some(second), Some(third)] = self.process.peek_amount(3)
            && ((Self::is_e(first) && Self::is_digit(second))
                || (Self::is_e(first) && Self::is_plus_or_minus(second) && Self::is_digit(third)))
        {
            self.process.next();
            self.process.next();
            repr.push(first);
            repr.push(second);
            if Self::is_e(first) && Self::is_plus_or_minus(second) && Self::is_digit(third) {
                self.process.next();
                repr.push(third);
            }
            r#type = NumberType::Number;
            while let Some(&v) = self.process.peek() {
                if !Self::is_digit(v) {
                    break;
                }
                self.process.next();
                repr.push(v);
            }
        }

        Number {
            value: Self::string_to_number(repr),
            r#type,
        }
    }

    fn string_to_number(input: String) -> f64 {
        let mut iter = input.chars().peekable();
        let mut _sign: Option<char> = None;
        if let Some(&v) = iter.peek()
            && (v == '\u{002b}' || v == '\u{002d}')
        {
            iter.next();
            _sign = Some(v);
        }
        let s: f64 = {
            match _sign {
                Some('\u{002d}') => -1.,
                _ => 1.,
            }
        };
        let mut integer_part = String::new();
        while let Some(&v) = iter.peek() {
            if !Self::is_digit(v) {
                break;
            }
            integer_part.push(v);
            iter.next();
        }
        let i = integer_part.parse::<f64>().unwrap_or(0.);
        let mut _decimal_point = None;
        if let Some(&v) = iter.peek()
            && (v == '\u{002e}')
        {
            iter.next();
            _decimal_point = Some(v);
        }

        let mut fractional_part = String::new();
        while let Some(&v) = iter.peek() {
            if !Self::is_digit(v) {
                break;
            }
            fractional_part.push(v);
            iter.next();
        }

        let f = fractional_part.parse::<f64>().unwrap_or(0.);
        let d = fractional_part.len() as f64;

        let mut _exponent_indicator = None;
        if let Some(&v) = iter.peek()
            && (v == '\u{0045}' || v == '\u{0065}')
        {
            iter.next();
            _exponent_indicator = Some(v);
        }

        let mut _exponent_sign = None;

        if let Some(&v) = iter.peek()
            && (v == '\u{002b}' || v == '\u{002d}')
        {
            iter.next();
            _exponent_sign = Some(v);
        }

        let t = {
            match _exponent_sign {
                Some('\u{002d}') => -1.,
                _ => 1.,
            }
        };

        let mut exponent = String::new();
        while let Some(&v) = iter.peek() {
            if !Self::is_digit(v) {
                break;
            }
            exponent.push(v);
            iter.next();
        }

        let e = exponent.parse::<f64>().unwrap_or(0.);

        s * (i + (f * 10.0_f64.powf(-d))) * 10.0_f64.powf(t * e)
    }

    fn is_plus_or_minus(input: char) -> bool {
        input == '\u{002d}' || input == '\u{002b}'
    }

    fn is_e(input: char) -> bool {
        input == '\u{0045}' || input == '\u{0065}'
    }

    fn would_start_number(&mut self) -> bool {
        let peek = self.process.peek_amount(3);
        let first = peek[0];
        if let Some(v) = first {
            if v == '\u{002b}' || v == '\u{002d}' {
                if let Some(second_val) = peek[1]
                    && Self::is_digit(second_val)
                {
                    return true;
                }
                if peek[1] == Some('\u{002e}')
                    && let Some(third_val) = peek[2]
                    && Self::is_digit(third_val)
                {
                    return true;
                }
                return false;
            }
        }
        false
    }

    fn consume_ident_sequence(&mut self) -> String {
        let mut result = String::new();
        loop {
            let (first, second) = self.peek_twin();
            if let Some(v) = first {
                if Self::is_ident_code_point(first) {
                    result.push(v);
                }
                if Self::is_valid_escape(first, second) {
                    self.process.next();
                    result.push(self.consume_escaped_code_point());
                }
                let _ = self.process.put_back(v);
                return result;
            }
            self.process.next();
        }
    }

    fn would_start_ident_sequence(&mut self) -> bool {
        let peek = self.process.peek_amount(3);
        if let Some(first) = peek[0] {
            if first == '\u{002d}' {
                if let Some(second) = peek[1] {
                    return Self::is_ident_start_code_point(second)
                        || Self::is_valid_escape(peek[1], peek[2]);
                }
            }
            if Self::is_ident_start_code_point(first) {
                return true;
            }
            if first == '\u{005c}' {
                return Self::is_valid_escape(peek[0], peek[1]);
            }
        }
        false
    }

    fn is_ident_code_point(input: Option<char>) -> bool {
        if let Some(v) = input {
            return Self::is_ident_start_code_point(v) || Self::is_digit(v) || v == '\u{002d}';
        }
        false
    }

    fn is_ident_start_code_point(input: char) -> bool {
        Self::is_letter(input) || Self::is_none_ascii(input) || input == '\u{0080}'
    }

    fn is_letter(input: char) -> bool {
        Self::is_uppercase_letter(input) || Self::is_lowercase_letter(input)
    }

    fn is_uppercase_letter(input: char) -> bool {
        input >= '\u{0041}' && input <= '\u{005a}'
    }

    fn is_lowercase_letter(input: char) -> bool {
        input >= '\u{0061}' && input <= '\u{007a}'
    }

    fn is_none_ascii(input: char) -> bool {
        input >= '\u{0080}'
    }

    fn consume_string_token(&mut self, code_point: char) -> CSSToken {
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
                    return CSSToken::BadStringToken;
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
                        return CSSToken::StringToken { string: out };
                    }

                    out.push(*x);
                    self.process.next();
                    // Return the <string-token>.
                }
            }
        }
        // EOF :  This is a parse error. Return the <string-token>.
        self.parse_error();
        CSSToken::StringToken { string: out }
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
        0x10ffff
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

    fn consume_comments(&mut self) {
        // https://www.w3.org/TR/css-syntax-3/#consume-comment
        // This section describes how to consume comments from a stream of code points. It returns nothing.
        // If the next two input code point are U+002F SOLIDUS (/) followed by a U+002A ASTERISK (*),
        let (mut first, mut second) = self.peek_twin();
        if !(first == Some('\u{002f}') && second == Some('\u{002a}')) {
            return;
        }

        self.process.next();
        self.process.next();

        loop {
            (first, second) = self.peek_twin();

            if first.is_none() {
                self.parse_error();
                return;
            }

            if first == Some('\u{002a}') && second == Some('\u{002f}') {
                println!("i was true");
                self.process.next();
                self.process.next();
                self.consume_comments();
                return;
            }

            self.process.next();
        }
    }

    fn parse_error(&mut self) {
        self.parse_error = true;
    }

    fn tokenize(&mut self) -> Vec<CSSToken> {
        let mut out = vec![];
        loop {
            let tok = self.consume_token();
            if tok == CSSToken::EOFToken {
                out.push(CSSToken::EOFToken);
                break;
            }
            out.push(tok);
        }
        out
    }
}

#[derive(Debug, PartialEq)]
pub enum CSSToken {
    EOFToken,
    WhitespaceToken,
    StringToken {
        string: String,
    },
    BadStringToken,
    HashToken {
        flag: HashTokenFlag,
        value: String,
    },
    DelimToken {
        value: char,
    },
    LParenToken,
    RParenToken,
    NumberToken {
        flag: NumberType,
        value: f64,
    },
    PercentageToken {
        flag: NumberType,
        value: f64,
    },
    DimensionToken {
        flag: NumberType,
        value: f64,
        unit: String,
    },
    CommaToken,
    CDCToken,
    FunctionToken {
        value: String,
    },
    URLToken {
        value: String,
    },
    BadURLToken,
    IdentToken {
        value: String,
    },
    ColonToken,
    SemicolonToken,
    CDOToken,
    AtKeywordToken {
        value: String,
    },
    LeftSquareBracketToken,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HashTokenFlag {
    Id,
    Unrestricted,
}

#[derive(Debug, PartialEq)]
pub struct Number {
    value: f64,
    r#type: NumberType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NumberType {
    Integer,
    Number,
}

pub struct PutBackPeekMore<'a> {
    pub peek_more: PeekMoreIterator<Chars<'a>>,
    pub put_back: Option<char>,
}

impl<'a> PutBackPeekMore<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            put_back: None,
            peek_more: input.chars().peekmore(),
        }
    }

    fn next(&mut self) -> Option<char> {
        if let Some(v) = self.put_back {
            let returned = v;
            self.put_back = None;
            return Some(v);
        }
        self.peek_more.next()
    }

    fn put_back(&mut self, input: char) {
        self.put_back = Some(input);
    }

    fn peek(&mut self) -> Option<&char> {
        self.peek_more.peek()
    }

    fn peek_amount(&mut self, amount: usize) -> &[Option<char>] {
        self.peek_more.peek_amount(amount)
    }

    fn peek_nth(&mut self, amount: usize) -> Option<&char> {
        self.peek_more.peek_nth(amount)
    }
}
