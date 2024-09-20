use std::rc::Rc;

use crate::*;
use elaborator::LookupTable;
use renaming::Rename;
use url::Url;

impl Database {
    pub fn load_ast(
        &mut self,
        uri: &Url,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut LookupTable,
    ) -> Result<Arc<ast::Module>, Error> {
        log::trace!("Loading AST: {}", uri);

        match self.ast.get_unless_stale(uri) {
            Some(ast) => {
                *cst_lookup_table = self.cst_lookup_table.get_even_if_stale(uri).unwrap().clone();
                *ast_lookup_table = self.ast_lookup_table.get_even_if_stale(uri).unwrap().clone();
                ast.clone()
            }
            None => {
                log::trace!("AST is stale, reloading");
                let ust = self.load_ust(uri, cst_lookup_table);
                let ast = ust
                    .and_then(|ust| {
                        let tst = elaborator::typechecker::check_with_lookup_table(
                            Rc::new(ust),
                            ast_lookup_table,
                        )
                        .map_err(Error::Type)?;
                        Ok(tst)
                    })
                    .map(Arc::new);
                self.ast.insert(uri.clone(), ast.clone());
                self.ast_lookup_table.insert(uri.clone(), ast_lookup_table.clone());
                self.cst_lookup_table.insert(uri.clone(), cst_lookup_table.clone());
                if let Ok(module) = &ast {
                    let (info_lapper, item_lapper) = collect_info(module.clone());
                    self.info_by_id.insert(uri.clone(), info_lapper);
                    self.item_by_id.insert(uri.clone(), item_lapper);
                }
                ast
            }
        }
    }

    pub fn load_ust(
        &mut self,
        uri: &Url,
        cst_lookup_table: &mut lowering::LookupTable,
    ) -> Result<ast::Module, Error> {
        log::trace!("Loading UST: {}", uri);

        let cst = self.load_cst(uri)?;
        log::debug!("Lowering module");
        lowering::lower_module_with_lookup_table(&cst, cst_lookup_table).map_err(Error::Lowering)
    }

    pub fn load_cst(&mut self, uri: &Url) -> Result<Arc<cst::decls::Module>, Error> {
        match self.cst.get_unless_stale(uri) {
            Some(cst) => cst.clone(),
            None => {
                let source = self.load_source(uri)?;
                let module = {
                    let source: &str = &source;
                    log::debug!("Parsing module: {}", uri);
                    parser::parse_module(uri.clone(), source).map_err(Error::Parser)
                }
                .map(Arc::new);
                self.cst.insert(uri.clone(), module.clone());
                module
            }
        }
    }

    pub fn load_source(&mut self, uri: &Url) -> Result<String, Error> {
        self.open_uri(uri)?;
        match self.files.get_unless_stale(uri) {
            Some(file) => Ok(file.source().to_string()),
            None => {
                let source = self.source.read_to_string(uri)?;
                let file = codespan::File::new(uri.as_str().into(), source.clone());
                self.files.insert(uri.clone(), file);
                Ok(source)
            }
        }
    }

    pub fn write_source(&mut self, uri: &Url, source: &str) -> Result<(), Error> {
        self.invalidate(uri)?;
        self.source.write_string(uri, source).map_err(|err| err.into())
    }

    pub fn print_to_string(&mut self, uri: &Url) -> Result<String, Error> {
        let module =
            self.load_ast(uri, &mut lowering::LookupTable::default(), &mut LookupTable::default())?;
        let module = (*module).clone().rename();
        Ok(printer::Print::print_to_string(&module, None))
    }
}
