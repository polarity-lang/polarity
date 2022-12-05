use renaming::Rename;
use syntax::wst;

mod lift;

pub use lift::LiftResult;

/// Lift local (co)matches for `name` in `prg` to top-level (co)definitions
pub fn lift(prg: wst::Prg, name: &str) -> LiftResult {
    let prg = prg.rename();
    lift::Lift::new(name.to_owned()).run(prg)
}

/// Inline lifted (co)definitions for `name` in `prg` to local (co)matches
pub fn inline(_prg: wst::Prg, _name: &str) -> wst::Prg {
    // TODO: Implement inlining
    unimplemented!()
}
