use ast::Exp;
use elaborator::normalizer::normalize::Normalize;

use crate::*;

impl Database {
    pub fn run(&mut self, uri: &Url) -> Result<Option<Box<Exp>>, Error> {
        let ast = self.load_module(uri)?;

        let main = ast.find_main();

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&ast)?;
                Ok(Some(nf))
            }
            None => Ok(None),
        }
    }

    pub fn pretty_error(&self, uri: &Url, err: Error) -> miette::Report {
        let miette_error: miette::Error = err.into();
        let source = &self.files.get_even_if_stale(uri).unwrap().source;
        miette_error.with_source_code(miette::NamedSource::new(uri, source.to_owned()))
    }
}
