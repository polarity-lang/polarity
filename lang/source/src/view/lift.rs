use lifting::LiftResult;

use syntax::common::*;
use syntax::ust;

use super::DatabaseView;

impl<'a> DatabaseView<'a> {
    pub fn lift(&self, type_name: &str) -> Result<ust::Prg, crate::Error> {
        let prg = self.tst()?;

        let LiftResult { prg, .. } = lifting::lift(prg.forget(), type_name);

        Ok(prg)
    }
}
