use syntax::ust;

pub mod matrix;

pub fn as_matrix(prg: &ust::Prg) -> matrix::Prg {
    matrix::build(prg)
}

pub fn repr(prg: &matrix::Prg, name: &str) -> matrix::Repr {
    prg.map[name].repr
}

pub fn as_data(prg: &matrix::Prg, name: &str) -> (ust::Data, Vec<ust::Ctor>, Vec<ust::Def>) {
    prg.map[name].as_data()
}

pub fn as_codata(prg: &matrix::Prg, name: &str) -> (ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>) {
    prg.map[name].as_codata()
}
