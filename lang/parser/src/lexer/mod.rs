use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
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
    #[token("..absurd")]
    DotsAbsurd,
    #[token("Type")]
    Type,

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

    // Names
    //
    //
    #[regex(r"[A-ZΑ-Ω𝔹ℕ𝕍∃∀×][a-zα-ωA-ZΑ-Ω0-9_]*['⁺⁻₀₁₂₃₄₅₆₇₈₉₊₋]*")]
    UpperCaseName,
    #[regex(r"[a-zα-ω][a-zα-ωA-ZΑ-Ω0-9_]*['⁺⁻₀₁₂₃₄₅₆₇₈₉₊₋]*")]
    LowerCaseName,

    // Literals
    //
    //
    #[regex(r"0|[1-9][0-9]*")]
    NumLit,

    // Comments and DocComments
    //
    //
    #[regex(r"--(([^ \n\r]| [^\|\n\r])[^\n\r]*)?[\n\r]*")]
    Comment,
    #[regex(r"-- \|[^\n\r]*[\n\r]*")]
    DocComment,

    // Whitespace
    //
    //
    #[regex(r"\s*")]
    Whitespace,
}