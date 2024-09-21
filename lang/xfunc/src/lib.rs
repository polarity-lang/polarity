use ast::Ident;

pub mod matrix;
pub mod result;

pub fn as_matrix(prg: &ast::Module) -> Result<matrix::Prg, crate::result::XfuncError> {
    matrix::build(prg)
}

pub fn repr(prg: &matrix::Prg, name: &Ident) -> Result<matrix::Repr, crate::result::XfuncError> {
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
    name: &Ident,
) -> Result<(ast::Data, Vec<ast::Def>), crate::result::XfuncError> {
    prg.map
        .get(name)
        .ok_or_else(|| crate::result::XfuncError::Impossible {
            message: format!("Could not resolve {name}"),
            span: None,
        })
        .map(|x| x.as_data())
}

pub fn as_codata(
    prg: &matrix::Prg,
    name: &Ident,
) -> Result<(ast::Codata, Vec<ast::Codef>), crate::result::XfuncError> {
    prg.map
        .get(name)
        .ok_or_else(|| crate::result::XfuncError::Impossible {
            message: format!("Could not resolve {name}"),
            span: None,
        })
        .map(|x| x.as_codata())
}
