use std::rc::Rc;

use super::DatabaseView;

use elaborator::normalizer::normalize::Normalize;
use parser::cst;
use syntax::{generic::Exp, generic::ForgetTST, generic::Module};

use crate::*;

impl<'a> DatabaseView<'a> {
    pub fn filename(&self) -> &str {
        self.database.files.name(self.file_id).to_str().unwrap()
    }

    pub fn source(&self) -> &'a str {
        let DatabaseView { file_id, database } = self;
        database.files.source(*file_id)
    }

    pub fn cst(&self) -> Result<cst::decls::Module, Error> {
        let source = self.source();
        parser::parse_module(Path::new(self.filename()), source).map_err(Error::Parser)
    }

    pub fn ust(&self) -> Result<Module, Error> {
        let cst = self.cst()?;
        lowering::lower_module(&cst).map_err(Error::Lowering)
    }

    pub fn tst(&self) -> Result<Module, Error> {
        let ust = self.ust()?;
        elaborator::typechecker::check(&ust).map_err(Error::Type)
    }

    pub fn run(&self) -> Result<Option<Rc<Exp>>, Error> {
        let tst = self.tst()?;
        let ust = tst.forget_tst();

        let main = ust.find_main();

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&ust)?;
                Ok(Some(nf))
            }
            None => Ok(None),
        }
    }

    pub fn pretty_error(&self, err: Error) -> miette::Report {
        let miette_error: miette::Error = err.into();
        miette_error
            .with_source_code(miette::NamedSource::new(self.filename(), self.source().to_owned()))
    }
}
