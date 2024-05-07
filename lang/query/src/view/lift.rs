use lifting::LiftResult;

use syntax::generic;

use super::DatabaseView;

impl<'a> DatabaseView<'a> {
    pub fn lift(&self, type_name: &str) -> Result<generic::Module, crate::Error> {
        let prg = self.tst()?;

        let LiftResult { prg, .. } = lifting::lift(prg, type_name);

        Ok(prg)
    }
}
