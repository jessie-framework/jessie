#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_string_to_number() {
        assert_eq!(Tokenizer::string_to_number("+1e12".into()), 1000000000000.);
        assert_eq!(Tokenizer::string_to_number("1e12".into()), 1000000000000.);
        assert_eq!(Tokenizer::string_to_number("-1e12".into()), -1000000000000.);
        assert_eq!(Tokenizer::string_to_number("1e-1".into()), 0.1);
        assert_eq!(Tokenizer::string_to_number("1.5".into()), 1.5);
        assert_eq!(Tokenizer::string_to_number("+3.1".into()), 3.1);
        assert_eq!(Tokenizer::string_to_number("-420".into()), -420.);
        assert_eq!(Tokenizer::string_to_number("29312345".into()), 29312345.);
    }
}
