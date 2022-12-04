// use renaming::Rename;
use syntax::common::*;
use syntax::tst;

// mod lift;

// pub use lift::LiftResult;

/// Result of lifting
pub struct LiftResult {
    /// The resulting program
    pub prg: tst::Prg,
    /// List of top-level declarations that have been modified in the lifting process
    pub modified_decls: data::HashSet<Ident>,
}

/// Lift local (co)matches for `name` in `prg` to top-level (co)definitions
pub fn lift(_prg: tst::Prg, _name: &str) -> LiftResult {
    // FIXME: Reimplement
    todo!()
    // let prg = prg.rename();
    // lift::Lift::new(name.to_owned()).run(prg)
}

/// Inline lifted (co)definitions for `name` in `prg` to local (co)matches
pub fn inline(_prg: tst::Prg, _name: &str) -> tst::Prg {
    // TODO: Implement inlining
    unimplemented!()
}
