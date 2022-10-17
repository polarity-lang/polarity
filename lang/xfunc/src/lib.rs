use syntax::ast;

mod matrix;
mod repr;

use repr::Represent;

pub fn as_matrix(prg: &ast::Prg) -> syntax::matrix::Prg {
    matrix::build(prg)
}

pub fn repr(prg: &syntax::matrix::Prg, name: &str) -> syntax::matrix::Repr {
    prg.map[name].repr
}

pub fn as_data(
    prg: &syntax::matrix::Prg,
    name: &str,
) -> (ast::Data, Vec<ast::Ctor>, Vec<ast::Def>) {
    prg.map[name].as_data()
}

pub fn as_codata(
    prg: &syntax::matrix::Prg,
    name: &str,
) -> (ast::Codata, Vec<ast::Dtor>, Vec<ast::Codef>) {
    prg.map[name].as_codata()
}
