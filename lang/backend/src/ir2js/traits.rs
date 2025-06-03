use swc_ecma_ast as js;

use crate::result::BackendResult;

pub trait ToJSExpr {
    fn to_js_expr(&self) -> BackendResult<js::Expr>;
}

pub trait ToJSStmt {
    fn to_js_stmt(&self) -> BackendResult<js::Stmt>;
}
