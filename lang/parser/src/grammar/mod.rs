use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[allow(dead_code)]
    pub cst, "/grammar/cst.rs"
);
