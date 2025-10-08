//! Erasure
//!
//! At the moment, erasure is purely syntactic and only affects parameter and argument lists.
//! Elaboration consults the functions in this module to determine whether a parameter or argument
//! should be marked as erased.
//!
//! At the moment, a parameter is erased if its type is ...
//!
//! * the type universe `Type`
//! * an annotated type universe `Type : Type`
//! * a hole that solves to `Type`
//!
//! An argument is erased if its corresponding parameter is erased.

/// Mark runtime-irrelevant parameters as erased
pub fn mark_erased_params(params: &mut polarity_lang_ast::Telescope) {
    for param in params.params.iter_mut() {
        param.erased = is_runtime_irrelevant(&param.typ);
    }
}

/// If this function return true on a term of type `typ`,
/// then it has no runtime relevance and the term can be erased.
pub fn is_runtime_irrelevant(typ: &polarity_lang_ast::Exp) -> bool {
    match typ {
        polarity_lang_ast::Exp::Variable(_) => false,
        polarity_lang_ast::Exp::TypCtor(_) => false,
        polarity_lang_ast::Exp::Call(_) => false,
        polarity_lang_ast::Exp::DotCall(_) => false,
        polarity_lang_ast::Exp::Anno(anno) => is_runtime_irrelevant(&anno.exp),
        polarity_lang_ast::Exp::TypeUniv(_) => true,
        polarity_lang_ast::Exp::LocalMatch(_) => false,
        polarity_lang_ast::Exp::LocalComatch(_) => false,
        polarity_lang_ast::Exp::Hole(hole) => {
            hole.solution.as_ref().map(|s| is_runtime_irrelevant(s)).unwrap_or(false)
        }
        polarity_lang_ast::Exp::LocalLet(_) => false,
    }
}

/// Mark runtime-irrelevant arguments as erased
///
/// We mark each argument as erased if the corresponding parameter is marked as erased.
pub fn mark_erased_args(params: &polarity_lang_ast::Telescope, args: &mut polarity_lang_ast::Args) {
    for (param, arg) in params.params.iter().zip(args.args.iter_mut()) {
        arg.set_erased(param.erased);
    }
}
