use ast::Exp;
use elaborator::normalizer::normalize::Normalize;

use crate::*;

impl<'a> DatabaseViewMut<'a> {
    pub fn run(&mut self) -> Result<Option<Box<Exp>>, Error> {
        let ast = self.load_module()?;

        let main = ast.find_main();

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&ast)?;
                Ok(Some(nf))
            }
            None => Ok(None),
        }
    }

    pub fn source(&'a self) -> &'a str {
        let DatabaseViewMut { uri: url, database } = self;
        &database.files.get_even_if_stale(url).unwrap().source
    }

    pub fn pretty_error(&self, err: Error) -> miette::Report {
        let miette_error: miette::Error = err.into();
        miette_error.with_source_code(miette::NamedSource::new(&self.uri, self.source().to_owned()))
    }
}
