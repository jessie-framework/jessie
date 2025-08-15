#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_consume_string_token() {
        assert_eq!(
            Tokenizer::new(include_str!("./test.css")).tokenize(),
            vec![
                CSSToken::StringToken {
                    string: "this is a string".into()
                },
                CSSToken::WhitespaceToken,
                CSSToken::EOFToken
            ]
        )
    }
}
