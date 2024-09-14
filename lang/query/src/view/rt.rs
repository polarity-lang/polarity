use super::DatabaseView;

use ast::{Exp, Module};
use elaborator::normalizer::normalize::Normalize;

use crate::*;

impl<'a> DatabaseView<'a> {
    pub fn source(&self) -> &'a str {
        let DatabaseView { url, database } = self;
        &database.files.get(url).unwrap().source
    }

    pub fn ast(&self) -> Result<Arc<Module>, Error> {
        match self.database.ast.get(&self.url).unwrap() {
            Ok(ast) => Ok(ast.clone()),
            Err(err) => Err(err.clone()),
        }
    }

    pub fn run(&self) -> Result<Option<Box<Exp>>, Error> {
        let ast = self.ast()?;

        let main = ast.find_main();

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&ast)?;
                Ok(Some(nf))
            }
            None => Ok(None),
        }
    }

    pub fn pretty_error(&self, err: Error) -> miette::Report {
        let miette_error: miette::Error = err.into();
        miette_error.with_source_code(miette::NamedSource::new(&self.url, self.source().to_owned()))
    }
}
