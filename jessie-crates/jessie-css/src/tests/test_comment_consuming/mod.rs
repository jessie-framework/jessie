#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_comment_consuming() {
        assert_eq!(
            Tokenizer::new(include_str!("./test.css")).tokenize(),
            Ok(vec![CSSToken::WhitespaceToken, CSSToken::EOFToken])
        )
    }
}
