use std::fmt;

use logos::{Logos, Span, SpannedIter};
use num_bigint::BigUint;

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalError {
    /// A generic lexer error
    InvalidToken(Option<Span>),

    /// Char literal that does not contain exactly one char
    InvalidCharLiteral(Span),

    /// Unknown or incomplete escape sequence
    InvalidEscapeSequence(Span),

    /// Unicode escape sequence that is syntactically incorrect
    MalformedUnicodeEscape(Span),

    /// Unicode literal that does not correspond to a valid codepoint
    InvalidUnicodeCodepoint(Span),

    /// An invalid hexadecimal number literal
    InvalidHexNumber(Span),
}

impl Default for LexicalError {
    fn default() -> Self {
        Self::InvalidToken(None)
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(error(LexicalError, callback = |lex| LexicalError::InvalidToken(Some(lex.span()))))]
#[logos(skip r"\s*", skip r"//([^/\n\r]([^\n\r]*))?[\n\r]*")]
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
    #[token("note")]
    Note,

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
    /// The regexp is from <https://gist.github.com/cellularmitosis/6fd5fc2a65225364f72d3574abd9d5d5>
    /// TODO: Maybe forbid multi-line strings or have a separate syntax?
    #[regex(r###""([^"\\]|\\.)*""###, |lex| StringLit::parse(lex.slice(), lex.span()))]
    StringLit(StringLit),
    #[regex(r###"'([^'\\]|\\.)*'"###, |lex| CharLit::parse(lex.slice(), lex.span()))]
    CharLit(CharLit),

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
        write!(f, "{self:?}")
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

#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    pub original: String,
    pub unescaped: String,
}

impl StringLit {
    /// Validate and unescape a string literal
    fn parse(literal: &str, span: Span) -> Result<Self, LexicalError> {
        let inner = &literal[1..literal.len() - 1];
        let mut chars = inner.chars();

        let mut unescaped = String::new();
        while let Some(mut ch) = chars.next() {
            if ch == '\\' {
                ch = unescape(&mut chars, span.clone())?;
            }

            unescaped.push(ch);
        }

        Ok(Self { original: inner.to_string(), unescaped })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharLit {
    pub original: String,
    pub unescaped: char,
}

impl CharLit {
    /// Validate and unescape a character literal
    fn parse(literal: &str, span: Span) -> Result<Self, LexicalError> {
        let inner = &literal[1..literal.len() - 1];
        let mut chars = inner.chars();

        let Some(mut unescaped) = chars.next() else {
            return Err(LexicalError::InvalidCharLiteral(span));
        };

        if unescaped == '\\' {
            unescaped = unescape(&mut chars, span.clone())?;
        }

        if chars.next().is_some() {
            return Err(LexicalError::InvalidCharLiteral(span));
        }

        Ok(Self { original: inner.to_string(), unescaped })
    }
}

/// Unescape a single escape sequence in a character iterator and consume it
fn unescape(seq: &mut std::str::Chars, span: Span) -> Result<char, LexicalError> {
    let Some(escape_code) = seq.next() else {
        return Err(LexicalError::InvalidEscapeSequence(span));
    };

    let unescaped_char = match escape_code {
        // control characters
        'n' => '\n',
        'r' => '\r',
        't' => '\t',

        // quotes
        '"' => '"',
        '\'' => '\'',

        // backslash
        '\\' => '\\',

        // unicode codepoints
        'u' => {
            if seq.next() != Some('{') {
                return Err(LexicalError::MalformedUnicodeEscape(span));
            }

            let mut hex_str = String::new();
            for hex_digit in seq.by_ref() {
                hex_str.push(hex_digit);
                if hex_digit == '}' {
                    break;
                }
            }

            if hex_str.pop() != Some('}') {
                return Err(LexicalError::MalformedUnicodeEscape(span));
            }

            // check hex code length (between 1 and 6)
            let hex_length = hex_str.chars().count();
            if !(1..=6).contains(&hex_length) {
                return Err(LexicalError::MalformedUnicodeEscape(span));
            }

            // parse to numeral
            let hex = u32::from_str_radix(&hex_str, 16)
                .map_err(|_| LexicalError::InvalidHexNumber(span.clone()))?;

            // convert to character
            char::from_u32(hex).ok_or(LexicalError::InvalidUnicodeCodepoint(span))?
        }

        _ => return Err(LexicalError::InvalidEscapeSequence(span)),
    };

    Ok(unescaped_char)
}

#[cfg(test)]
mod lexer_tests {
    use super::{CharLit, Lexer, LexicalError, StringLit, Token};

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

    fn assert_eq_string_lit(str: &str, unescaped: &str) {
        let without_quotes = &str[1..str.len() - 1];
        let mut lexer = Lexer::new(str);

        assert_eq!(
            lexer.next().unwrap().unwrap().1,
            Token::StringLit(StringLit {
                original: without_quotes.to_string(),
                unescaped: unescaped.to_string(),
            })
        )
    }

    fn assert_eq_char_lit(str: &str, unescaped: char) {
        let without_quotes = &str[1..str.len() - 1];
        let mut lexer = Lexer::new(str);

        assert_eq!(
            lexer.next().unwrap().unwrap().1,
            Token::CharLit(CharLit { original: without_quotes.to_string(), unescaped: unescaped })
        )
    }

    #[test]
    fn string_lit_simple() {
        let str = r###""hi""###;
        assert_eq_string_lit(str, "hi");
    }

    #[test]
    fn string_lit_newline() {
        let str = r###""h\ni""###;
        assert_eq_string_lit(str, "h\ni");
    }

    #[test]
    fn string_lit_escaped_quote() {
        let str = r###""h\"i""###;
        assert_eq_string_lit(str, "h\"i");
    }

    #[test]
    fn char_lit_simple() {
        let str = r###"'h'"###;
        assert_eq_char_lit(str, 'h');
    }

    #[test]
    fn char_lit_unicode() {
        let str = r###"'π'"###;
        assert_eq_char_lit(str, 'π');
    }

    #[test]
    fn char_lit_newline() {
        let str = r###"'\n'"###;
        assert_eq_char_lit(str, '\n');
    }

    #[test]
    fn char_lit_escaped_quote() {
        let str = r###"'\''"###;
        assert_eq_char_lit(str, '\'');
    }

    #[test]
    fn escape_control_chars() {
        let str = r###""A\nB\rC\t\\""###;
        assert_eq_string_lit(str, "A\nB\rC\t\\");
    }

    #[test]
    fn escape_unicode_literals() {
        let str = r###""\u{03BB} \u{03bb}""###;
        assert_eq_string_lit(str, "\u{03bb} \u{03bb}");
    }

    #[test]
    fn escape_unicode_emoji() {
        let str = r###"'\u{1f60e}'"###;
        assert_eq_char_lit(str, '😎');
    }

    #[test]
    fn escape_unknown() {
        let str = r###""\x""###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::InvalidEscapeSequence(0..4)))
    }

    #[test]
    fn char_too_long() {
        let str = r###"'aa'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::InvalidCharLiteral(0..4)))
    }

    #[test]
    fn char_empty() {
        let str = r###"''"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::InvalidCharLiteral(0..2)))
    }

    #[test]
    fn invalid_unicode_escape_1() {
        let str = r###"'\u1234'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::MalformedUnicodeEscape(0..8)))
    }

    #[test]
    fn invalid_unicode_escape_2() {
        let str = r###"'\u{1234'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::MalformedUnicodeEscape(0..9)))
    }

    #[test]
    fn invalid_unicode_escape_3() {
        let str = r###"'\u1234}'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::MalformedUnicodeEscape(0..9)))
    }

    #[test]
    fn invalid_unicode_escape_4() {
        let str = r###"'\u{}'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::MalformedUnicodeEscape(0..6)))
    }

    #[test]
    fn invalid_unicode_escape_5() {
        let str = r###"'\u{1234567}'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::MalformedUnicodeEscape(0..13)))
    }

    #[test]
    fn invalid_unicode_escape_surrogate_1() {
        let str = r###"'\u{D800}'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::InvalidUnicodeCodepoint(0..10)))
    }

    #[test]
    fn invalid_unicode_escape_surrogate_2() {
        let str = r###"'\u{DFFF}'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::InvalidUnicodeCodepoint(0..10)))
    }

    #[test]
    fn invalid_unicode_escape_too_big() {
        let str = r###"'\u{FFFFFF}'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::InvalidUnicodeCodepoint(0..12)))
    }

    #[test]
    fn invalid_unicode_escape_bad_hex() {
        let str = r###"'\u{123g4}'"###;
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.next().unwrap(), Err(LexicalError::InvalidHexNumber(0..11)))
    }
}
