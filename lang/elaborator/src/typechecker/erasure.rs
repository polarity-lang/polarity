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
pub fn mark_erased_params(params: &mut ast::Telescope) {
    for param in params.params.iter_mut() {
        param.erased = is_runtime_irrelevant(&param.typ);
    }
}

/// If this function return true on a term of type `typ`,
/// then it has no runtime relevance and the term can be erased.
pub fn is_runtime_irrelevant(typ: &ast::Exp) -> bool {
    match typ {
        ast::Exp::Variable(_) => false,
        ast::Exp::TypCtor(_) => false,
        ast::Exp::Call(_) => false,
        ast::Exp::DotCall(_) => false,
        ast::Exp::Anno(anno) => is_runtime_irrelevant(&anno.exp),
        ast::Exp::TypeUniv(_) => true,
        ast::Exp::LocalMatch(_) => false,
        ast::Exp::LocalComatch(_) => false,
        ast::Exp::Hole(hole) => {
            hole.solution.as_ref().map(|s| is_runtime_irrelevant(s)).unwrap_or(false)
        }
        ast::Exp::LocalLet(_) => false,
    }
}

/// Mark runtime-irrelevant arguments as erased
///
/// We mark each argument as erased if the corresponding parameter is marked as erased.
pub fn mark_erased_args(params: &ast::Telescope, args: &mut ast::Args) {
    for (param, arg) in params.params.iter().zip(args.args.iter_mut()) {
        arg.set_erased(param.erased);
    }
}
