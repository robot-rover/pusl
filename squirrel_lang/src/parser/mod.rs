mod tokens;

use logos::Logos;
use tokens::Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_token_test() {
        println!("Checking Basic Tokens");
        let result = Token::lexer("base").map(Result::unwrap).collect::<Vec<_>>();
        assert_eq!(result, vec![Token::Base]);

        let result = Token::lexer("true").map(Result::unwrap).collect::<Vec<_>>();
        assert_eq!(result, vec![Token::Boolean(true)]);

        // Strings
        println!("Checking Strings");
        let result = Token::lexer(r#""Hello World""#)
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![Token::String("Hello World".into())]);

        println!("Checking String Escapes");
        let result = Token::lexer(r#""Hello\"World""#)
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![Token::String(r#"Hello"World"#.into())]);

        let result = Token::lexer(r#""Hello\nWorld""#)
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![Token::String("Hello\nWorld".into())]);

        // TODO: This should error when its not a verbatim string
        let result = Token::lexer("\"Hello\nWorld\"")
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![Token::String("Hello\nWorld".into())]);

        println!("Checking Verbatim Strings");
        let result = Token::lexer("@\"Hello\\nWorld\nNewline\"")
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert_eq!(result, vec![Token::String("Hello\\nWorld\nNewline".into())]);
    }
}
