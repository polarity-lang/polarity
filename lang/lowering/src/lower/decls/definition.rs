use ast::IdBind;
use parser::cst::{self};

use super::super::*;
use super::lower_self_param;
use super::lower_telescope;

impl Lower for cst::decls::Def {
    type Target = ast::Def;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        log::trace!("Lowering definition: {}", self.name.id);

        let cst::decls::Def { span, doc, name, attr, params, scrutinee, ret_typ, cases } = self;

        let self_param: cst::decls::SelfParam = scrutinee.clone().into();

        lower_telescope(params, ctx, |ctx, params| {
            let cases = cases.lower(ctx)?;
            lower_self_param(&self_param, ctx, |ctx, self_param| {
                Ok(ast::Def {
                    span: Some(*span),
                    doc: doc.lower(ctx)?,
                    name: IdBind { span: Some(name.span), id: name.id.clone() },
                    attr: attr.lower(ctx)?,
                    params,
                    self_param,
                    ret_typ: ret_typ.lower(ctx)?,
                    cases,
                })
            })
        })
    }
}
