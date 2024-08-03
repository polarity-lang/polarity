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
    //
    #[regex(r"[a-zα-ωA-ZΑ-Ω𝔹ℕ𝕍∃∀×][a-zα-ωA-ZΑ-Ω0-9_]*['⁺⁻₀₁₂₃₄₅₆₇₈₉₊₋]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Literals
    //
    //
    #[regex(r"0|[1-9][0-9]*", |lex| BigUint::parse_bytes(lex.slice().as_ref(), 10).unwrap())]
    NumLit(BigUint),

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
