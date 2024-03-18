use std::rc::Rc;

use syntax::common::*;
use syntax::ctx::{BindContext, LevelCtx};
use syntax::generic::DocComment;
use syntax::ust;
use syntax::ust::util::Instantiate;

use crate::result::XfuncError;
use codespan::Span;

#[derive(Debug, Clone)]
pub struct Prg {
    pub map: HashMap<Ident, XData>,
    pub exp: Option<Rc<ust::Exp>>,
}

#[derive(Debug, Clone)]
pub struct XData {
    pub repr: Repr,
    pub info: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub typ: Rc<ust::TypAbs>,
    pub ctors: HashMap<Ident, ust::Ctor>,
    pub dtors: HashMap<Ident, ust::Dtor>,
    pub exprs: HashMap<Key, Option<Rc<ust::Exp>>>,
}

/// A key points to a matrix cell
///
/// The binding order in the matrix cell is as follors:
/// * dtor telescope
/// * ctor telescope
/// This invariant needs to be handled when translating
/// between the matrix and other representations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub ctor: Ident,
    pub dtor: Ident,
}

#[derive(Debug, Clone, Copy)]
pub enum Repr {
    Data,
    Codata,
}

/// Take the red pill
pub fn build(prg: &ust::Prg) -> Result<Prg, XfuncError> {
    let mut out = Prg { map: HashMap::default(), exp: None };
    let mut ctx = Ctx::empty();
    prg.build_matrix(&mut ctx, &mut out)?;
    Ok(out)
}

pub trait BuildMatrix {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError>;
}

pub struct Ctx {
    type_for_xtor: HashMap<Ident, Ident>,
}

impl Ctx {
    pub fn empty() -> Self {
        Self { type_for_xtor: HashMap::default() }
    }
}

impl BuildMatrix for ust::Prg {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError> {
        let ust::Prg { decls, exp } = self;
        out.exp = exp.clone();

        for decl in decls.map.values() {
            match decl {
                ust::Decl::Data(data) => data.build_matrix(ctx, out),
                ust::Decl::Codata(codata) => codata.build_matrix(ctx, out),
                _ => Ok(()),
            }?
        }

        for decl in decls.map.values() {
            match decl {
                ust::Decl::Ctor(ctor) => ctor.build_matrix(ctx, out),
                ust::Decl::Dtor(dtor) => dtor.build_matrix(ctx, out),
                ust::Decl::Def(def) => def.build_matrix(ctx, out),
                ust::Decl::Codef(codef) => codef.build_matrix(ctx, out),
                _ => Ok(()),
            }?
        }
        Ok(())
    }
}

impl BuildMatrix for ust::Data {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError> {
        let ust::Data { info, doc, name, hidden: _, typ, ctors } = self;

        let xdata = XData {
            repr: Repr::Data,
            info: *info,
            doc: doc.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
        };

        for ctor in ctors {
            ctx.type_for_xtor.insert(ctor.name().clone(), name.clone());
        }

        out.map.insert(name.clone(), xdata);
        Ok(())
    }
}

impl BuildMatrix for ust::Codata {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError> {
        let ust::Codata { info, doc, name, hidden: _, typ, dtors } = self;

        let xdata = XData {
            repr: Repr::Codata,
            info: *info,
            doc: doc.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
        };

        for dtor in dtors {
            ctx.type_for_xtor.insert(dtor.name().clone(), name.clone());
        }

        out.map.insert(name.clone(), xdata);

        Ok(())
    }
}

impl BuildMatrix for ust::Ctor {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &ctx.type_for_xtor.get(&self.name).ok_or(XfuncError::Impossible {
            message: format!("Could not resolve {}", self.name),
            span: None,
        })?;
        let xdata = out.map.get_mut(*type_name).ok_or(XfuncError::Impossible {
            message: format!("Could not resolve {}", self.name),
            span: None,
        })?;
        xdata.ctors.insert(self.name.clone(), self.clone());
        Ok(())
    }
}

impl BuildMatrix for ust::Dtor {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &ctx.type_for_xtor.get(&self.name).ok_or(XfuncError::Impossible {
            message: format!("Could not resolve {}", self.name),
            span: None,
        })?;
        let xdata = out.map.get_mut(*type_name).ok_or(XfuncError::Impossible {
            message: format!("Could not resolve {}", type_name),
            span: None,
        })?;
        xdata.dtors.insert(self.name.clone(), self.clone());
        Ok(())
    }
}

impl BuildMatrix for ust::Def {
    fn build_matrix(&self, _ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &self.self_param.typ.name;
        let xdata = out.map.get_mut(type_name).ok_or(XfuncError::Impossible {
            message: format!("Could not resolve {type_name}"),
            span: None,
        })?;
        xdata.dtors.insert(self.name.clone(), self.to_dtor());

        let ust::Match { cases, .. } = &self.body;

        for case in cases {
            let ust::Case { name, body, .. } = case;
            let key = Key { dtor: self.name.clone(), ctor: name.clone() };
            xdata.exprs.insert(key, body.clone());
        }
        Ok(())
    }
}

impl BuildMatrix for ust::Codef {
    fn build_matrix(&self, _ctx: &mut Ctx, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &self.typ.name;
        let xdata = out.map.get_mut(type_name).ok_or(XfuncError::Impossible {
            message: format!("Could not resolve {type_name}"),
            span: None,
        })?;
        xdata.ctors.insert(self.name.clone(), self.to_ctor());

        let ust::Match { cases, .. } = &self.body;

        for case in cases {
            let ust::Case { name, body, .. } = case;
            let key = Key { ctor: self.name.clone(), dtor: name.clone() };
            // Swap binding order to the order imposed by the matrix representation
            let body = body.as_ref().map(|body| {
                let mut ctx = LevelCtx::empty();
                // TODO: Reconsider where to swap this
                ctx.bind_iter(self.params.params.iter().map(|_| ()), |ctx| {
                    ctx.bind_iter(case.args.params.iter().map(|_| ()), |ctx| {
                        body.swap_with_ctx(ctx, 0, 1)
                    })
                })
            });
            xdata.exprs.insert(key, body.clone());
        }
        Ok(())
    }
}

impl XData {
    pub fn as_data(&self) -> (ust::Data, Vec<ust::Ctor>, Vec<ust::Def>) {
        let XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let data = ust::Data {
            info: None,
            doc: doc.clone(),
            name: name.clone(),
            hidden: false,
            typ: typ.clone(),
            ctors: ctors.keys().cloned().collect(),
        };

        let defs = dtors
            .values()
            .map(|dtor| {
                let mut omit_absurd = false;
                let cases = ctors
                    .values()
                    .flat_map(|ctor| {
                        let key = Key { dtor: dtor.name.clone(), ctor: ctor.name.clone() };
                        let body = exprs.get(&key).cloned();
                        if body.is_none() {
                            omit_absurd = true;
                        }
                        body.map(|body| ust::Case {
                            info: None,
                            name: ctor.name.clone(),
                            args: ctor.params.instantiate(),
                            body,
                        })
                    })
                    .collect();

                ust::Def {
                    info: None,
                    doc: dtor.doc.clone(),
                    name: dtor.name.clone(),
                    hidden: false,
                    params: dtor.params.clone(),
                    self_param: dtor.self_param.clone(),
                    ret_typ: dtor.ret_typ.clone(),
                    body: ust::Match { cases, info: None, omit_absurd },
                }
            })
            .collect();

        let ctors = ctors.values().cloned().collect();

        (data, ctors, defs)
    }

    pub fn as_codata(&self) -> (ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>) {
        let XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let codata = ust::Codata {
            info: None,
            doc: doc.clone(),
            name: name.clone(),
            hidden: false,
            typ: typ.clone(),
            dtors: dtors.keys().cloned().collect(),
        };

        let codefs = ctors
            .values()
            .map(|ctor| {
                let mut omit_absurd = false;
                let cases = dtors
                    .values()
                    .flat_map(|dtor| {
                        let key = Key { dtor: dtor.name.clone(), ctor: ctor.name.clone() };
                        let body = &exprs.get(&key);
                        // Swap binding order (which is different in the matrix representation)
                        let body = body.as_ref().map(|body| {
                            let mut ctx = LevelCtx::empty();
                            ctx.bind_iter(dtor.params.params.iter(), |ctx| {
                                ctx.bind_iter(ctor.params.params.iter(), |ctx| {
                                    body.swap_with_ctx(ctx, 0, 1)
                                })
                            })
                        });
                        if body.is_none() {
                            omit_absurd = true;
                        }
                        body.map(|body| ust::Case {
                            info: None,
                            name: dtor.name.clone(),
                            args: dtor.params.instantiate(),
                            body,
                        })
                    })
                    .collect();

                ust::Codef {
                    info: None,
                    doc: ctor.doc.clone(),
                    name: ctor.name.clone(),
                    hidden: false,
                    params: ctor.params.clone(),
                    typ: ctor.typ.clone(),
                    body: ust::Match { cases, info: None, omit_absurd },
                }
            })
            .collect();

        let dtors = dtors.values().cloned().collect();

        (codata, dtors, codefs)
    }
}
