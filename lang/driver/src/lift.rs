use transformations::LiftResult;
use url::Url;

use crate::database::Database;

impl Database {
    pub async fn lift(&mut self, uri: &Url, type_name: &str) -> Result<ast::Module, crate::Error> {
        let prg = self.ast(uri).await?;

        let LiftResult { module: prg, .. } = transformations::lift(prg, type_name);

        Ok(prg)
    }
}
