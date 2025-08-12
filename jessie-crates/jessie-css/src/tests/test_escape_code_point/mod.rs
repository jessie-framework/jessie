#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_escape_code_point() {
        assert_eq!(
            Tokenizer::new("1234").consume_escaped_code_point(),
            '\u{1234}'
        )
    }
}
