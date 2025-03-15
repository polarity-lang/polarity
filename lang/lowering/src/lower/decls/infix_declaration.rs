use parser::cst;

use super::super::*;

impl Lower for cst::decls::Infix {
    type Target = ast::Decl;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        todo!()
    }
}
