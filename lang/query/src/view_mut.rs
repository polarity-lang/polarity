use std::rc::Rc;

use crate::*;
use url::Url;

/// Mutable view on a file in the database
pub struct DatabaseViewMut<'a> {
    pub(crate) url: Url,
    pub(crate) database: &'a mut Database,
}

impl<'a> DatabaseViewMut<'a> {
    pub fn load(&mut self) -> Result<(), Error> {
        self.reset();
        let module = load_module(&self.url, self.database).map(Arc::new);
        self.database.ast.insert(self.url.clone(), module.clone());
        if let Ok(module) = &module {
            let (info_lapper, item_lapper) = collect_info(module.clone());
            self.set(info_lapper, item_lapper);
        }
        module.map(|_| ())
    }

    pub fn query(self) -> DatabaseView<'a> {
        let DatabaseViewMut { url, database } = self;
        DatabaseView { url, database }
    }

    pub fn query_ref(&self) -> DatabaseView<'_> {
        let DatabaseViewMut { url, database } = self;
        DatabaseView { url: url.clone(), database }
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

fn load_module(url: &Url, database: &Database) -> Result<ast::Module, Error> {
    let source = database.files.get(url).unwrap().source();
    let cst = parser::parse_module(url.clone(), source).map_err(Error::Parser)?;
    let ust = lowering::lower_module(&cst).map_err(Error::Lowering)?;
    let tst = elaborator::typechecker::check(Rc::new(ust)).map_err(Error::Type)?;
    Ok(tst)
}
