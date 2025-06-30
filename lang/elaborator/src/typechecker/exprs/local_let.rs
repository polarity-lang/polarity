//! Bidirectional type checker

use ast::ctx::BindContext;
use ast::ctx::values::Binder;
use ast::ctx::values::Binding;
use ast::ctx::values::BoundValue;
use ast::*;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::result::TcResult;

use super::super::ctx::*;
use super::CheckInfer;
use super::ExpectType;

impl CheckInfer for LocalLet {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let LocalLet { span, name, typ, bound, body, inferred_type: _ } = self;

        let (typ, typ_nf, bound) = match typ {
            Some(typ) => {
                let typ = typ.check(ctx, &TypeUniv::new().into())?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;

                let bound = bound.check(ctx, &typ_nf)?;

                (Some(typ), typ_nf, bound)
            }
            None => {
                let bound = bound.infer(ctx)?;
                let typ = bound.expect_typ()?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;

                (None, typ_nf, bound)
            }
        };

        let elem = Binder {
            name: name.clone(),
            content: Binding {
                typ: typ_nf,
                val: Some(BoundValue::LetBinding { val: bound.clone() }),
            },
        };

        // We need to shift the binder type here because we treat it as a 1-element telescope
        let body = ctx.bind_single(shift_and_clone(&elem, (1, 0)), |ctx| body.check(ctx, t))?;
        let inferred_type = body.expect_typ()?;

        Ok(LocalLet {
            span: *span,
            name: name.clone(),
            typ,
            bound,
            body,
            inferred_type: Some(inferred_type),
        })
    }

    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let LocalLet { span, name, typ, bound, body, inferred_type: _ } = self;

        let (typ, typ_nf, bound) = match typ {
            Some(typ) => {
                let typ = typ.check(ctx, &TypeUniv::new().into())?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;

                let bound = bound.check(ctx, &typ_nf)?;

                (Some(typ), typ_nf, bound)
            }
            None => {
                let bound = bound.infer(ctx)?;
                let typ = bound.expect_typ()?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;

                (None, typ_nf, bound)
            }
        };

        let elem = Binder {
            name: name.clone(),
            content: Binding {
                typ: typ_nf,
                val: Some(BoundValue::LetBinding { val: bound.clone() }),
            },
        };

        // We need to shift the binder type here because we treat it as a 1-element telescope
        let body = ctx.bind_single(shift_and_clone(&elem, (1, 0)), |ctx| body.infer(ctx))?;
        let inferred_type = body.expect_typ()?;

        Ok(LocalLet {
            span: *span,
            name: name.clone(),
            typ,
            bound,
            body,
            inferred_type: Some(inferred_type),
        })
    }
}
