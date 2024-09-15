use std::rc::Rc;

use crate::*;
use elaborator::LookupTable;
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
                let ast = ust.and_then(|ust| load_ast(ust, ast_lookup_table)).map(Arc::new);
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
        load_ust(cst, cst_lookup_table)
    }

    pub fn load_cst(&mut self) -> Result<Arc<cst::decls::Module>, Error> {
        match self.database.cst.get_unless_stale(self.database, &self.uri) {
            Some(cst) => cst.clone(),
            None => {
                let source = self.load_source()?;
                let module = load_cst(&self.uri, &source).map(Arc::new);
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

    pub fn print_to_string(&mut self) -> Result<String, Error> {
        let module =
            self.load_ast(&mut lowering::LookupTable::default(), &mut LookupTable::default())?;
        Ok(printer::Print::print_to_string(&*module, None))
    }

    pub fn reset(&mut self) {
        self.database.info_by_id.insert(self.uri.clone(), Lapper::new(vec![]));
        self.database.item_by_id.insert(self.uri.clone(), Lapper::new(vec![]));
        self.database.ast.remove(&self.uri);
    }

    pub fn set(&mut self, info_index: Lapper<u32, Info>, item_index: Lapper<u32, Item>) {
        self.database.info_by_id.insert(self.uri.clone(), info_index);
        self.database.item_by_id.insert(self.uri.clone(), item_index);
    }
}

fn load_cst(url: &Url, source: &str) -> Result<cst::decls::Module, Error> {
    log::debug!("Parsing module: {}", url);
    parser::parse_module(url.clone(), source).map_err(Error::Parser)
}

fn load_ust(
    cst: Arc<cst::decls::Module>,
    cst_lookup_table: &mut lowering::LookupTable,
) -> Result<ast::Module, Error> {
    log::debug!("Lowering module");
    lowering::lower_module_with_lookup_table(&cst, cst_lookup_table).map_err(Error::Lowering)
}

fn load_ast(ust: ast::Module, ast_lookup_table: &mut LookupTable) -> Result<ast::Module, Error> {
    let tst = elaborator::typechecker::check_with_lookup_table(Rc::new(ust), ast_lookup_table)
        .map_err(Error::Type)?;
    Ok(tst)
}
