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
#[logos(skip r"\s*", skip r"//([^/\n\r]([^\n\r]*))?[\n\r]*", error = LexicalError)]
//                          ^^   ^^^^^^  ^^^^^^^  ^^^^^^^
//                          (1)   (2)      (3)     (4)
// Comments start with "//" (1).
// Then we have to exclude the possibility of a doc comment which starts with "///":
// If the line is not empty, then the next character must not contain "/" (2)
// And this character can be followed by any number of characters which don't end the line (3)
// And finally many newlines (5)
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
    #[token("implicit")]
    Implicit,
    #[token("use")]
    Use,
    #[token("infix")]
    Infix,

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
    // No = other numbers (includes subscripts and superscript numerals)
    // ' is contained in the category Po (Punctuation other)
    // _ is contained in the category Pc (Punctuation connector)
    #[regex(r"[\p{Ll}\p{Lu}][\p{Ll}\p{Lu}\p{No}0-9_']*", |lex| lex.slice().to_string())]
    Ident(String),

    // Operators
    //
    // We use the following unicode categories:
    // Pd = Dash Punctuation
    // Sm = math symbol
    #[regex(r"[\p{Pd}\p{Sm}][\p{Pd}\p{Sm}]*", |lex| lex.slice().to_string())]
    Operator(String),

    // Literals
    //
    //
    #[regex(r"0|[1-9][0-9]*", |lex| BigUint::parse_bytes(lex.slice().as_ref(), 10).unwrap())]
    NumLit(BigUint),
    /// The regexp is from `https://gist.github.com/cellularmitosis/6fd5fc2a65225364f72d3574abd9d5d5`
    /// We do not allow multi line strings.
    #[regex(r###""([^"\\]|\\.)*""###, |lex| {
        let slice = lex.slice();
        // Remove the surrounding quotation marks
        let inner = &slice[1..slice.len()-1];
        inner.to_string()
    })]
    StringLit(String),

    // DocComments
    //
    //
    #[regex(r"///[^\n\r]*[\n\r]*", |lex| lex.slice().to_string())]
    //        ^^^ ^^^^^^^ ^^^^^^
    //        (1)   (2)    (3)
    // Doc comments start with "///" (1),
    // followed by any number of non-line-break characters (2),
    // followed by any number of empty lines (3).
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

impl Iterator for Lexer<'_> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::{Lexer, Token};

    #[test]
    fn doc_comment_1() {
        let str = r###"/// hello"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::DocComment("/// hello".to_string()))
    }

    #[test]
    fn doc_comment_2() {
        let str = "//comment\n/// hello";
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::DocComment("/// hello".to_string()))
    }

    #[test]
    fn string_lit_simple() {
        let str = r###""hi""###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::StringLit("hi".to_string()))
    }

    #[test]
    fn string_lit_newline() {
        let str = r###""h\ni""###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::StringLit("h\\ni".to_string()))
    }

    #[test]
    fn string_lit_escaped_quote() {
        let str = r###""h\"i""###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::StringLit("h\\\"i".to_string()))
    }
}
