use url::Url;

use polarity_lang_ast::ctx::{BindContext, LevelCtx};
use polarity_lang_ast::{self, HashMap, IdBound, SwapWithCtx};
use polarity_lang_ast::{Attributes, DocComment};
use polarity_lang_miette_util::codespan::Span;

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
    pub name: polarity_lang_ast::IdBind,
    pub typ: Box<polarity_lang_ast::Telescope>,
    pub ctors: HashMap<String, polarity_lang_ast::Ctor>,
    pub dtors: HashMap<String, polarity_lang_ast::Dtor>,
    pub exprs: HashMap<Key, Option<Box<polarity_lang_ast::Exp>>>,
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
pub fn build(prg: &polarity_lang_ast::Module) -> Result<Prg, XfuncError> {
    let mut out = Prg { map: HashMap::default(), uri: prg.uri.clone() };
    prg.build_matrix(&mut out)?;
    Ok(out)
}

pub trait BuildMatrix {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError>;
}

impl BuildMatrix for polarity_lang_ast::Module {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let polarity_lang_ast::Module { decls, .. } = self;

        for decl in decls {
            match decl {
                polarity_lang_ast::Decl::Data(data) => data.build_matrix(out),
                polarity_lang_ast::Decl::Codata(codata) => codata.build_matrix(out),
                _ => Ok(()),
            }?
        }

        for decl in decls {
            match decl {
                polarity_lang_ast::Decl::Def(def) => def.build_matrix(out),
                polarity_lang_ast::Decl::Codef(codef) => codef.build_matrix(out),
                _ => Ok(()),
            }?
        }

        Ok(())
    }
}

impl BuildMatrix for polarity_lang_ast::Data {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let polarity_lang_ast::Data { span, doc, name, attr: _, typ, ctors } = self;

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
impl BuildMatrix for polarity_lang_ast::Codata {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let polarity_lang_ast::Codata { span, doc, name, attr: _, typ, dtors } = self;

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

impl BuildMatrix for polarity_lang_ast::Def {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &self.self_param.typ.name;
        // Only add to the matrix if the type is declared in this module
        let Some(xdata) = out.map.get_mut(&type_name.id) else { return Ok(()) };
        xdata.dtors.insert(self.name.id.clone(), self.to_dtor());

        let cases = &self.cases;

        for case in cases {
            let polarity_lang_ast::Case { pattern, body, .. } = case;
            let key = Key { dtor: self.name.id.clone(), ctor: pattern.name.id.clone() };
            xdata.exprs.insert(key, body.clone());
        }
        Ok(())
    }
}

impl BuildMatrix for polarity_lang_ast::Codef {
    fn build_matrix(&self, out: &mut Prg) -> Result<(), XfuncError> {
        let type_name = &self.typ.name;
        // Only add to the matrix if the type is declared in this module
        let Some(xdata) = out.map.get_mut(&type_name.id) else {
            return Ok(());
        };
        xdata.ctors.insert(self.name.id.clone(), self.to_ctor());

        let cases = &self.cases;

        for case in cases {
            let polarity_lang_ast::Case { pattern, body, .. } = case;
            let key = Key { ctor: self.name.id.clone(), dtor: pattern.name.id.clone() };
            // Swap binding order to the order imposed by the matrix representation
            let body = body.as_ref().map(|body| {
                let mut ctx = LevelCtx::empty();
                // TODO: Reconsider where to swap this
                ctx.bind_iter(self.params.params.iter(), |ctx| {
                    ctx.bind_iter(case.pattern.params.params.iter(), |ctx| {
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
    pub fn as_data(&self, uri: &Url) -> (polarity_lang_ast::Data, Vec<polarity_lang_ast::Def>) {
        let XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let data = polarity_lang_ast::Data {
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
                        body.map(|body| polarity_lang_ast::Case {
                            span: None,
                            pattern: polarity_lang_ast::Pattern {
                                span: None,
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

                polarity_lang_ast::Def {
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

    pub fn as_codata(
        &self,
        uri: &Url,
    ) -> (polarity_lang_ast::Codata, Vec<polarity_lang_ast::Codef>) {
        let XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let codata = polarity_lang_ast::Codata {
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
                        body.map(|body| polarity_lang_ast::Case {
                            span: None,
                            pattern: polarity_lang_ast::Pattern {
                                span: None,
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

                polarity_lang_ast::Codef {
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
