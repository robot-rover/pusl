mod tokens;

use logos::Logos;
use tokens::Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_token_test() {
        println!("Checking Basic Tokens");
        let result = Token::lexer("base").collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Base)], result);

        let result = Token::lexer("true").collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Boolean(true))], result);

        // Strings
        println!("Checking Strings");
        let result = Token::lexer(r#""Hello World""#)
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello World".into()))], result);

        println!("Checking String Escapes");
        let result = Token::lexer(r#""Hello\"World""#)
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String(r#"Hello"World"#.into()))], result);

        let result = Token::lexer(r#""Hello\nWorld""#)
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello\nWorld".into()))], result);

        // TODO: This should error when its not a verbatim string
        let result = Token::lexer("\"Hello\nWorld\"")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello\nWorld".into()))], result);

        println!("Checking Verbatim Strings");
        let result = Token::lexer("@\"Hello\\nWorld\nNewline\"")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::String("Hello\\nWorld\nNewline".into()))], result);

        println!("Checking Comments");
        let result = Token::lexer("// This is a comment")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Comment(" This is a comment".into()))], result);

        let result = Token::lexer("# This is a comment")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Comment(" This is a comment".into()))], result);

        let result = Token::lexer("  /* This is a\n\n comment */  ")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Comment(" This is a\n\n comment ".into()))], result);

        println!("Checkint Integer Literals");
        let result = Token::lexer("123")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Integer(123))], result);

        let result = Token::lexer("0123")
            .collect::<Vec<_>>();
        assert_eq!(vec![Ok(Token::Integer(1*8*8 + 2*8 + 3))], result);

        let result = Token::lexer("0x123")
            .collect::<Result<Vec<_>, _>>();
        assert_eq!(Ok(vec![Token::Integer(0x123)]), result);

        let result = Token::lexer("'a'")
            .collect::<Result<Vec<_>, _>>();
        assert_eq!(Ok(vec![Token::Integer(97)]), result);

        println!("Checking Float Literals");
        let result = Token::lexer("7.")
            .collect::<Result<Vec<_>, _>>();
        assert_eq!(Ok(vec![Token::Number(7.)]), result);

        let result = Token::lexer("4.0")
            .collect::<Result<Vec<_>, _>>();
        assert_eq!(Ok(vec![Token::Number(4.)]), result);

        let result = Token::lexer("4.e2")
            .collect::<Result<Vec<_>, _>>();
        assert_eq!(Ok(vec![Token::Number(4e2)]), result);

        let result = Token::lexer("4.e-2")
            .collect::<Result<Vec<_>, _>>();
        assert_eq!(Ok(vec![Token::Number(4e-2)]), result);
    }
}
