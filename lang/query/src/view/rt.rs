use std::rc::Rc;

use super::DatabaseView;

use ast::{Exp, Module};
use elaborator::normalizer::normalize::Normalize;
use parser::cst;

use crate::*;

impl<'a> DatabaseView<'a> {
    pub fn source(&self) -> &'a str {
        let DatabaseView { url, database } = self;
        &database.files.get(url).unwrap().source
    }

    pub fn cst(&self) -> Result<cst::decls::Module, Error> {
        let source = self.source();
        parser::parse_module(self.url.clone(), source).map_err(Error::Parser)
    }

    pub fn ust(&self) -> Result<Module, Error> {
        let cst = self.cst()?;
        lowering::lower_module(&cst).map_err(Error::Lowering)
    }

    pub fn tst(&self) -> Result<Module, Error> {
        let ust = self.ust()?;
        elaborator::typechecker::check(Rc::new(ust)).map_err(Error::Type)
    }

    pub fn run(&self) -> Result<Option<Rc<Exp>>, Error> {
        let tst = self.tst()?;

        let main = tst.find_main();

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&tst)?;
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
