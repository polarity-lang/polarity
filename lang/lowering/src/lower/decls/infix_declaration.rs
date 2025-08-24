use miette_util::ToMiette;
use parser::cst;

use super::super::*;

impl Lower for cst::decls::Infix {
    type Target = ast::Infix;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::decls::Infix { span, doc, attr, pattern, rhs } = self;

        let (operator, pattern_rhs) = match pattern.rhs.as_slice() {
            [x] => x,
            _ => {
                let err = LoweringError::Impossible {
                    message: "Should already have been caught when computing the symbol table."
                        .to_string(),
                    span: Some(span.to_miette()),
                };
                return Err(err.into());
            }
        };

        // Check that LHS is of the form `_ + _`
        if !(pattern.lhs.is_underscore() && pattern_rhs.is_underscore()) {
            return Err(LoweringError::InvalidInfixDeclaration {
                message:
                    "The left hand side of an infix declaration must have the form \"_ <op> _\"."
                        .to_owned(),
                span: pattern.span.to_miette(),
            }
            .into());
        }

        // Check that RHS is of the form `T(_,_)`
        if rhs.args.len() != 2 {
            return Err(LoweringError::InvalidInfixDeclaration {
                message:
                    "The right hand side of an infix declaration must take exactly two arguments."
                        .to_owned(),
                span: rhs.span.to_miette(),
            }
            .into());
        }
        if !(rhs.args[0].is_underscore() && rhs.args[1].is_underscore()) {
            return Err(LoweringError::InvalidInfixDeclaration {
                message:
                    "The right hand side of an infix declaration must have the form \"T(_,_)\"."
                        .to_owned(),
                span: rhs.span.to_miette(),
            }
            .into());
        }

        // Check that the name on the RHS is available at the location
        // of the infix declaration.
        ctx.symbol_table.lookup(&rhs.name)?;

        Ok(ast::Infix {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            attr: attr.lower(ctx)?,
            lhs: operator.id.clone(),
            rhs: rhs.name.id.clone(),
        })
    }
}
