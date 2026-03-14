use polarity_lang_ast::ctx::BindContext;
use polarity_lang_ast::ctx::values::Binder;
use polarity_lang_ast::ctx::values::Binding;
use polarity_lang_ast::ctx::values::BoundValue;
use polarity_lang_ast::*;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::result::TcResult;
use crate::typechecker::util::ExpectIo;

use super::super::ctx::*;
use super::CheckInfer;
use super::ExpectType;

impl CheckInfer for DoBlock {
    fn check(&self, ctx: &mut Ctx, t: &polarity_lang_ast::Exp) -> TcResult<Self> {
        let DoBlock { span, statements, inferred_type: _ } = self;
        let statements = statements.check(ctx, t)?;
        let inferred_type = statements.expect_typ()?;
        Ok(DoBlock { span: *span, statements, inferred_type: Some(inferred_type) })
    }

    fn infer(&self, ctx: &mut crate::typechecker::ctx::Ctx) -> TcResult<Self> {
        let DoBlock { span, statements, inferred_type: _ } = self;
        let statements = statements.infer(ctx)?;
        let inferred_type = statements.expect_typ()?;
        Ok(DoBlock { span: *span, statements, inferred_type: Some(inferred_type) })
    }
}

impl CheckInfer for DoStatements {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        match self {
            DoStatements::Bind { span, name, bound, body, inferred_type: _ } => {
                let bound = bound.infer(ctx)?;
                let typ = bound.expect_typ()?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
                let inner_typ = typ_nf.expect_io_with_span(bound.span().or(Some(*span)))?;

                let elem =
                    Binder { name: name.clone(), content: Binding { typ: inner_typ, val: None } };

                // We need to shift the binder type here because we treat it as a 1-element telescope
                let body =
                    ctx.bind_single(shift_and_clone(&elem, (1, 0)), |ctx| body.check(ctx, t))?;
                let inferred_type = body.expect_typ()?;

                Ok(DoStatements::Bind {
                    span: *span,
                    name: name.clone(),
                    bound,
                    body,
                    inferred_type: Some(inferred_type),
                })
            }
            DoStatements::Let { span, name, typ, bound, body, inferred_type: _ } => {
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
                let body =
                    ctx.bind_single(shift_and_clone(&elem, (1, 0)), |ctx| body.check(ctx, t))?;
                let inferred_type = body.expect_typ()?;

                Ok(DoStatements::Let {
                    span: *span,
                    name: name.clone(),
                    typ,
                    bound,
                    body,
                    inferred_type: Some(inferred_type),
                })
            }
            DoStatements::Return { span, exp, inferred_type: _ } => {
                let _ = t.expect_io_with_span(Some(*span))?;
                let exp = exp.check(ctx, t)?;
                let inferred_type = exp.expect_typ()?;

                Ok(DoStatements::Return { span: *span, exp, inferred_type: Some(inferred_type) })
            }
        }
    }

    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        match self {
            DoStatements::Bind { span, name, bound, body, inferred_type: _ } => {
                let bound = bound.infer(ctx)?;
                let typ = bound.expect_typ()?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
                let inner_typ = typ_nf.expect_io_with_span(bound.span().or(Some(*span)))?;

                let elem =
                    Binder { name: name.clone(), content: Binding { typ: inner_typ, val: None } };

                // We need to shift the binder type here because we treat it as a 1-element telescope
                let body =
                    ctx.bind_single(shift_and_clone(&elem, (1, 0)), |ctx| body.infer(ctx))?;
                let inferred_type = body.expect_typ()?;

                Ok(DoStatements::Bind {
                    span: *span,
                    name: name.clone(),
                    bound,
                    body,
                    inferred_type: Some(inferred_type),
                })
            }
            DoStatements::Let { span, name, typ, bound, body, inferred_type: _ } => {
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
                let body =
                    ctx.bind_single(shift_and_clone(&elem, (1, 0)), |ctx| body.infer(ctx))?;
                let inferred_type = body.expect_typ()?;

                Ok(DoStatements::Let {
                    span: *span,
                    name: name.clone(),
                    typ,
                    bound,
                    body,
                    inferred_type: Some(inferred_type),
                })
            }
            DoStatements::Return { span, exp, inferred_type: _ } => {
                let exp = exp.infer(ctx)?;
                let inferred_type = exp.expect_typ()?;
                let _ = inferred_type.expect_io_with_span(Some(*span))?;

                Ok(DoStatements::Return { span: *span, exp, inferred_type: Some(inferred_type) })
            }
        }
    }
}
