use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast as js;

use crate::ir;
use crate::ir::ident::Ident;
use crate::result::BackendResult;

use super::tokens::*;
use super::traits::{ToJSExpr, ToJSStmt};

impl ir::Module {
    pub fn to_js_module(&self) -> BackendResult<js::Module> {
        let Self { uri: _, use_decls: _, def_decls, codef_decls, let_decls } = self;
        let mut body = vec![];

        for let_decl in let_decls {
            let stmt = let_decl.to_js_stmt()?;
            body.push(js::ModuleItem::Stmt(stmt));
        }

        for def_decl in def_decls {
            let stmt = def_decl.to_js_stmt()?;
            body.push(js::ModuleItem::Stmt(stmt));
        }

        for codef_decl in codef_decls {
            let stmt = codef_decl.to_js_stmt()?;
            body.push(js::ModuleItem::Stmt(stmt));
        }

        Ok(js::Module { span: DUMMY_SP, body, shebang: None })
    }
}

/// Input:
///
/// ```text
/// let name(params) { body }
/// ```
///
/// Output:
///
/// ```js
/// function name(〚 params 〛) {
///     return 〚 body 〛;
/// }
impl ToJSStmt for ir::Let {
    fn to_js_stmt(&self) -> BackendResult<js::Stmt> {
        let Self { name, params, body } = self;

        let params = params_to_js_params(params);
        let body_expr = body.to_js_expr()?;

        Ok(js::Stmt::Decl(js::Decl::Fn(js::FnDecl {
            ident: js::Ident::new(name.to_string().into(), DUMMY_SP, SyntaxContext::empty()),
            declare: false,
            function: Box::new(js::Function {
                params,
                decorators: vec![],
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                body: Some(js::BlockStmt {
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::empty(),
                    stmts: vec![js::Stmt::Return(js::ReturnStmt {
                        span: DUMMY_SP,
                        arg: Some(Box::new(body_expr)),
                    })],
                }),
                is_generator: false,
                is_async: false,
                type_params: None,
                return_type: None,
            }),
        })))
    }
}

/// Input:
///
/// ```text
/// def name(params) {
///     cases
/// }
/// ```
///
/// Output:
///
/// ```js
/// function name(__self, 〚 params 〛) {
///     switch (__self.tag) {
///         〚 cases 〛
///     }
/// }
/// ```
impl ToJSStmt for ir::Def {
    fn to_js_stmt(&self) -> BackendResult<js::Stmt> {
        let Self { name, params, cases } = self;

        // Generate self parameter
        let mut all_params = vec![js::Param {
            span: DUMMY_SP,
            decorators: vec![],
            pat: js::Pat::Ident(js::BindingIdent {
                id: js::Ident::new(SELF_PARAM_NAME.into(), DUMMY_SP, SyntaxContext::empty()),
                type_ann: None,
            }),
        }];
        all_params.extend(params_to_js_params(params));

        // Generate switch statement on __self.tag
        let cases = cases
            .iter()
            .map(|case| case.to_js_switch_case(SELF_PARAM_NAME))
            .collect::<Result<Vec<_>, _>>()?;

        let body_stmts = vec![js::Stmt::Switch(js::SwitchStmt {
            span: DUMMY_SP,
            discriminant: Box::new(js::Expr::Member(js::MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(js::Expr::Ident(js::Ident::new(
                    SELF_PARAM_NAME.into(),
                    DUMMY_SP,
                    SyntaxContext::empty(),
                ))),
                prop: js::MemberProp::Ident(js::IdentName { span: DUMMY_SP, sym: CTOR_TAG.into() }),
            })),
            cases,
        })];

        Ok(js::Stmt::Decl(js::Decl::Fn(js::FnDecl {
            ident: js::Ident::new(name.to_string().into(), DUMMY_SP, SyntaxContext::empty()),
            declare: false,
            function: Box::new(js::Function {
                params: all_params,
                decorators: vec![],
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                body: Some(js::BlockStmt {
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::empty(),
                    stmts: body_stmts,
                }),
                is_generator: false,
                is_async: false,
                type_params: None,
                return_type: None,
            }),
        })))
    }
}

/// Input:
///
/// ```text
/// codef name(params) {
///     cocases
/// }
/// ```
///
/// Output:
///
/// ```js
/// function name(〚 params 〛) {
///     return {
///         〚 cocases 〛
///     };
/// }
/// ```
impl ToJSStmt for ir::Codef {
    fn to_js_stmt(&self) -> BackendResult<js::Stmt> {
        let Self { name, params, cases } = self;

        let params = params_to_js_params(params);

        let props =
            cases.iter().map(|case| case.to_js_object_method()).collect::<Result<Vec<_>, _>>()?;

        let return_stmt = js::Stmt::Return(js::ReturnStmt {
            span: DUMMY_SP,
            arg: Some(Box::new(js::Expr::Object(js::ObjectLit { span: DUMMY_SP, props }))),
        });

        Ok(js::Stmt::Decl(js::Decl::Fn(js::FnDecl {
            ident: js::Ident::new(name.to_string().into(), DUMMY_SP, SyntaxContext::empty()),
            declare: false,
            function: Box::new(js::Function {
                params,
                decorators: vec![],
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                body: Some(js::BlockStmt {
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::empty(),
                    stmts: vec![return_stmt],
                }),
                is_generator: false,
                is_async: false,
                type_params: None,
                return_type: None,
            }),
        })))
    }
}

fn params_to_js_params(params: &[Ident]) -> Vec<js::Param> {
    params
        .iter()
        .map(|p| js::Param {
            span: DUMMY_SP,
            decorators: vec![],
            pat: js::Pat::Ident(js::BindingIdent {
                id: js::Ident::new(p.to_string().into(), DUMMY_SP, SyntaxContext::empty()),
                type_ann: None,
            }),
        })
        .collect()
}
