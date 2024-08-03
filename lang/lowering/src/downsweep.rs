use codespan::Span;
use miette_util::ToMiette;

use parser::cst;
use parser::cst::ident::Ident;
use syntax::ast::lookup_table;
use syntax::ast::lookup_table::DeclMeta;
use syntax::ast::HashMap;

use super::result::*;

/// Build the structure tracking the declaration order in the source code
pub fn build_lookup_table(
    items: &[cst::decls::Decl],
) -> Result<(HashMap<Ident, DeclMeta>, lookup_table::LookupTable), LoweringError> {
    let mut lookup_table = lookup_table::LookupTable::default();
    let mut top_level_map = HashMap::default();

    let mut add_top_level_decl = |name: &Ident, span: &Span, decl_kind: DeclMeta| {
        if top_level_map.contains_key(name) {
            return Err(LoweringError::AlreadyDefined {
                name: name.to_owned(),
                span: Some(span.to_miette()),
            });
        }
        top_level_map.insert(name.clone(), decl_kind);
        Ok(())
    };

    for item in items {
        match item {
            cst::decls::Decl::Data(data) => {
                // top_level_map
                add_top_level_decl(
                    &data.name,
                    &data.span,
                    DeclMeta::Data { arity: data.params.len() },
                )?;
                for ctor in &data.ctors {
                    add_top_level_decl(
                        &ctor.name,
                        &ctor.span,
                        DeclMeta::Ctor { ret_typ: data.name.id.clone() },
                    )?;
                }

                // lookup_table
                let mut typ_decl = lookup_table.add_type_decl(data.name.id.clone());
                let xtors = data.ctors.iter().map(|ctor| ctor.name.id.clone());
                typ_decl.set_xtors(xtors);
            }
            cst::decls::Decl::Codata(codata) => {
                // top_level_map
                add_top_level_decl(
                    &codata.name,
                    &codata.span,
                    DeclMeta::Codata { arity: codata.params.len() },
                )?;
                for dtor in &codata.dtors {
                    add_top_level_decl(
                        &dtor.name,
                        &dtor.span,
                        DeclMeta::Dtor { self_typ: codata.name.id.clone() },
                    )?;
                }

                // lookup_table
                let mut typ_decl = lookup_table.add_type_decl(codata.name.id.clone());
                let xtors = codata.dtors.iter().map(|ctor| ctor.name.id.clone());
                typ_decl.set_xtors(xtors);
            }
            cst::decls::Decl::Def(def) => {
                // top_level_map
                add_top_level_decl(&def.name, &def.span, DeclMeta::Def)?;

                // lookup_table
                let type_name = def.scrutinee.typ.name.clone();
                lookup_table.add_def(type_name.id, def.name.id.to_owned());
            }
            cst::decls::Decl::Codef(codef) => {
                // top_level_map
                add_top_level_decl(&codef.name, &codef.span, DeclMeta::Codef)?;

                // lookup_table
                let type_name = codef.typ.name.clone();
                lookup_table.add_def(type_name.id, codef.name.id.to_owned())
            }
            cst::decls::Decl::Let(tl_let) => {
                // top_level_map
                add_top_level_decl(&tl_let.name, &tl_let.span, DeclMeta::Let)?;

                lookup_table.add_let(tl_let.name.id.clone());
            }
        }
    }

    Ok((top_level_map, lookup_table))
}
