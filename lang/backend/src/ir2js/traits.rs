use swc_ecma_ast as js;

use crate::result::BackendError;

pub trait ToJSExpr {
    fn to_js_expr(&self) -> Result<js::Expr, BackendError>;
}

pub trait ToJSStmt {
    fn to_js_stmt(&self) -> Result<js::Stmt, BackendError>;
}
