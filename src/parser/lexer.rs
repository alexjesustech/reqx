// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Lexer for .reqx DSL expressions

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    // Literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    String(String),

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"true|false", |lex| lex.slice() == "true")]
    Boolean(bool),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Operators
    #[token("=")]
    Equals,

    #[token("!=")]
    NotEquals,

    #[token("<")]
    LessThan,

    #[token(">")]
    GreaterThan,

    #[token("<=")]
    LessOrEqual,

    #[token(">=")]
    GreaterOrEqual,

    // Delimiters
    #[token(".")]
    Dot,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("{{")]
    VariableStart,

    #[token("}}")]
    VariableEnd,

    #[token("|")]
    Pipe,

    // Keywords
    #[token("exists")]
    Exists,

    #[token("is_array")]
    IsArray,

    #[token("is_number")]
    IsNumber,

    #[token("is_string")]
    IsString,

    #[token("is_uuid")]
    IsUuid,

    #[token("is_iso8601")]
    IsIso8601,

    #[token("contains")]
    Contains,

    #[token("matches")]
    Matches,

    #[token("length")]
    Length,

    #[token("res")]
    Res,

    #[token("body")]
    Body,

    #[token("headers")]
    Headers,

    #[token("status")]
    Status,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    Token::lexer(input).filter_map(|t| t.ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("status = 200");
        assert!(tokens.contains(&Token::Status));
        assert!(tokens.contains(&Token::Equals));
        assert!(tokens.contains(&Token::Integer(200)));
    }

    #[test]
    fn test_tokenize_jsonpath() {
        let tokens = tokenize("body.data[0].id");
        assert!(tokens.contains(&Token::Body));
        assert!(tokens.contains(&Token::Dot));
        assert!(tokens.contains(&Token::LeftBracket));
        assert!(tokens.contains(&Token::Integer(0)));
        assert!(tokens.contains(&Token::RightBracket));
    }

    #[test]
    fn test_tokenize_variable() {
        let tokens = tokenize("{{user_id}}");
        assert!(tokens.contains(&Token::VariableStart));
        assert!(tokens.contains(&Token::Identifier("user_id".to_string())));
        assert!(tokens.contains(&Token::VariableEnd));
    }
}
