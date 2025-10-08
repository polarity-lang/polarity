pub mod matrix;
pub mod result;

pub fn as_matrix(
    prg: &polarity_lang_ast::Module,
) -> Result<matrix::Prg, crate::result::XfuncError> {
    matrix::build(prg)
}

pub fn repr(prg: &matrix::Prg, name: &str) -> Result<matrix::Repr, crate::result::XfuncError> {
    prg.map
        .get(name)
        .ok_or_else(|| crate::result::XfuncError::Impossible {
            message: format!("Could not resolve {name}"),
            span: None,
        })
        .map(|x| x.repr)
}

pub fn as_data(
    prg: &matrix::Prg,
    name: &str,
) -> Result<(polarity_lang_ast::Data, Vec<polarity_lang_ast::Def>), crate::result::XfuncError> {
    prg.map
        .get(name)
        .ok_or_else(|| crate::result::XfuncError::Impossible {
            message: format!("Could not resolve {name}"),
            span: None,
        })
        .map(|x| x.as_data(&prg.uri))
}

pub fn as_codata(
    prg: &matrix::Prg,
    name: &str,
) -> Result<(polarity_lang_ast::Codata, Vec<polarity_lang_ast::Codef>), crate::result::XfuncError> {
    prg.map
        .get(name)
        .ok_or_else(|| crate::result::XfuncError::Impossible {
            message: format!("Could not resolve {name}"),
            span: None,
        })
        .map(|x| x.as_codata(&prg.uri))
}
