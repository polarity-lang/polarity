use polarity_lang_ast::Note;

use super::CheckToplevel;

impl CheckToplevel for Note {
    fn check_wf(&self, _ctx: &mut crate::typechecker::ctx::Ctx) -> crate::result::TcResult<Self> {
        Ok(self.clone())
    }
}
