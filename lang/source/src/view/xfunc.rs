use codespan::Span;
use printer::PrintToStringInCtx;
use renaming::Rename;

use data::HashMap;
use syntax::ust;

use super::DatabaseView;

pub struct Xfunc {
    pub title: String,
    pub edits: Vec<Edit>,
}

pub struct Edit {
    pub span: Span,
    pub text: String,
}

impl<'a> DatabaseView<'a> {
    pub fn xfunc(&self, type_name: &str) -> Result<Xfunc, String> {
        let prg = self.ust().map_err(|err| format!("{}", err))?;
        let mat = xfunc::as_matrix(&prg);
        let type_span = mat.map[type_name].info.span.unwrap();
        let impl_span = mat.map[type_name].impl_block.as_ref().and_then(|block| block.info.span);
        let title;

        let (type_text, impl_text): (String, String) = match xfunc::repr(&mat, type_name) {
            syntax::matrix::Repr::Data => {
                title = format!("Refunctionalize {}", type_name);
                let (codata, dtors, codefs) = xfunc::as_codata(&mat, type_name);

                let impl_block = ust::Impl {
                    info: ust::Info::empty(),
                    name: type_name.to_owned(),
                    defs: codefs.iter().map(|def| def.name.clone()).collect(),
                };

                let mut order = vec![codata.name.clone()];
                order.extend(codefs.iter().map(|def| def.name.clone()));

                let mut map = HashMap::default();
                map.insert(codata.name.clone(), ust::Decl::Codata(codata.clone()));
                map.extend(codefs.into_iter().map(|def| (def.name.clone(), ust::Decl::Codef(def))));
                map.extend(
                    dtors.into_iter().map(|dtor| (dtor.name.clone(), ust::Decl::Dtor(dtor))),
                );

                let decls = ust::Decls { map, order };

                let codata = codata.rename();
                let decls = decls.rename();
                let impl_block = impl_block.rename();

                let codata_string = codata.print_to_string_in_ctx(&decls);
                let impl_string = impl_block.print_to_string_in_ctx(&decls);

                (codata_string, impl_string)
            }
            syntax::matrix::Repr::Codata => {
                title = format!("Defunctionalize {}", type_name);
                let (data, ctors, defs) = xfunc::as_data(&mat, type_name);

                let impl_block = ust::Impl {
                    info: ust::Info::empty(),
                    name: type_name.to_owned(),
                    defs: defs.iter().map(|def| def.name.clone()).collect(),
                };

                let mut order = vec![data.name.clone()];
                order.extend(defs.iter().map(|def| def.name.clone()));

                let mut map = HashMap::default();
                map.insert(data.name.clone(), ust::Decl::Data(data.clone()));
                map.extend(defs.into_iter().map(|def| (def.name.clone(), ust::Decl::Def(def))));
                map.extend(
                    ctors.into_iter().map(|ctor| (ctor.name.clone(), ust::Decl::Ctor(ctor))),
                );

                let decls = ust::Decls { map, order };

                let data = data.rename();
                let decls = decls.rename();
                let impl_block = impl_block.rename();

                let data_string = data.print_to_string_in_ctx(&decls);
                let impl_string = impl_block.print_to_string_in_ctx(&decls);

                (data_string, impl_string)
            }
        };

        let edits = if let Some(impl_span) = impl_span {
            vec![
                Edit { span: type_span, text: type_text },
                Edit { span: impl_span, text: impl_text },
            ]
        } else {
            let mut type_and_impl_text = type_text;
            type_and_impl_text.push_str("\n\n");
            type_and_impl_text.push_str(&impl_text);
            vec![Edit { span: type_span, text: type_and_impl_text }]
        };

        Ok(Xfunc { title, edits })
    }
}
