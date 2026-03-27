use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast as js;

/// Wrap expression in parentheses.
pub fn paren_expr(e: js::Expr) -> js::Expr {
    js::Expr::Paren(js::ParenExpr { span: DUMMY_SP, expr: Box::new(e) })
}

/// Force expression by directly calling it.
pub fn force_expr(e: js::Expr) -> js::Expr {
    js::Expr::Call(js::CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: js::Callee::Expr(Box::new(e)),
        args: vec![],
        type_args: None,
    })
}
