use syntax::ust;

mod matrix;
mod repr;

use repr::Represent;

pub fn as_matrix(prg: &ust::Prg) -> syntax::matrix::Prg {
    matrix::build(prg)
}

pub fn repr(prg: &syntax::matrix::Prg, name: &str) -> syntax::matrix::Repr {
    prg.map[name].repr
}

pub fn as_data(
    prg: &syntax::matrix::Prg,
    name: &str,
) -> (ust::Data, Vec<ust::Ctor>, Vec<ust::Def>) {
    prg.map[name].as_data()
}

pub fn as_codata(
    prg: &syntax::matrix::Prg,
    name: &str,
) -> (ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>) {
    prg.map[name].as_codata()
}
