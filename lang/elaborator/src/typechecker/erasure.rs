/// Mark parameters as erased where applicable
pub fn mark_erased_params(params: &mut ast::Telescope) {
    for param in params.params.iter_mut() {
        param.erased = is_erased_type(&param.typ);
    }
}

/// Whether a term of type `typ` can be erased.
pub fn is_erased_type(typ: &ast::Exp) -> bool {
    match typ {
        ast::Exp::Variable(_) => false,
        ast::Exp::TypCtor(_) => false,
        ast::Exp::Call(_) => false,
        ast::Exp::DotCall(_) => false,
        ast::Exp::Anno(anno) => is_erased_type(&anno.exp),
        ast::Exp::TypeUniv(_) => true,
        ast::Exp::LocalMatch(_) => false,
        ast::Exp::LocalComatch(_) => false,
        ast::Exp::Hole(hole) => hole.solution.as_ref().map(|s| is_erased_type(s)).unwrap_or(false),
    }
}

pub fn mark_erased_args(params: &ast::Telescope, args: &mut ast::Args) {
    for (param, arg) in params.params.iter().zip(args.args.iter_mut()) {
        arg.set_erased(param.erased);
    }
}
