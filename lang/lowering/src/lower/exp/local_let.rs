use parser::cst;

use crate::lower::Lower;

impl Lower for cst::exp::LocalLet {
    type Target = ast::Exp;

    fn lower(&self, _ctx: &mut crate::Ctx) -> crate::LoweringResult<Self::Target> {
        todo!()
    }
}
