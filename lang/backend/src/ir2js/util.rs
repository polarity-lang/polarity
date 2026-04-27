use swc_core::common::{DUMMY_SP, SyntaxContext};
use swc_core::ecma::ast as js;

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

/// Wrap expression in a thunk
pub fn thunk_expr(e: js::Expr) -> js::Expr {
    let arr = js::Expr::Arrow(js::ArrowExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        params: vec![],
        body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(e))),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
    });
    paren_expr(arr)
}

/// Wrap expression in a thunk
pub fn thunk_block(block: Vec<js::Stmt>) -> js::Expr {
    let arr = js::Expr::Arrow(js::ArrowExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        params: vec![],
        body: Box::new(js::BlockStmtOrExpr::BlockStmt(js::BlockStmt {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            stmts: block,
        })),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
    });
    paren_expr(arr)
}
