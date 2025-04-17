use std::rc::Rc;

mod codatatype;
mod codefinition;
mod datatype;
mod definition;
mod global_let;
mod infix_declaration;

use ast::*;
use miette_util::ToMiette;
use printer::Print;

use crate::result::TcResult;

use super::{TypeError, ctx::Ctx, type_info_table::TypeInfoTable};

/// Check a module
///
/// The caller of this function needs to resolve module dependencies, check all dependencies, and provide a info table with all symbols from these dependencies.
pub fn check_with_lookup_table(prg: Rc<Module>, info_table: &TypeInfoTable) -> TcResult<Module> {
    log::debug!("Checking module: {}", prg.uri);

    let mut ctx = Ctx::new(prg.meta_vars.clone(), info_table.clone(), prg.clone());

    let mut decls =
        prg.decls.iter().map(|decl| decl.check_wf(&mut ctx)).collect::<TcResult<Vec<_>>>()?;

    decls
        .zonk(&ctx.meta_vars)
        .map_err(|err| TypeError::Impossible { message: err.to_string(), span: None })?;

    check_metavars_solved(&ctx.meta_vars)?;
    check_metavars_resolved(&ctx.meta_vars, &decls)?;

    Ok(Module {
        uri: prg.uri.clone(),
        use_decls: prg.use_decls.clone(),
        decls,
        meta_vars: ctx.meta_vars.clone(),
    })
}

/// Check that there are no unresolved metavariables that remain after typechecking.
pub fn check_metavars_solved(meta_vars: &HashMap<MetaVar, MetaVarState>) -> TcResult {
    let mut unsolved: HashSet<MetaVar> = HashSet::default();
    for (var, state) in meta_vars.iter() {
        // We only have to throw an error for unsolved metavars which were either
        // inserted or are holes `_` which must be solved
        // Unsolved metavariables that correspond to typed holes `?` do not lead
        // to an error.
        if !state.is_solved() && var.must_be_solved() {
            unsolved.insert(*var);
        }
    }

    // We are only throwing one error for the first unresolved metavariable in the hashset.
    // Ideally we want to throw multiple errors here, but this functionality is
    // not yet implemented.
    if let Some(mv) = unsolved.iter().next() {
        let err = TypeError::UnresolvedMeta {
            span: mv.span.to_miette(),
            meta_var: mv.print_to_string(None),
        };
        return Err(err.into());
    }
    Ok(())
}

/// Check that there are no must-solve metavariables whose solution references
/// other metavariables.
fn check_metavars_resolved(meta_vars: &HashMap<MetaVar, MetaVarState>, decls: &[Decl]) -> TcResult {
    // Check in module metavars table
    for (var, state) in meta_vars.iter() {
        if var.must_be_solved() {
            let solution = state.solution().unwrap();
            if solution.contains_metavars() {
                return Err(TypeError::Impossible { message:
                    format!("Metavariable {} must be solved, but its solution references other metavariables", var.id),
                    span: None,
                }.into());
            }
        }
    }
    // Check in all declarations
    for decl in decls {
        if decl.contains_metavars() {
            return Err(TypeError::Impossible {
                message: format!(
                    "Declaration {:?} contains unresolved metavariables",
                    decl.ident()
                ),
                span: decl.span().to_miette(),
            }
            .into());
        }
    }
    Ok(())
}

pub trait CheckToplevel: Sized {
    fn check_wf(&self, ctx: &mut Ctx) -> TcResult<Self>;
}

/// Check a declaration
impl CheckToplevel for Decl {
    fn check_wf(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let out = match self {
            Decl::Data(data) => Decl::Data(data.check_wf(ctx)?),
            Decl::Codata(codata) => Decl::Codata(codata.check_wf(ctx)?),
            Decl::Def(def) => Decl::Def(def.check_wf(ctx)?),
            Decl::Codef(codef) => Decl::Codef(codef.check_wf(ctx)?),
            Decl::Let(tl_let) => Decl::Let(tl_let.check_wf(ctx)?),
            Decl::Infix(infix) => Decl::Infix(infix.check_wf(ctx)?),
        };
        Ok(out)
    }
}
