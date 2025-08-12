#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_consume_string_token() {
        assert_eq!(
            Tokenizer::new(include_str!("./test.css")).consume_string_token('"'),
            Ok(CSSToken::StringToken {
                string: "this is a test".into()
            })
        )
    }
}
