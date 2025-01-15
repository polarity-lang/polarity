use ast::ctx::LevelCtx;
use ast::*;

use super::TypeError;

// Checks whether the codata type contains destructors with a self parameter
pub fn uses_self(codata: &Codata) -> Result<bool, TypeError> {
    for dtor in &codata.dtors {
        let mut ctx = LevelCtx::from(vec![dtor.params.len(), 1]);
        if dtor.ret_typ.occurs(&mut ctx, Lvl { fst: 1, snd: 0 }) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub trait ExpectTypApp {
    fn expect_typ_app(&self) -> Result<TypCtor, TypeError>;
}

impl ExpectTypApp for Exp {
    fn expect_typ_app(&self) -> Result<TypCtor, TypeError> {
        match self {
            Exp::TypCtor(TypCtor { span, name, args }) => {
                Ok(TypCtor { span: *span, name: name.clone(), args: args.clone() })
            }
            _ => Err(TypeError::expected_typ_app(self)),
        }
    }
}
