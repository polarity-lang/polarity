use crate::ir::{self, ModuleSymbolTable};

use super::erasure_symbol_table::{Codata, Ctor, Data, Dtor, ModuleErasureSymbolTable, Param};

pub fn build_ir_symbol_table(table: ModuleErasureSymbolTable) -> ir::ModuleSymbolTable {
    let ModuleErasureSymbolTable { map_data, map_codata, .. } = table;

    let mut ir_table = ir::ModuleSymbolTable::default();

    for data in map_data.into_values() {
        add_data(&mut ir_table, data);
    }

    for codata in map_codata.into_values() {
        add_codata(&mut ir_table, codata);
    }

    ir_table
}

fn add_data(ir_table: &mut ModuleSymbolTable, data: Data) {
    let Data { name, ctors } = data;

    let data = ir::Data {
        name: name.clone(),
        ctors: ctors
            .into_iter()
            .map(|Ctor { name, params }| ir::Ctor { name, params: erase_params(params) })
            .collect(),
    };

    ir_table.data_decls.insert(name, data);
}

fn add_codata(ir_table: &mut ModuleSymbolTable, codata: Codata) {
    let Codata { name, dtors } = codata;

    let codata = ir::Codata {
        name: name.clone(),
        dtors: dtors
            .into_iter()
            .filter(|dtor| !dtor.erased)
            .map(|Dtor { name, self_param, params, .. }| ir::Dtor {
                name,
                self_param,
                params: erase_params(params),
            })
            .collect(),
    };

    ir_table.codata_decls.insert(name, codata);
}

fn erase_params(params: Vec<Param>) -> Vec<String> {
    params.into_iter().filter(|param| !param.erased).map(|param| param.name).collect()
}
