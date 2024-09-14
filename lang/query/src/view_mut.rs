use std::rc::Rc;

use crate::*;
use elaborator::typechecker::lookup_table::LookupTable;
use url::Url;

/// Mutable view on a file in the database
pub struct DatabaseViewMut<'a> {
    pub(crate) url: Url,
    pub(crate) database: &'a mut Database,
}

impl<'a> DatabaseViewMut<'a> {
    pub fn load_cst(&mut self) -> Result<Arc<cst::decls::Module>, Error> {
        if self.database.source.is_modified(&self.url)?
            || !self.database.cst.contains_key(&self.url)
        {
            self.reset();
            let module = load_cst(&self.url, self.database).map(Arc::new);
            self.database.cst.insert(self.url.clone(), module.clone());
            module
        } else {
            self.database.cst.get(&self.url).unwrap().clone()
        }
    }

    pub fn load_ast(
        &mut self,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut LookupTable,
    ) -> Result<Arc<ast::Module>, Error> {
        if self.database.source.is_modified(&self.url)?
            || !self.database.ast.contains_key(&self.url)
        {
            let cst = self.load_cst()?;
            let ast = load_ast(cst, cst_lookup_table, ast_lookup_table).map(Arc::new);
            self.database.ast.insert(self.url.clone(), ast.clone());
            if let Ok(module) = &ast {
                let (info_lapper, item_lapper) = collect_info(module.clone());
                self.set(info_lapper, item_lapper);
            }
            ast
        } else {
            self.database.ast.get(&self.url).unwrap().clone()
        }
    }

    pub fn reset(&mut self) {
        self.database.info_by_id.insert(self.url.clone(), Lapper::new(vec![]));
        self.database.item_by_id.insert(self.url.clone(), Lapper::new(vec![]));
        self.database.ast.remove(&self.url);
    }

    pub fn set(&mut self, info_index: Lapper<u32, Info>, item_index: Lapper<u32, Item>) {
        self.database.info_by_id.insert(self.url.clone(), info_index);
        self.database.item_by_id.insert(self.url.clone(), item_index);
    }
}

fn load_cst(url: &Url, database: &Database) -> Result<cst::decls::Module, Error> {
    log::debug!("Parsing module: {}", url);
    let source = database.files.get(url).unwrap().source();
    parser::parse_module(url.clone(), source).map_err(Error::Parser)
}

fn load_ast(
    cst: Arc<cst::decls::Module>,
    cst_lookup_table: &mut lowering::LookupTable,
    ast_lookup_table: &mut LookupTable,
) -> Result<ast::Module, Error> {
    let ust = lowering::lower_module_with_lookup_table(&cst, cst_lookup_table)
        .map_err(Error::Lowering)?;
    let tst = elaborator::typechecker::check_with_lookup_table(Rc::new(ust), ast_lookup_table)
        .map_err(Error::Type)?;
    Ok(tst)
}
