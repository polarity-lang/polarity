use url::Url;

use polarity_lang_ast::*;
use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::Print;
use polarity_lang_transformations::LiftResult;

use crate::{DriverError, Edit, database::Database};

impl Database {
    pub async fn lift(&mut self, uri: &Url, type_name: &str) -> Result<Vec<Edit>, crate::Error> {
        let prg = self.ast(uri).await?;

        let type_span = prg
            .decls
            .iter()
            .find(|decl| match decl.ident() {
                None => false,
                Some(id) => id.id == type_name,
            })
            .and_then(|x| x.span())
            .ok_or(DriverError::Impossible(format!("Could not resolve {type_name}")))?;

        let LiftResult { module: prg, modified_decls, new_decls } =
            polarity_lang_transformations::lift(prg, type_name);

        let edits = generate_edits(type_span, &prg, modified_decls, new_decls);

        Ok(edits)
    }
}

fn generate_edits(
    type_span: Span,
    module: &polarity_lang_ast::Module,
    modified_decls: HashSet<IdBind>,
    new_decls: HashSet<IdBind>,
) -> Vec<Edit> {
    // If there are no modifications, no local (co)matches have been lifted.
    if new_decls.is_empty() {
        assert!(modified_decls.is_empty());
        return vec![];
    }

    let new_decls = module
        .decls
        .iter()
        .filter(|decl| match decl.ident() {
            None => false,
            Some(id) => new_decls.contains(id),
        })
        .cloned()
        .collect();

    let new_items = Module {
        uri: module.uri.clone(),
        // Use declarations don't change, and we are only printing an excerpt of the module
        use_decls: vec![],
        decls: new_decls,
        meta_vars: module.meta_vars.clone(),
    };

    let mut text = "\n\n".to_owned();
    text.push_str(&new_items.print_to_string(None));

    let span = Span { start: type_span.end, end: type_span.end };

    let mut edits = vec![Edit { span, text }];

    for name in modified_decls {
        let decl = module
            .decls
            .iter()
            .find(|d| match d.ident() {
                None => false,
                Some(id) => id == &name,
            })
            .unwrap();
        let span = decl.span().unwrap();
        let text = decl.print_to_string(None);
        edits.push(Edit { span, text });
    }

    edits
}
