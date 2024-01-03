use lalrpop_util::lalrpop_mod;

mod util;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(unused_imports)]
    #[allow(dead_code)]
    pub cst, "/grammar/cst.rs"
);
