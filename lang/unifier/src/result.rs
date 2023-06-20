use std::rc::Rc;

use miette::{Diagnostic, SourceSpan};
use miette_util::ToMiette;
use thiserror::Error;

use syntax::common::*;
use syntax::ust;

use printer::PrintToString;

#[derive(Error, Diagnostic, Debug)]
pub enum UnifyError {
    #[error("{idx} occurs in {exp}")]
    #[diagnostic(code("U-001"))]
    OccursCheckFailed {
        idx: Idx,
        exp: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot unify annotated expression {exp}")]
    #[diagnostic(code("U-002"))]
    UnsupportedAnnotation {
        exp: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot automatically decide whether {lhs} and {rhs} unify")]
    #[diagnostic(code("U-003"))]
    CannotDecide {
        lhs: String,
        rhs: String,
        #[label]
        lhs_span: Option<SourceSpan>,
        #[label]
        rhs_span: Option<SourceSpan>,
    },
}

impl UnifyError {
    pub fn occurs_check_failed(idx: Idx, exp: Rc<ust::Exp>) -> Self {
        Self::OccursCheckFailed {
            idx,
            exp: exp.print_to_string(None),
            span: exp.span().to_miette(),
        }
    }

    pub fn unsupported_annotation(exp: Rc<ust::Exp>) -> Self {
        Self::UnsupportedAnnotation { exp: exp.print_to_string(None), span: exp.span().to_miette() }
    }

    pub fn cannot_decide(lhs: Rc<ust::Exp>, rhs: Rc<ust::Exp>) -> Self {
        Self::CannotDecide {
            lhs: lhs.print_to_string(None),
            rhs: rhs.print_to_string(None),
            lhs_span: lhs.span().to_miette(),
            rhs_span: rhs.span().to_miette(),
        }
    }
}
