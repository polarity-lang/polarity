use polarity_lang_ast::ctx::LevelCtx;
use polarity_lang_ast::*;

use crate::result::TcResult;

use super::TypeError;

// Checks whether the codata type contains destructors with a self parameter
pub fn uses_self(codata: &Codata) -> TcResult<bool> {
    for dtor in &codata.dtors {
        let mut ctx =
            LevelCtx::from(vec![dtor.params.params.clone(), vec![dtor.self_param.to_param()]]);
        if dtor.ret_typ.occurs_var(&mut ctx, Lvl { fst: 1, snd: 0 }) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub trait ExpectTypApp {
    fn expect_typ_app(&self) -> TcResult<TypCtor>;
}

impl ExpectTypApp for Exp {
    fn expect_typ_app(&self) -> TcResult<TypCtor> {
        match self {
            Exp::TypCtor(TypCtor { span, name, args, is_bin_op }) => Ok(TypCtor {
                span: *span,
                name: name.clone(),
                args: args.clone(),
                is_bin_op: is_bin_op.clone(),
            }),
            _ => Err(TypeError::expected_typ_app(self).into()),
        }
    }
}

pub trait ExpectIo {
    fn expect_io(&self) -> TcResult<Box<Exp>>;
}

impl ExpectIo for Exp {
    fn expect_io(&self) -> TcResult<Box<Exp>> {
        let Exp::Call(Call {
            span: _,
            kind: CallKind::Extern,
            name: IdBound { span: _, id: name, uri: _ },
            args,
            inferred_type: _,
        }) = self
        else {
            todo!()
        };

        if name.as_str() != "IO" {
            todo!()
        }

        let args = args.to_exps();
        if args.len() != 1 {
            todo!()
        }

        let inner_typ = args.into_iter().next().unwrap();
        Ok(inner_typ)
    }
}
