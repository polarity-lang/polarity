use std::rc::Rc;

use crate::*;
use elaborator::LookupTable;
use renaming::Rename;
use url::Url;

/// Mutable view on a file in the database
pub struct DatabaseViewMut<'a> {
    pub(crate) uri: Url,
    pub(crate) database: &'a mut Database,
}

impl<'a> DatabaseViewMut<'a> {
    pub fn load_ast(
        &mut self,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut LookupTable,
    ) -> Result<Arc<ast::Module>, Error> {
        log::trace!("Loading AST: {}", self.uri);

        match self.database.ast.get_unless_stale(self.database, &self.uri) {
            Some(ast) => {
                *cst_lookup_table =
                    self.database.cst_lookup_table.get_even_if_stale(&self.uri).unwrap().clone();
                *ast_lookup_table =
                    self.database.ast_lookup_table.get_even_if_stale(&self.uri).unwrap().clone();
                ast.clone()
            }
            None => {
                log::trace!("AST is stale, reloading");
                let ust = self.load_ust(cst_lookup_table);
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
                self.database.ast.insert(self.uri.clone(), ast.clone());
                self.database.ast_lookup_table.insert(self.uri.clone(), ast_lookup_table.clone());
                self.database.cst_lookup_table.insert(self.uri.clone(), cst_lookup_table.clone());
                if let Ok(module) = &ast {
                    let (info_lapper, item_lapper) = collect_info(module.clone());
                    self.set(info_lapper, item_lapper);
                }
                ast
            }
        }
    }

    pub fn load_ust(
        &mut self,
        cst_lookup_table: &mut lowering::LookupTable,
    ) -> Result<ast::Module, Error> {
        log::trace!("Loading UST: {}", self.uri);

        let cst = self.load_cst()?;
        log::debug!("Lowering module");
        lowering::lower_module_with_lookup_table(&cst, cst_lookup_table).map_err(Error::Lowering)
    }

    pub fn load_cst(&mut self) -> Result<Arc<cst::decls::Module>, Error> {
        match self.database.cst.get_unless_stale(self.database, &self.uri) {
            Some(cst) => cst.clone(),
            None => {
                let source = self.load_source()?;
                let module = {
                    let url: &Url = &self.uri;
                    let source: &str = &source;
                    log::debug!("Parsing module: {}", url);
                    parser::parse_module(url.clone(), source).map_err(Error::Parser)
                }
                .map(Arc::new);
                self.database.cst.insert(self.uri.clone(), module.clone());
                module
            }
        }
    }

    pub fn load_source(&mut self) -> Result<String, Error> {
        match self.database.files.get_unless_stale(self.database, &self.uri) {
            Some(file) => Ok(file.source().to_string()),
            None => {
                let source = self.database.source.read_to_string(&self.uri)?;
                let uri = self.uri.clone();
                let file = codespan::File::new(uri.as_str().into(), source.clone());
                self.database.files.insert(uri.clone(), file);
                Ok(source)
            }
        }
    }

    pub fn write_source(&mut self, source: &str) -> Result<(), Error> {
        self.reset();
        self.database.source.write_string(&self.uri, source).map_err(|err| err.into())
    }

    pub fn print_to_string(&mut self) -> Result<String, Error> {
        let module =
            self.load_ast(&mut lowering::LookupTable::default(), &mut LookupTable::default())?;
        let module = (*module).clone().rename();
        Ok(printer::Print::print_to_string(&module, None))
    }

    pub fn reset(&mut self) {
        self.database.ast.remove(&self.uri);
        self.database.ast_lookup_table.remove(&self.uri);
        self.database.cst.remove(&self.uri);
        self.database.cst_lookup_table.remove(&self.uri);
        self.database.info_by_id.insert(self.uri.clone(), Lapper::new(vec![]));
        self.database.item_by_id.insert(self.uri.clone(), Lapper::new(vec![]));
    }

    pub fn set(&mut self, info_index: Lapper<u32, Info>, item_index: Lapper<u32, Item>) {
        self.database.info_by_id.insert(self.uri.clone(), info_index);
        self.database.item_by_id.insert(self.uri.clone(), item_index);
    }
}
