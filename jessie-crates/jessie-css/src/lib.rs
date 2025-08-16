use peekmore::{PeekMore, PeekMoreIterator};
use std::{borrow::Cow, str::Chars};
mod tests;

pub struct Tokenizer<'a> {
    process: PutBackPeekMore<'a>,
    parse_error: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            process: PutBackPeekMore::new(input),
            parse_error: false,
        }
    }

    pub fn consume_token(&mut self) -> CSSToken {
        // https://www.w3.org/TR/css-syntax-3/#consume-token

        // This section describes how to consume a token from a stream of code points. It will return a single token of any type.

        // Consume comments.
        self.consume_comments();

        // Consume the next input code point.
        match self.process.next() {
            Some(v) => {
                // whitespace
                if Self::is_whitespace(v) {
                    // Consume as much whitespace as possible. Return a <whitespace-token>.
                    self.consume_whitespace();
                    return CSSToken::WhitespaceToken;
                }
                // U+0022 QUOTATION MARK (")
                if v == '\u{0022}' {
                    // Consume a string token and return it.
                    return self.consume_string_token(v);
                }
                // U+0023 NUMBER SIGN (#)
                if v == '\u{0023}' {
                    let (first, second) = self.peek_twin();

                    // If the next input code point is an ident code point or the next two input code points are a valid escape, then:
                    if Self::is_ident_code_point(first) || Self::is_valid_escape(first, second) {
                        // Create a <hash-token>.  NOTE: we cant really lol
                        let mut flag = HashTokenFlag::Unrestricted;
                        //  If the next 3 input code points would start an ident sequence, set the <hash-token>’s type flag to "id".
                        if self.would_start_ident_sequence() {
                            flag = HashTokenFlag::Id;
                        }
                        // Consume an ident sequence, and set the <hash-token>’s value to the returned string.
                        let value = self.consume_ident_sequence();
                        // Return the <hash-token>.
                        return CSSToken::HashToken { flag, value };
                    }
                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    return CSSToken::DelimToken { value: v };
                }
                // U+0027 APOSTROPHE (')
                if v == '\u{0027}' {
                    // Consume a string token and return it.
                    return self.consume_string_token(v);
                }
                // U+0028 LEFT PARENTHESIS (()
                if v == '\u{0028}' {
                    // Return a <(-token>.
                    return CSSToken::LeftParenthesisToken;
                }
                // U+0029 RIGHT PARENTHESIS ())
                if v == '\u{0029}' {
                    // Return a <)-token>.
                    return CSSToken::RightParenthesisToken;
                }
                // U+002B PLUS SIGN (+)
                if v == '\u{002b}' {
                    // If the input stream starts with a number, reconsume the current input code point, consume a numeric token, and return it.
                    if self.would_start_number() {
                        self.process.put_back(v);
                        return self.consume_numeric_token();
                    }
                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    return CSSToken::DelimToken { value: v };
                }
                // U+002C COMMA (,)
                if v == '\u{002c}' {
                    // Return a <comma-token>.
                    return CSSToken::CommaToken;
                }
                // U+002D HYPHEN-MINUS (-)
                if v == '\u{002d}' {
                    // If the input stream starts with a number, reconsume the current input code point, consume a numeric token, and return it.
                    if self.would_start_number() {
                        self.process.put_back(v);
                        return self.consume_numeric_token();
                    }
                    // Otherwise, if the next 2 input code points are U+002D HYPHEN-MINUS U+003E GREATER-THAN SIGN (->), consume them and return a <CDC-token>.
                    if self.peek_twin() == (Some('\u{002d}'), Some('\u{003e}')) {
                        self.process.next();
                        self.process.next();
                        return CSSToken::CDCToken;
                    }
                    // Otherwise, if the input stream starts with an ident sequence, reconsume the current input code point, consume an ident-like token, and return it.
                    if self.would_start_ident_sequence() {
                        self.process.put_back(v);
                        return self.consume_ident_like_token();
                    }
                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    return CSSToken::DelimToken { value: v };
                }
                // U+002E FULL STOP (.)
                if v == '\u{002e}' {
                    // If the input stream starts with a number, reconsume the current input code point, consume a numeric token, and return it.
                    if self.would_start_number() {
                        self.process.put_back(v);
                        return self.consume_numeric_token();
                    }
                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    return CSSToken::DelimToken { value: v };
                }
                // U+003A COLON (:)
                if v == '\u{003a}' {
                    // Return a <colon-token>.
                    return CSSToken::ColonToken;
                }
                // U+003B SEMICOLON (;)
                if v == '\u{003b}' {
                    // Return a <semicolon-token>.
                    return CSSToken::SemicolonToken;
                }
                // U+003C LESS-THAN SIGN (<)
                if v == '\u{003c}' {
                    // If the next 3 input code points are U+0021 EXCLAMATION MARK U+002D HYPHEN-MINUS U+002D HYPHEN-MINUS (!--), consume them and return a <CDO-token>.
                    if self.process.peek_amount(3)
                        == [Some('\u{0021}'), Some('\u{002d}'), Some('\u{002d}')]
                    {
                        self.process.next();
                        self.process.next();
                        self.process.next();
                        return CSSToken::CDOToken;
                    }
                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    return CSSToken::DelimToken { value: v };
                }
                // U+0040 COMMERCIAL AT (@)
                if v == '\u{0040}' {
                    // If the next 3 input code points would start an ident sequence, consume an ident sequence, create an <at-keyword-token> with its value set to the returned value, and return it.
                    if self.would_start_ident_sequence() {
                        return CSSToken::AtKeywordToken {
                            value: self.consume_ident_sequence(),
                        };
                    }
                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    return CSSToken::DelimToken { value: v };
                }
                // U+005B LEFT SQUARE BRACKET ([)
                if v == '\u{005b}' {
                    // Return a <[-token>.
                    return CSSToken::LeftSquareBracketToken;
                }
                // U+005C REVERSE SOLIDUS (\)
                if v == '\u{005c}' {
                    // If the input stream starts with a valid escape, reconsume the current input code point, consume an ident-like token, and return it.
                    if let Some(&second) = self.process.peek()
                        && Self::is_valid_escape(Some(v), Some(second))
                    {
                        self.process.put_back(v);
                        return self.consume_ident_like_token();
                    }
                    // Otherwise, this is a parse error. Return a <delim-token> with its value set to the current input code point.
                    self.parse_error();
                    return CSSToken::DelimToken { value: v };
                }
                // U+005D RIGHT SQUARE BRACKET (])
                if v == '\u{005d}' {
                    // Return a <]-token>.
                    return CSSToken::RightSquareBracketToken;
                }
                // U+007B LEFT CURLY BRACKET ({)
                if v == '\u{007b}' {
                    // Return a <{-token>.
                    return CSSToken::LeftCurlyBracketToken;
                }
                // U+007D RIGHT CURLY BRACKET (})
                if v == '\u{007d}' {
                    // Return a <}-token>.
                    return CSSToken::RightCurlyBracketToken;
                }
                // digit
                if Self::is_digit(v) {
                    // Reconsume the current input code point, consume a numeric token, and return it.
                    self.process.put_back(v);
                    return self.consume_numeric_token();
                }
                // ident-start code point
                if Self::is_ident_start_code_point(v) {
                    // Reconsume the current input code point, consume an ident-like token, and return it.
                    self.process.put_back(v);
                    return self.consume_ident_like_token();
                }
                // anything else
                // Return a <delim-token> with its value set to the current input code point.
                CSSToken::DelimToken { value: v }
            }
            // EOF
            // Return an <EOF-token>.
            None => CSSToken::EOFToken,
        }
    }

    pub fn is_parse_error(&mut self) -> bool {
        self.parse_error
    }

    pub fn consume_ident_like_token(&mut self) -> CSSToken {
        // https://www.w3.org/TR/css-syntax-3/#consume-ident-like-token
        // This section describes how to consume an ident-like token from a stream of code points. It returns an <ident-token>, <function-token>, <url-token>, or <bad-url-token>.

        // Consume an ident sequence, and let string be the result.
        let string = self.consume_ident_sequence();

        // If string’s value is an ASCII case-insensitive match for "url", and the next input code point is U+0028 LEFT PARENTHESIS ((),
        if &string.to_lowercase() == "url" && self.process.peek() == Some(&'\u{0028}') {
            //  consume it.
            self.process.next();

            // 	While the next two input code points are whitespace, consume the next input code point.
            while let (Some(first), Some(second)) = self.peek_twin() {
                if !(Self::is_whitespace(first) && Self::is_whitespace(second)) {
                    break;
                }
                self.process.next();
            }

            // If the next one or two input code points are U+0022 QUOTATION MARK ("), U+0027 APOSTROPHE ('), or whitespace followed by U+0022 QUOTATION MARK (") or U+0027 APOSTROPHE ('),
            if let (Some(first), Some(second)) = self.peek_twin() {
                if (first == '\u{0022}' || first == '\u{0027}')
                    || (Self::is_whitespace(first)
                        && (second == '\u{0022}' || second == '\u{0027}'))
                {
                    // then create a <function-token> with its value set to string and return it.
                    return CSSToken::FunctionToken { value: string };
                }
                // 	Otherwise, consume a url token, and return it.
                return self.consume_url_token();
            }
        }
        // Otherwise, if the next input code point is U+0028 LEFT PARENTHESIS ((), consume it.
        if self.process.peek() == Some(&'\u{0028}') {
            self.process.next();
            // Create a <function-token> with its value set to string and return it.
            return CSSToken::FunctionToken { value: string };
        }
        // Otherwise, create an <ident-token> with its value set to string and return it.
        CSSToken::IdentToken { value: string }
    }

    pub fn consume_url_token(&mut self) -> CSSToken {
        // https://www.w3.org/TR/css-syntax-3/#consume-url-token
        // This section describes how to consume a url token from a stream of code points. It returns either a <url-token> or a <bad-url-token>.

        // Initially create a <url-token> with its value set to the empty string.
        let mut url_token_val = String::new();

        //  Consume as much whitespace as possible.
        self.consume_whitespace();

        // Repeatedly consume the next input code point from the stream:
        loop {
            let next = self.process.next();

            // U+0029 RIGHT PARENTHESIS ())
            if next == Some('\u{0029}') {
                // Return the <url-token>.
                return CSSToken::URLToken {
                    value: url_token_val,
                };
            }

            // EOF
            if next.is_none() {
                // This is a parse error. Return the <url-token>.
                self.parse_error();
                return CSSToken::URLToken {
                    value: url_token_val,
                };
            }

            // whitespace
            if Self::is_whitespace(next.unwrap()) {
                // Consume as much whitespace as possible.
                self.consume_whitespace();

                // 	If the next input code point is U+0029 RIGHT PARENTHESIS ()) or EOF,
                let peek = self.process.peek();
                if matches!(peek, None | Some(&'\u{0029}')) {
                    // 	consume it and return the <url-token>  (if EOF was encountered, this is a parse error);
                    if peek.is_none() {
                        self.parse_error();
                    }
                    self.process.next();
                    return CSSToken::URLToken {
                        value: url_token_val,
                    };
                }

                // otherwise, consume the remnants of a bad url, create a <bad-url-token>, and return it.
                self.consume_remnants_of_bad_url();
                return CSSToken::BadURLToken;
            }

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            // U+0028 LEFT PARENTHESIS (()
            // non-printable code point
            if let Some(v) = next
                && ((v == '\u{0022}')
                    || (v == '\u{0027}')
                    || (v == '\u{0028}')
                    || Self::is_none_printable_code_point(v))
            {
                // This is a parse error. Consume the remnants of a bad url, create a <bad-url-token>, and return it.
                self.parse_error();
                self.consume_remnants_of_bad_url();
                return CSSToken::BadURLToken;
            }

            // U+005C REVERSE SOLIDUS (\)
            if next == Some('\u{005c}') {
                // If the stream starts with a valid escape, consume an escaped code point and append the returned code point to the <url-token>’s value.
                if let Some(&v) = self.process.peek()
                    && Self::is_valid_escape(next, Some(v))
                {
                    url_token_val.push(self.consume_escaped_code_point());
                }

                // Otherwise, this is a parse error. Consume the remnants of a bad url, create a <bad-url-token>, and return it.
                self.parse_error();
                self.consume_remnants_of_bad_url();
                return CSSToken::BadURLToken;
            }

            // anything else
            if let Some(v) = next {
                // Append the current input code point to the <url-token>’s value.
                url_token_val.push(v);
            }
        }
    }

    pub fn is_none_printable_code_point(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A code point between U+0000 NULL and U+0008 BACKSPACE inclusive, or U+000B LINE TABULATION, or a code point between U+000E SHIFT OUT and U+001F INFORMATION SEPARATOR ONE inclusive, or U+007F DELETE.
        (('\u{0000}'..='\u{0008}').contains(&input))
            || (input == '\u{000b}')
            || (('\u{000e}'..='\u{001f}').contains(&input))
            || (input == '\u{007f}')
    }

    pub fn consume_remnants_of_bad_url(&mut self) {
        // https://www.w3.org/TR/css-syntax-3/#consume-remnants-of-bad-url
        loop {
            // Repeatedly consume the next input code point from the stream:
            let next = self.process.next();

            // U+0029 RIGHT PARENTHESIS ())
            // EOF
            if next == Some('\u{0029}') || next.is_none() {
                return;
            }

            // the input stream starts with a valid escape
            if let Some(&peek) = self.process.peek()
                && Self::is_valid_escape(next, Some(peek))
            {
                // Consume an escaped code point. This allows an escaped right parenthesis ("\)") to be encountered without ending the <bad-url-token>. This is otherwise identical to the "anything else" clause.
                self.consume_escaped_code_point();
            }

            // anything else
            // Do nothing.
        }
    }

    pub fn consume_numeric_token(&mut self) -> CSSToken {
        // https://www.w3.org/TR/css-syntax-3/#consume-numeric-token

        // This section describes how to consume a numeric token from a stream of code points. It returns either a <number-token>, <percentage-token>, or <dimension-token>.

        // Consume a number and let number be the result.
        let number = self.consume_number();

        // If the next 3 input code points would start an ident sequence, then:
        if self.would_start_ident_sequence() {
            // Create a <dimension-token> with the same value and type flag as number, and a unit set initially to the empty string.
            // Consume an ident sequence. Set the <dimension-token>’s unit to the returned value.
            // Return the <dimension-token>.
            return CSSToken::DimensionToken {
                flag: number.r#type,
                value: number.value,
                unit: self.consume_ident_sequence(),
            };
        }

        // Otherwise, if the next input code point is U+0025 PERCENTAGE SIGN (%), consume it.
        if self.process.peek() == Some(&'\u{0025}') {
            self.process.next();

            // 	Create a <percentage-token> with the same value as number, and return it.
            return CSSToken::PercentageToken {
                flag: number.r#type,
                value: number.value,
            };
        }

        // Otherwise, create a <number-token> with the same value and type flag as number, and return it.
        CSSToken::NumberToken {
            flag: number.r#type,
            value: number.value,
        }
    }

    pub fn consume_number(&mut self) -> Number {
        // https://www.w3.org/TR/css-syntax-3/#consume-number
        // This section describes how to consume a number from a stream of code points. It returns a numeric value, and a type which is either "integer" or "number".

        //  Initially set type to "integer". Let repr be the empty string.
        let mut r#type = NumberType::Integer;
        let mut repr = String::new();

        // If the next input code point is U+002B PLUS SIGN (+) or U+002D HYPHEN-MINUS (-), consume it and append it to repr.
        if let Some(&v) = self.process.peek()
            && (v == '\u{002b}' || v == '\u{002d}')
        {
            self.process.next();
            repr.push(v);
        }

        // While the next input code point is a digit, consume it and append it to repr.
        while let Some(&v) = self.process.peek() {
            if !Self::is_digit(v) {
                break;
            }
            self.process.next();
            repr.push(v);
        }

        // If the next 2 input code points are U+002E FULL STOP (.) followed by a digit, then:
        if let (Some(first), Some(second)) = self.peek_twin()
            && (first == '\u{002e}' && Self::is_digit(second))
        {
            // Consume them.
            self.process.next();
            self.process.next();
            // Append them to repr.
            repr.push(first);
            repr.push(second);
            // Set type to "number".
            r#type = NumberType::Number;

            // While the next input code point is a digit, consume it and append it to repr.
            while let Some(&v) = self.process.peek() {
                if !Self::is_digit(v) {
                    break;
                }
                self.process.next();
                repr.push(v);
            }
        }

        // If the next 2 or 3 input code points are U+0045 LATIN CAPITAL LETTER E (E) or U+0065 LATIN SMALL LETTER E (e), optionally followed by U+002D HYPHEN-MINUS (-) or U+002B PLUS SIGN (+), followed by a digit, then:
        if let &[Some(first), Some(second), Some(third)] = self.process.peek_amount(3).as_slice()
            && ((Self::is_e(first) && Self::is_digit(second))
                || (Self::is_e(first) && Self::is_plus_or_minus(second) && Self::is_digit(third)))
        {
            // Consume them.
            // Append them to repr
            self.process.next();
            self.process.next();
            repr.push(first);
            repr.push(second);
            if Self::is_e(first) && Self::is_plus_or_minus(second) && Self::is_digit(third) {
                self.process.next();
                repr.push(third);
            }

            // Set type to "number".
            r#type = NumberType::Number;

            // While the next input code point is a digit, consume it and append it to repr.
            while let Some(&v) = self.process.peek() {
                if !Self::is_digit(v) {
                    break;
                }
                self.process.next();
                repr.push(v);
            }
        }

        // Convert repr to a number, and set the value to the returned value.
        // Return value and type.
        Number {
            value: Self::string_to_number(repr),
            r#type,
        }
    }

    pub fn string_to_number(input: String) -> f64 {
        // https://www.w3.org/TR/css-syntax-3/#convert-string-to-number
        // This section describes how to convert a string to a number. It returns a number.
        let mut iter = input.chars().peekable();
        let mut _sign: Option<char> = None;

        // Divide the string into seven components, in order from left to right:
        // A sign: a single U+002B PLUS SIGN (+) or U+002D HYPHEN-MINUS (-), or the empty string.
        if let Some(&v) = iter.peek()
            && (v == '\u{002b}' || v == '\u{002d}')
        {
            iter.next();
            _sign = Some(v);
        }
        // 	Let s be the number -1 if the sign is U+002D HYPHEN-MINUS (-); otherwise, let s be the number 1.
        let s: f64 = {
            match _sign {
                Some('\u{002d}') => -1.,
                _ => 1.,
            }
        };

        // An integer part: zero or more digits.
        let mut integer_part = String::new();
        while let Some(&v) = iter.peek() {
            if !Self::is_digit(v) {
                break;
            }
            integer_part.push(v);
            iter.next();
        }

        // If there is at least one digit, let i be the number formed by interpreting the digits as a base-10 integer; otherwise, let i be the number 0.
        let i = integer_part.parse::<f64>().unwrap_or(0.);

        // A decimal point: a single U+002E FULL STOP (.), or the empty string.
        let mut _decimal_point = None;
        if let Some(&v) = iter.peek()
            && (v == '\u{002e}')
        {
            iter.next();
            _decimal_point = Some(v);
        }

        // A fractional part: zero or more digits.
        let mut fractional_part = String::new();
        while let Some(&v) = iter.peek() {
            if !Self::is_digit(v) {
                break;
            }
            fractional_part.push(v);
            iter.next();
        }

        // If there is at least one digit, let f be the number formed by interpreting the digits as a base-10 integer and d be the number of digits; otherwise, let f and d be the number 0.
        let f = fractional_part.parse::<f64>().unwrap_or(0.);
        let d = fractional_part.len() as f64;

        // An exponent indicator: a single U+0045 LATIN CAPITAL LETTER E (E) or U+0065 LATIN SMALL LETTER E (e), or the empty string.
        let mut _exponent_indicator = None;
        if let Some(&v) = iter.peek()
            && (v == '\u{0045}' || v == '\u{0065}')
        {
            iter.next();
            _exponent_indicator = Some(v);
        }

        // An exponent sign: a single U+002B PLUS SIGN (+) or U+002D HYPHEN-MINUS (-), or the empty string.
        let mut _exponent_sign = None;

        if let Some(&v) = iter.peek()
            && (v == '\u{002b}' || v == '\u{002d}')
        {
            iter.next();
            _exponent_sign = Some(v);
        }

        // Let t be the number -1 if the sign is U+002D HYPHEN-MINUS (-); otherwise, let t be the number 1.
        let t = {
            match _exponent_sign {
                Some('\u{002d}') => -1.,
                _ => 1.,
            }
        };

        // An exponent: zero or more digits.
        let mut exponent = String::new();
        while let Some(&v) = iter.peek() {
            if !Self::is_digit(v) {
                break;
            }
            exponent.push(v);
            iter.next();
        }

        // If there is at least one digit, let e be the number formed by interpreting the digits as a base-10 integer; otherwise, let e be the number 0.
        let e = exponent.parse::<f64>().unwrap_or(0.);

        // Return the number s·(i + f·10-d)·10te.
        s * (i + (f * 10.0_f64.powf(-d))) * 10.0_f64.powf(t * e)
    }

    pub fn is_plus_or_minus(input: char) -> bool {
        input == '\u{002d}' || input == '\u{002b}'
    }

    pub fn is_e(input: char) -> bool {
        input == '\u{0045}' || input == '\u{0065}'
    }

    pub fn would_start_number(&mut self) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#starts-with-a-number
        let peek = self.process.peek_amount(3);

        // Look at the first code point:
        let first = peek[0];

        // U+002B PLUS SIGN (+)
        // U+002D HYPHEN-MINUS (-)
        if first == Some('\u{002b}') || first == Some('\u{002d}') {
            if let Some(second_val) = peek[1]
                && Self::is_digit(second_val)
            {
                // If the second code point is a digit, return true.
                return true;
            }

            // Otherwise, if the second code point is a U+002E FULL STOP (.) and the third code point is a digit, return true.
            if peek[1] == Some('\u{002e}')
                && let Some(third_val) = peek[2]
                && Self::is_digit(third_val)
            {
                return true;
            }

            // Otherwise, return false.
            return false;
        }

        // U+002E FULL STOP (.)
        if first == Some('\u{002e}') {
            // If the second code point is a digit, return true. Otherwise, return false.
            if let Some(second) = peek[1]
                && Self::is_digit(second)
            {
                return true;
            }

            return false;
        }

        // digit
        if let Some(v) = first
            && Self::is_digit(v)
        {
            // Return true.
            return true;
        }

        // anything else
        // Return false.
        false
    }

    pub fn consume_ident_sequence(&mut self) -> String {
        // https://www.w3.org/TR/css-syntax-3/#consume-name

        // Let result initially be an empty string.
        let mut result = String::new();

        // Repeatedly consume the next input code point from the stream:
        loop {
            let next = self.process.next();

            // ident code point
            if let Some(v) = next
                && Self::is_ident_code_point(Some(v))
            {
                // Append the code point to result.
                result.push(v);
            }
            // the stream starts with a valid escape
            else if let Some(&v) = self.process.peek()
                && Self::is_valid_escape(next, Some(v))
            {
                // Consume an escaped code point. Append the returned code point to result.
                result.push(self.consume_escaped_code_point());
            }
            // anything else
            else {
                // Reconsume the current input code point. Return result.
                self.process.put_back_option(next);
                return result;
            }
        }
    }

    pub fn would_start_ident_sequence(&mut self) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#would-start-an-identifier
        let peek = self.process.peek_amount(3);
        let first = peek[0];
        let second = peek[1];
        let third = peek[2];

        // Look at the first code point:

        // U+002D HYPHEN-MINUS
        if first == Some('\u{002d}') {
            // If the second code point is an ident-start code point or a U+002D HYPHEN-MINUS, or the second and third code points are a valid escape, return true
            if let Some(v) = second
                && (Self::is_ident_start_code_point(v) || v == '\u{002d}')
            {
                return true;
            }

            if Self::is_valid_escape(second, third) {
                return true;
            }

            // Otherwise, return false.
            return false;
        }

        // ident-start code point
        if let Some(v) = first
            && Self::is_ident_start_code_point(v)
        {
            return true;
        }

        // U+005C REVERSE SOLIDUS (\)
        if first == Some('\u{005c}') {
            // If the first and second code points are a valid escape, return true. Otherwise, return false.
            return Self::is_valid_escape(first, second);
        }
        // anything else
        // Return false.
        false
    }

    pub fn is_ident_code_point(input: Option<char>) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // An ident-start code point, a digit, or U+002D HYPHEN-MINUS (-).
        if let Some(v) = input {
            return Self::is_ident_start_code_point(v) || Self::is_digit(v) || v == '\u{002d}';
        }
        false
    }

    pub fn is_ident_start_code_point(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A letter, a non-ASCII code point, or U+005F LOW LINE (_).
        Self::is_letter(input) || Self::is_none_ascii(input) || input == '\u{0080}'
    }

    pub fn is_letter(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // An uppercase letter or a lowercase letter.
        Self::is_uppercase_letter(input) || Self::is_lowercase_letter(input)
    }

    pub fn is_uppercase_letter(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A code point between U+0041 LATIN CAPITAL LETTER A (A) and U+005A LATIN CAPITAL LETTER Z (Z) inclusive.
        ('\u{0041}'..='\u{005a}').contains(&input)
    }

    pub fn is_lowercase_letter(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A code point between U+0061 LATIN SMALL LETTER A (a) and U+007A LATIN SMALL LETTER Z (z) inclusive.
        ('\u{0061}'..='\u{007a}').contains(&input)
    }

    pub fn is_none_ascii(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A code point with a value equal to or greater than U+0080 <control>.
        input >= '\u{0080}'
    }

    pub fn consume_string_token(&mut self, code_point: char) -> CSSToken {
        // https://www.w3.org/TR/css-syntax-3/#consume-string-token
        // This section describes how to consume a string token from a stream of code points. It returns either a <string-token> or <bad-string-token>.

        //This algorithm may be called with an ending code point, which denotes the code point that ends the string. If an ending code point is not specified, the current input code point is used.

        //Initially create a <string-token> with its value set to the empty string.
        let mut out = String::new();

        // Repeatedly consume the next input code point from the stream:
        loop {
            let next = self.process.next();
            // ending code point
            if next == Some(code_point) {
                // Return the <string-token>.
                return CSSToken::StringToken { string: out };
            }

            //EOF
            if next.is_none() {
                // This is a parse error. Return the <string-token>.
                self.parse_error();
                return CSSToken::StringToken { string: out };
            }
            // newline
            if next == Some('\u{000a}') {
                // This is a parse error. Reconsume the current input code point, create a <bad-string-token>, and return it.
                self.parse_error();
                self.process.put_back('\u{000a}');
                return CSSToken::BadStringToken;
            }
            // U+005C REVERSE SOLIDUS (\)
            if next == Some('\u{005c}') {
                // If the next input code point is EOF, do nothing.
                if self.process.peek().is_none() {
                    continue;
                }
                // Otherwise, if the next input code point is a newline, consume it.
                else if let Some(&v) = self.process.peek()
                    && Self::is_newline(v)
                {
                    self.process.next();
                    continue;
                }
                // Otherwise, (the stream starts with a valid escape) consume an escaped code point and append the returned code point to the <string-token>’s value.
                else if let Some(&v) = self.process.peek()
                    && Self::is_valid_escape(Some('\u{005c}'), Some(v))
                {
                    out.push(self.consume_escaped_code_point());
                    continue;
                }
            }

            // anything else
            // Append the current input code point to the <string-token>’s value.
            if let Some(v) = next {
                out.push(v);
            }
        }
    }

    pub fn is_newline(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // U+000A LINE FEED.
        input == '\u{000a}'
    }

    pub fn peek_twin(&mut self) -> (Option<char>, Option<char>) {
        let peek = self.process.peek_amount(2);
        let first = peek[0];
        let second = peek[1];
        (first, second)
    }

    pub fn consume_escaped_code_point(&mut self) -> char {
        // https://www.w3.org/TR/css-syntax-3/#consume-escaped-code-point

        // Consume the next input code point.
        let next = self.process.next();

        // hex digit
        if let Some(v) = next
            && Self::is_hex_digit(v)
        {
            // Consume as many hex digits as possible, but no more than 5.
            let mut digits = String::new();
            digits.push(v);

            while let Some(&peek) = self.process.peek()
                && Self::is_hex_digit(peek)
                && digits.len() <= 6
            {
                digits.push(peek);
                self.process.next();
            }

            // If the next input code point is whitespace, consume it as well.
            if let Some(&peek) = self.process.peek()
                && Self::is_whitespace(peek)
            {
                self.process.next();
            }

            // Interpret the hex digits as a hexadecimal number.
            let parsed = u32::from_str_radix(&digits, 16).unwrap();

            // If this number is zero, or is for a surrogate, or is greater than the maximum allowed code point,
            if parsed == 0 || Self::is_surrogate(parsed) || parsed > Self::max_allowed_code_point()
            {
                // return U+FFFD REPLACEMENT CHARACTER (�).
                return '\u{fffd}';
            }

            // 	Otherwise, return the code point with that value.
            return char::from_u32(parsed).unwrap();
        }

        // EOF
        if next.is_none() {
            // This is a parse error. Return U+FFFD REPLACEMENT CHARACTER (�).
            self.parse_error();
            return '\u{fffd}';
        }

        // anything else
        // Return the current input code point.
        next.unwrap()
    }

    pub fn max_allowed_code_point() -> u32 {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // The greatest code point defined by Unicode: U+10FFFF.
        0x10ffff
    }

    pub fn is_surrogate(input: u32) -> bool {
        // https://infra.spec.whatwg.org/#surrogate
        // A surrogate is a leading surrogate or a trailing surrogate.
        Self::is_leading_surrogate(input) || Self::is_trailing_surrogate(input)
    }

    pub fn is_leading_surrogate(input: u32) -> bool {
        // https://infra.spec.whatwg.org/#surrogate
        // A leading surrogate is a code point that is in the range U+D800 to U+DBFF, inclusive.
        (0xd800..=0xdbff).contains(&input)
    }

    pub fn is_trailing_surrogate(input: u32) -> bool {
        // https://infra.spec.whatwg.org/#surrogate
        // A trailing surrogate is a code point that is in the range U+DC00 to U+DFFF, inclusive.
        (0xdc00..=0xdfff).contains(&input)
    }

    pub fn code_point_to_char(input: &str) -> char {
        char::from_u32(u32::from_str_radix(input, 16).unwrap()).unwrap()
    }

    pub fn is_digit(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A code point between U+0030 DIGIT ZERO (0) and U+0039 DIGIT NINE (9) inclusive.
        ('\u{0030}'..='\u{0039}').contains(&input)
    }

    pub fn is_hex_digit(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A digit, or a code point between U+0041 LATIN CAPITAL LETTER A (A) and U+0046 LATIN CAPITAL LETTER F (F) inclusive, or a code point between U+0061 LATIN SMALL LETTER A (a) and U+0066 LATIN SMALL LETTER F (f) inclusive.
        Self::is_digit(input)
            || ('\u{0041}'..='\u{0046}').contains(&input)
            || ('\u{0061}'..='\u{0066}').contains(&input)
    }

    pub fn is_valid_escape(first: Option<char>, second: Option<char>) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#starts-with-a-valid-escape

        // If the first code point is not U+005C REVERSE SOLIDUS (\), return false.
        if first != Some('\u{005c}') {
            return false;
        }

        //Otherwise, if the second code point is a newline, return false.
        if second == Some('\u{000a}') {
            return false;
        }

        //Otherwise, return true.
        true
    }

    pub fn consume_whitespace(&mut self) {
        while let Some(&v) = self.process.peek() {
            if !Self::is_whitespace(v) {
                break;
            }
            println!("consuming {v}");
            self.process.next();
        }
    }

    pub fn is_whitespace(input: char) -> bool {
        // https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions
        // A newline, U+0009 CHARACTER TABULATION, or U+0020 SPACE.
        input == '\u{000a}' || input == '\u{0009}' || input == '\u{0020}'
    }

    pub fn consume_comments(&mut self) {
        // https://www.w3.org/TR/css-syntax-3/#consume-comment
        // This section describes how to consume comments from a stream of code points. It returns nothing.
        // If the next two input code point are U+002F SOLIDUS (/) followed by a U+002A ASTERISK (*), consume them and all following code points up to and including the first U+002A ASTERISK (*) followed by a U+002F SOLIDUS (/), or up to an EOF code point. Return to the start of this step.
        let (mut first, mut second) = self.peek_twin();
        if !(first == Some('\u{002f}') && second == Some('\u{002a}')) {
            return;
        }

        self.process.next();
        self.process.next();

        loop {
            (first, second) = self.peek_twin();

            if first.is_none() || second.is_none() {
                // If the preceding paragraph ended by consuming an EOF code point, this is a parse error.
                self.parse_error();
                return;
            }

            if first == Some('\u{002a}') && second == Some('\u{002f}') {
                println!("i was true");
                self.process.next();
                self.process.next();
                self.consume_comments();
                // Return nothing.
                return;
            }

            self.process.next();
        }
    }

    pub fn parse_error(&mut self) {
        self.parse_error = true;
    }

    pub fn tokenize(&mut self) -> Vec<CSSToken> {
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
    LeftParenthesisToken,
    RightParenthesisToken,
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
    RightSquareBracketToken,
    LeftCurlyBracketToken,
    RightCurlyBracketToken,
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

impl<'a> Iterator for PutBackPeekMore<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.put_back {
            let returned = v;
            self.put_back = None;
            return Some(v);
        }
        self.peek_more.next()
    }
}

impl<'a> PutBackPeekMore<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            put_back: None,
            peek_more: input.chars().peekmore(),
        }
    }

    pub fn put_back(&mut self, input: char) {
        self.put_back = Some(input);
    }

    pub fn put_back_option(&mut self, input: Option<char>) {
        self.put_back = input;
    }

    pub fn peek(&mut self) -> Option<&char> {
        if self.put_back.is_some() {
            return self.put_back.as_ref();
        }
        self.peek_more.peek()
    }

    pub fn peek_amount(&mut self, amount: usize) -> Vec<Option<char>> {
        if self.put_back.is_some() {
            let mut out = vec![self.put_back];
            out.extend_from_slice(self.peek_more.peek_amount(amount - 1));
            return out;
        }
        self.peek_more.peek_amount(amount).to_vec()
    }

    pub fn peek_nth(&mut self, amount: usize) -> Option<&char> {
        if self.put_back.is_some() {
            return self.peek_more.peek_nth(amount - 1);
        }
        self.peek_more.peek_nth(amount)
    }
}
