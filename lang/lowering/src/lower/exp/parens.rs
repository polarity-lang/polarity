use parser::cst;

use crate::lower::Lower;

impl Lower for cst::exp::Parens {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut crate::Ctx) -> crate::LoweringResult<Self::Target> {
        let e = self.exp.lower(ctx)?;
        Ok(*e)
    }
}
