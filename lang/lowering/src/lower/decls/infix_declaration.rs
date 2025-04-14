use miette_util::ToMiette;
use parser::cst;

use super::super::*;

impl Lower for cst::decls::Infix {
    type Target = ast::Infix;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Infix { span, doc, lhs, rhs } = self;

        // Check that LHS is of the form `_ + _`
        if !(lhs.lhs.is_underscore() && lhs.rhs.is_underscore()) {
            return Err(LoweringError::InvalidInfixDeclaration {
                message: "The left hand side of an infix declaration must have the form \"_ + _\"."
                    .to_owned(),
                span: lhs.span.to_miette(),
            });
        }

        // Check that RHS is of the form `T(_,_)`
        if rhs.args.len() != 2 {
            return Err(LoweringError::InvalidInfixDeclaration {
                message:
                    "The right hand side of an infix declaration must take exactly two arguments."
                        .to_owned(),
                span: rhs.span.to_miette(),
            });
        }
        if !(rhs.args[0].is_underscore() && rhs.args[1].is_underscore()) {
            return Err(LoweringError::InvalidInfixDeclaration {
                message:
                    "The right hand side of an infix declaration must have the form \"T(_,_)\"."
                        .to_owned(),
                span: rhs.span.to_miette(),
            });
        }

        // Check that the name on the RHS is available at the location
        // of the infix declaration.
        ctx.symbol_table.lookup(&rhs.name)?;

        Ok(ast::Infix {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            attr: Default::default(),
            lhs: lhs.operator.id.clone(),
            rhs: rhs.name.id.clone(),
        })
    }
}
