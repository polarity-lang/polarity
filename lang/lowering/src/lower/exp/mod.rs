use ast::HasSpan;
use ast::ctx::BindContext;
use ast::ctx::values::Binder;
use miette_util::ToMiette;
use parser::cst;
use parser::cst::exp::BindingSite;

use super::Lower;
use crate::ctx::*;
use crate::result::*;

mod anno;
mod args;
mod binop;
mod call;
mod dot_call;
mod hole;
mod lam;
mod local_comatch;
mod local_let;
mod local_match;
mod nat_lit;
mod parens;

impl Lower for cst::exp::Exp {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        match self {
            cst::exp::Exp::Call(e) => e.lower(ctx),
            cst::exp::Exp::DotCall(e) => e.lower(ctx),
            cst::exp::Exp::Anno(e) => e.lower(ctx),
            cst::exp::Exp::LocalMatch(e) => e.lower(ctx),
            cst::exp::Exp::LocalComatch(e) => e.lower(ctx),
            cst::exp::Exp::Hole(e) => e.lower(ctx),
            cst::exp::Exp::NatLit(e) => e.lower(ctx),
            cst::exp::Exp::BinOp(e) => e.lower(ctx),
            cst::exp::Exp::Lam(e) => e.lower(ctx),
            cst::exp::Exp::LocalLet(e) => e.lower(ctx),
            cst::exp::Exp::Parens(e) => e.lower(ctx),
        }
    }
}

fn lower_telescope_inst<T, F: FnOnce(&mut Ctx, ast::TelescopeInst) -> LoweringResult<T>>(
    tel_inst: &[cst::exp::BindingSite],
    ctx: &mut Ctx,
    f: F,
) -> LoweringResult<T> {
    let tel_inst = tel_inst.iter().map(|bs| bs.lower(ctx)).collect::<Result<Vec<_>, _>>()?;
    ctx.bind_fold_failable(
        tel_inst.into_iter(),
        vec![],
        |_ctx, params_out, name| -> LoweringResult<Binder<()>> {
            let param_out =
                ast::ParamInst { span: name.span(), name: name.clone(), typ: None, erased: false };
            params_out.push(param_out);
            Ok(Binder { name, content: () })
        },
        |ctx, params| f(ctx, ast::TelescopeInst { params }),
    )?
}

impl Lower for cst::exp::BindingSite {
    type Target = ast::VarBind;

    fn lower(&self, _ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        match self {
            BindingSite::Var { span, name } => {
                if name.id == "Type" {
                    Err(LoweringError::TypeUnivIdentifier { span: span.to_miette() }.into())
                } else {
                    Ok(ast::VarBind::Var { span: Some(*span), id: name.id.clone() })
                }
            }
            BindingSite::Wildcard { span } => Ok(ast::VarBind::Wildcard { span: Some(*span) }),
        }
    }
}

impl Lower for cst::exp::Motive {
    type Target = ast::Motive;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Motive { span, param, ret_typ } = self;

        let name = param.lower(ctx)?;

        Ok(ast::Motive {
            span: Some(*span),
            param: ast::ParamInst {
                span: name.span(),
                name: name.clone(),
                typ: None,
                erased: false,
            },
            ret_typ: ctx.bind_single(name, |ctx| ret_typ.lower(ctx))?,
        })
    }
}
