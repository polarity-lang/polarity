use std::fmt;

use logos::{Logos, SpannedIter};
use num_bigint::BigUint;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    #[default]
    InvalidToken,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"\s*", skip r"--(([^ \n\r]| [^\|\n\r])[^\n\r]*)?[\n\r]*", error = LexicalError)]
pub enum Token {
    // Keywords
    //
    //
    #[token("data")]
    Data,
    #[token("codata")]
    Codata,
    #[token("def")]
    Def,
    #[token("codef")]
    Codef,
    #[token("let")]
    Let,
    #[token("match")]
    Match,
    #[token("as")]
    As,
    #[token("comatch")]
    Comatch,
    #[token("absurd")]
    Absurd,
    #[token("Type")]
    Type,
    #[token("implicit")]
    Implicit,
    #[token("use")]
    Use,

    // Parens, Braces and Brackets
    //
    //
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    // Symbols
    //
    //
    #[token(";")]
    Semicolon,
    #[token(":=")]
    ColonEq,
    #[token("=>")]
    DoubleRightArrow,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("?")]
    QuestionMark,
    #[token("->")]
    RightArrow,
    #[token("\\")]
    Backslash,
    #[token("#")]
    Hash,
    #[token("_")]
    Underscore,

    // Identifiers
    //
    // We use the following unicode categories:
    // Ll = lowercase letter
    // Lu = uppercase letter
    // Sm = math symbol
    // No = other numbers (includes subscripts and superscript numerals)
    // ' is contained in the category Po (Punctuation other)
    // _ is contained in the category Pc (Punctuation connector)
    #[regex(r"[\p{Ll}\p{Lu}\p{Sm}][\p{Ll}\p{Lu}\p{Sm}\p{No}0-9_']*", |lex| lex.slice().to_string())]
    Ident(String),

    // Literals
    //
    //
    #[regex(r"0|[1-9][0-9]*", |lex| BigUint::parse_bytes(lex.slice().as_ref(), 10).unwrap())]
    NumLit(BigUint),
    /// The regexp is from `https://gist.github.com/cellularmitosis/6fd5fc2a65225364f72d3574abd9d5d5`
    /// We do not allow multi line strings.
    #[regex(r###""([^"\\]|\\.)*""###, |lex| lex.slice().to_string())]
    StringLit(String),

    // DocComments
    //
    //
    #[regex(r"-- \|[^\n\r]*[\n\r]*", |lex| lex.slice().to_string())]
    DocComment(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        // the Token::lexer() method is provided by the Logos trait
        Self { token_stream: Token::lexer(input).spanned() }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}

#[cfg(test)]
mod lexer_tests {
    use crate::lexer::{Lexer, Token};

    #[test]
    fn string_lit_simple() {
        let str = r###""hi""###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::StringLit(str.to_string()))
    }

    #[test]
    fn string_lit_newline() {
        let str = r###""h\ni""###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::StringLit(str.to_string()))
    }

    #[test]
    fn string_lit_escaped_quote() {
        let str = r###""h\"i""###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::StringLit(str.to_string()))
    }
}
