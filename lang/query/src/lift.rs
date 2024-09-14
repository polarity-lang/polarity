use lifting::LiftResult;

use crate::DatabaseViewMut;

impl<'a> DatabaseViewMut<'a> {
    pub fn lift(&mut self, type_name: &str) -> Result<ast::Module, crate::Error> {
        let prg = self.load_module()?;

        let LiftResult { module: prg, .. } = lifting::lift(prg, type_name);

        Ok(prg)
    }
}
