use ast::Infix;

use super::CheckToplevel;

impl CheckToplevel for Infix {
    fn check_wf(&self, _ctx: &mut crate::typechecker::ctx::Ctx) -> crate::result::TcResult<Self> {
        Ok(self.clone())
    }
}
