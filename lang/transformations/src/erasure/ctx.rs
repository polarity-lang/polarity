use url::Url;

use super::symbol_table::GlobalErasureSymbolTable;

pub struct Ctx {
    /// Global symbol table, tracking erased parameters
    pub symbol_table: GlobalErasureSymbolTable,
    /// URI of the current module
    pub uri: Url,
}

impl Ctx {
    pub fn new(uri: Url, symbol_table: GlobalErasureSymbolTable) -> Self {
        Self { symbol_table, uri }
    }
}
