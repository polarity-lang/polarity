use lifting::LiftResult;

use syntax::ast;

use super::DatabaseView;

impl<'a> DatabaseView<'a> {
    pub fn lift(&self, type_name: &str) -> Result<ast::Module, crate::Error> {
        let prg = self.tst()?;

        let LiftResult { module: prg, .. } = lifting::lift(prg, type_name);

        Ok(prg)
    }
}
