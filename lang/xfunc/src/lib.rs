use syntax::ust;

pub mod matrix;

pub fn as_matrix(prg: &ust::Prg) -> matrix::Prg {
    matrix::build(prg)
}

pub fn repr(prg: &matrix::Prg, name: &str) -> Option<matrix::Repr> {
    prg.map.get(name).map(|x| x.repr)
}

pub fn as_data(
    prg: &matrix::Prg,
    name: &str,
) -> Option<(ust::Data, Vec<ust::Ctor>, Vec<ust::Def>)> {
    prg.map.get(name).map(|x| x.as_data())
}

pub fn as_codata(
    prg: &matrix::Prg,
    name: &str,
) -> Option<(ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>)> {
    prg.map.get(name).map(|x| x.as_codata())
}
