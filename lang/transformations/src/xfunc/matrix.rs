use codespan::Span;
use url::Url;

use ast::ctx::{BindContext, LevelCtx};
use ast::{self, HashMap, IdBound, SwapWithCtx};
use ast::{Attributes, DocComment};

use crate::result::XfuncError;

#[derive(Debug, Clone)]
pub struct Prg {
    pub map: HashMap<String, XData>,
    pub uri: Url,
}

#[derive(Debug, Clone)]
pub struct XData {
    pub repr: Repr,
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: ast::IdBind,
    pub typ: Box<ast::Telescope>,
    pub ctors: HashMap<String, ast::Ctor>,
    pub dtors: HashMap<String, ast::Dtor>,
    pub exprs: HashMap<Key, Option<Box<ast::Exp>>>,
}

/// A key points to a matrix cell
///
/// The binding order in the matrix cell is as follors:
/// * dtor telescope
/// * ctor telescope
///
/// This invariant needs to be handled when translating
/// between the matrix and other representations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub ctor: String,
    pub dtor: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Repr {
    Data,
    Codata,
}

/// Take the red pill
pub fn build(prg: &ast::Module) -> Result<Prg, XfuncError> {
    let mut out = Prg { map: HashMap::default(), uri: prg.uri.clone() };
    prg.build_matrix(&mut out)?;
    Ok(out)
}

pub trait BuildMatrix {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError>;
}

impl BuildMatrix for ast::Module {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let ast::Module { decls, .. } = self;

        for decl in decls {
            match decl {
                ast::Decl::Data(data) => data.build_matrix(out),
                ast::Decl::Codata(codata) => codata.build_matrix(out),
                _ => Ok(()),
            }?
        }

        for decl in decls {
            match decl {
                ast::Decl::Def(def) => def.build_matrix(out),
                ast::Decl::Codef(codef) => codef.build_matrix(out),
                _ => Ok(()),
            }?
        }

        Ok(())
    }
}

impl BuildMatrix for ast::Data {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let ast::Data { span, doc, name, attr: _, typ, ctors } = self;

        let mut xdata = XData {
            repr: Repr::Data,
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
        };
        for ctor in ctors {
            xdata.ctors.insert(ctor.name.id.clone(), ctor.clone());
        }

        out.map.insert(name.id.clone(), xdata);
        Ok(())
    }
}
impl BuildMatrix for ast::Codata {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let ast::Codata { span, doc, name, attr: _, typ, dtors } = self;

        let mut xdata = XData {
            repr: Repr::Codata,
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
        };

        for dtor in dtors {
            xdata.dtors.insert(dtor.name.id.clone(), dtor.clone());
        }

        out.map.insert(name.id.clone(), xdata);
        Ok(())
    }
}

impl BuildMatrix for ast::Def {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &self.self_param.typ.name;
        // Only add to the matrix if the type is declared in this module
        let Some(xdata) = out.map.get_mut(&type_name.id) else { return Ok(()) };
        xdata.dtors.insert(self.name.id.clone(), self.to_dtor());

        let cases = &self.cases;

        for case in cases {
            let ast::Case { pattern, body, .. } = case;
            let key = Key { dtor: self.name.id.clone(), ctor: pattern.name.id.clone() };
            xdata.exprs.insert(key, body.clone());
        }
        Ok(())
    }
}

impl BuildMatrix for ast::Codef {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &self.typ.name;
        // Only add to the matrix if the type is declared in this module
        let Some(xdata) = out.map.get_mut(&type_name.id) else {
            return Ok(());
        };
        xdata.ctors.insert(self.name.id.clone(), self.to_ctor());

        let cases = &self.cases;

        for case in cases {
            let ast::Case { pattern, body, .. } = case;
            let key = Key { ctor: self.name.id.clone(), dtor: pattern.name.id.clone() };
            // Swap binding order to the order imposed by the matrix representation
            let body = body.as_ref().map(|body| {
                let mut ctx = LevelCtx::empty();
                // TODO: Reconsider where to swap this
                ctx.bind_iter(self.params.params.iter().map(|_| ()), |ctx| {
                    ctx.bind_iter(case.pattern.params.params.iter().map(|_| ()), |ctx| {
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
    pub fn as_data(&self, uri: &Url) -> (ast::Data, Vec<ast::Def>) {
        let XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let data = ast::Data {
            span: None,
            doc: doc.clone(),
            name: name.clone(),
            attr: Attributes::default(),
            typ: typ.clone(),
            ctors: ctors.values().cloned().collect(),
        };

        let defs = dtors
            .values()
            .map(|dtor| {
                let cases = ctors
                    .values()
                    .flat_map(|ctor| {
                        let key = Key { dtor: dtor.name.id.clone(), ctor: ctor.name.id.clone() };
                        let body = exprs.get(&key).cloned();
                        body.map(|body| ast::Case {
                            span: None,
                            pattern: ast::Pattern {
                                is_copattern: false,
                                name: IdBound {
                                    span: None,
                                    id: ctor.name.id.clone(),
                                    uri: uri.clone(),
                                },
                                params: ctor.params.instantiate(),
                            },
                            body,
                        })
                    })
                    .collect();

                ast::Def {
                    span: None,
                    doc: dtor.doc.clone(),
                    name: dtor.name.clone(),
                    attr: Attributes::default(),
                    params: dtor.params.clone(),
                    self_param: dtor.self_param.clone(),
                    ret_typ: dtor.ret_typ.clone(),
                    cases,
                }
            })
            .collect();

        (data, defs)
    }

    pub fn as_codata(&self, uri: &Url) -> (ast::Codata, Vec<ast::Codef>) {
        let XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let codata = ast::Codata {
            span: None,
            doc: doc.clone(),
            name: name.clone(),
            attr: Attributes::default(),
            typ: typ.clone(),
            dtors: dtors.values().cloned().collect(),
        };

        let codefs = ctors
            .values()
            .map(|ctor| {
                let cases = dtors
                    .values()
                    .flat_map(|dtor| {
                        let key = Key { dtor: dtor.name.id.clone(), ctor: ctor.name.id.clone() };
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
                        body.map(|body| ast::Case {
                            span: None,
                            pattern: ast::Pattern {
                                is_copattern: true,
                                name: IdBound {
                                    span: None,
                                    id: dtor.name.id.clone(),
                                    uri: uri.clone(),
                                },
                                params: dtor.params.instantiate(),
                            },
                            body,
                        })
                    })
                    .collect();

                ast::Codef {
                    span: None,
                    doc: ctor.doc.clone(),
                    name: ctor.name.clone(),
                    attr: Attributes::default(),
                    params: ctor.params.clone(),
                    typ: ctor.typ.clone(),
                    cases,
                }
            })
            .collect();

        (codata, codefs)
    }
}
