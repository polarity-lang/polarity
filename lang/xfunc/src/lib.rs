use syntax::generic;

pub mod matrix;
pub mod result;

pub fn as_matrix(prg: &generic::Prg) -> Result<matrix::Prg, crate::result::XfuncError> {
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
) -> Result<(generic::Data, Vec<generic::Ctor>, Vec<generic::Def>), crate::result::XfuncError> {
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
    name: &str,
) -> Result<(generic::Codata, Vec<generic::Dtor>, Vec<generic::Codef>), crate::result::XfuncError> {
    prg.map
        .get(name)
        .ok_or_else(|| crate::result::XfuncError::Impossible {
            message: format!("Could not resolve {name}"),
            span: None,
        })
        .map(|x| x.as_codata())
}
