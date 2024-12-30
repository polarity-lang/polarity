use url::Url;

use super::erasure_symbol_table::{GlobalErasureSymbolTable, Param};

pub struct Ctx {
    /// For each variable in the context, whether it is erased
    pub erased: Vec<Vec<bool>>,
    /// Global symbol table, tracking erased parameters
    pub symbol_table: GlobalErasureSymbolTable,
    /// URI of the current module
    pub uri: Url,
}

impl Ctx {
    pub fn new(uri: Url, symbol_table: GlobalErasureSymbolTable) -> Self {
        Self { erased: Vec::new(), symbol_table, uri }
    }

    pub fn is_erased(&self, idx: ast::Idx) -> bool {
        let fst = self.erased.len() - 1 - idx.fst;
        let snd = self.erased[fst].len() - 1 - idx.snd;
        self.erased[fst][snd]
    }

    /// Bind a telescope of potentially erased parameters
    pub fn bind<'a, T, F, I>(&mut self, params: I, f: F) -> T
    where
        F: FnOnce(&mut Ctx) -> T,
        I: IntoIterator<Item = &'a Param>,
    {
        self.erased.push(params.into_iter().map(|param| param.erased).collect());
        let result = f(self);
        self.erased.pop();
        result
    }

    pub fn bind_single<T>(&mut self, erased: bool, f: impl FnOnce(&mut Ctx) -> T) -> T {
        self.erased.push(vec![erased]);
        let result = f(self);
        self.erased.pop();
        result
    }
}
