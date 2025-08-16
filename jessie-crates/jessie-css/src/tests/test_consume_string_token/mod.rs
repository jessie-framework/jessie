#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_consume_string_token_quotation_mark() {
        assert_eq!(
            Tokenizer::new(include_str!("./quotationmark.css")).tokenize(),
            vec![
                CSSToken::StringToken {
                    string: "this is a string".into()
                },
                CSSToken::WhitespaceToken,
                CSSToken::EOFToken
            ]
        )
    }
    #[test]
    fn test_consume_string_token_apostrophe() {
        assert_eq!(
            Tokenizer::new(include_str!("./apostrophe.css")).tokenize(),
            vec![
                CSSToken::StringToken {
                    string: "this is a string".into()
                },
                CSSToken::WhitespaceToken,
                CSSToken::EOFToken
            ]
        )
    }
    #[test]
    fn test_consume_string_token_escaping() {
        assert_eq!(
            Tokenizer::new(include_str!("./escaping.css")).tokenize(),
            vec![
                CSSToken::StringToken {
                    string: "this is a test and this is a cherry blossom: \u{1f338}".into()
                },
                CSSToken::WhitespaceToken,
                CSSToken::EOFToken
            ]
        )
    }
}
