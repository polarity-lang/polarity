use transformations::LiftResult;
use url::Url;

use crate::database::Database;

impl Database {
    pub fn lift(&mut self, uri: &Url, type_name: &str) -> Result<ast::Module, crate::Error> {
        let prg = self.load_module(uri)?;

        let LiftResult { module: prg, .. } = transformations::lift(prg, type_name);

        Ok(prg)
    }
}
