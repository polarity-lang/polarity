use crate::ir::Variable;

pub trait Rename {
    fn rename(&mut self, ctx: &mut RenameCtx);
}

pub struct RenameCtx(pub Vec<(Variable, Variable)>);
