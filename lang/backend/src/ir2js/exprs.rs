//! JavaScript code generation for IR expressions using SWC AST.

use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast as js;

use crate::ir;
use crate::result::BackendError;

use super::traits::ToJSExpr;

impl ToJSExpr for ir::Exp {
    fn to_js_expr(&self) -> Result<js::Expr, BackendError> {
        match self {
            ir::Exp::Variable(var) => var.to_js_expr(),
            ir::Exp::CtorCall(call) => call.to_js_ctor_record(),
            ir::Exp::CodefCall(call) => call.to_js_function_call(),
            ir::Exp::LetCall(call) => call.to_js_function_call(),
            ir::Exp::DtorCall(dot_call) => dot_call.to_js_record_member_call(),
            ir::Exp::DefCall(dot_call) => dot_call.to_js_function_call_with_self(),
            ir::Exp::LocalMatch(local_match) => local_match.to_js_expr(),
            ir::Exp::LocalComatch(local_comatch) => local_comatch.to_js_expr(),
            ir::Exp::Panic(panic) => panic.to_js_expr(),
            ir::Exp::ExternCall(_) => todo!(),
            ir::Exp::LocalLet(_) => todo!(),
            ir::Exp::Literal(_) => todo!(),
            ir::Exp::ZST => Ok(js::Expr::Ident(js::Ident::new(
                "undefined".into(),
                DUMMY_SP,
                SyntaxContext::empty(),
            ))),
        }
    }
}

/// Input:
///
/// ```
/// variable_name
/// ```
///
/// Output:
///
/// ```js
/// variable_name
/// ```
impl ToJSExpr for ir::Variable {
    fn to_js_expr(&self) -> Result<js::Expr, BackendError> {
        let Self { name } = self;
        let name = name.clone();
        Ok(js::Expr::Ident(js::Ident::new(name.into(), DUMMY_SP, SyntaxContext::empty())))
    }
}

impl ir::Call {
    /// Input:
    ///
    /// ```
    /// CtorName(arg1, arg2)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// { tag: "CtorName", args: [arg1, arg2] }
    /// ```
    fn to_js_ctor_record(&self) -> Result<js::Expr, BackendError> {
        let Self { name, module_uri: _, args } = self;
        let ctor_name = name.clone();

        // Always include args array, even if empty
        let args_exprs: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| {
                arg.to_js_expr()
                    .map(|expr| Some(js::ExprOrSpread { spread: None, expr: Box::new(expr) }))
            })
            .collect();

        let props = vec![
            js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(js::KeyValueProp {
                key: js::PropName::Ident(js::IdentName { span: DUMMY_SP, sym: "tag".into() }),
                value: Box::new(js::Expr::Lit(js::Lit::Str(js::Str {
                    span: DUMMY_SP,
                    value: ctor_name.into(),
                    raw: None,
                }))),
            }))),
            js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(js::KeyValueProp {
                key: js::PropName::Ident(js::IdentName { span: DUMMY_SP, sym: "args".into() }),
                value: Box::new(js::Expr::Array(js::ArrayLit {
                    span: DUMMY_SP,
                    elems: args_exprs?,
                })),
            }))),
        ];

        Ok(js::Expr::Object(js::ObjectLit { span: DUMMY_SP, props }))
    }

    /// Input:
    ///
    /// ```
    /// f(arg1, arg2)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// f(arg1, arg2)
    /// ```
    fn to_js_function_call(&self) -> Result<js::Expr, BackendError> {
        let Self { name, module_uri: _, args } = self;
        let codef_name = name.clone();
        let args_exprs: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| {
                arg.to_js_expr().map(|expr| js::ExprOrSpread { spread: None, expr: Box::new(expr) })
            })
            .collect();

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Ident(js::Ident::new(
                codef_name.into(),
                DUMMY_SP,
                SyntaxContext::empty(),
            )))),
            args: args_exprs?,
            type_args: None,
        }))
    }
}

impl ir::DotCall {
    /// Input:
    ///
    /// ```
    /// exp.dtor_name(arg1, arg2)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// exp.dtor_name(arg1, arg2)
    /// ```
    fn to_js_record_member_call(&self) -> Result<js::Expr, BackendError> {
        let Self { exp, module_uri: _, name, args } = self;
        let obj_expr = exp.to_js_expr()?;
        let dtor_name = name.clone();
        let args_exprs: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| {
                arg.to_js_expr().map(|expr| js::ExprOrSpread { spread: None, expr: Box::new(expr) })
            })
            .collect();

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Member(js::MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(obj_expr),
                prop: js::MemberProp::Ident(js::IdentName {
                    span: DUMMY_SP,
                    sym: dtor_name.into(),
                }),
            }))),
            args: args_exprs?,
            type_args: None,
        }))
    }

    /// Input:
    ///
    /// ```
    /// exp.def_name(arg1, arg2)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// def_name(obj, arg1, arg2)
    /// ```
    fn to_js_function_call_with_self(&self) -> Result<js::Expr, BackendError> {
        let Self { exp, module_uri: _, name, args } = self;
        let obj_expr = exp.to_js_expr()?;
        let def_name = name.clone();

        let mut all_args =
            vec![js::ExprOrSpread { spread: None, expr: Box::new(obj_expr.clone()) }];
        let args_exprs: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| {
                arg.to_js_expr().map(|expr| js::ExprOrSpread { spread: None, expr: Box::new(expr) })
            })
            .collect();
        all_args.extend(args_exprs?);

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Ident(js::Ident::new(
                def_name.into(),
                DUMMY_SP,
                SyntaxContext::empty(),
            )))),
            args: all_args,
            type_args: None,
        }))
    }
}

/// Input:
///
/// ```
/// match expr {
///     C1(x, y) => body1,
///     C2(z) => body2,
/// }
/// ```
///
/// Output:
///
/// ```js
/// (() => {
///     const __scrutinee = expr;
///     switch (__scrutinee.tag) {
///         case "C1":
///             const x = __scrutinee.args[0];
///             const y = __scrutinee.args[1];
///             return body1;
///         case "C2":
///             const z = __scrutinee.args[0];
///             return body2;
///     }
/// })()
/// ```
impl ToJSExpr for ir::LocalMatch {
    fn to_js_expr(&self) -> Result<js::Expr, BackendError> {
        let Self { on_exp, cases } = self;
        let on_expr = on_exp.to_js_expr()?;

        // Generate IIFE: (() => { const __scrutinee = ...; switch (...) { ... } })()
        let match_var = js::VarDeclarator {
            span: DUMMY_SP,
            name: js::Pat::Ident(js::BindingIdent {
                id: js::Ident::new("__scrutinee".into(), DUMMY_SP, SyntaxContext::empty()),
                type_ann: None,
            }),
            init: Some(Box::new(on_expr)),
            definite: false,
        };

        let var_decl = js::Stmt::Decl(js::Decl::Var(Box::new(js::VarDecl {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            kind: js::VarDeclKind::Const,
            declare: false,
            decls: vec![match_var],
        })));

        let cases = cases
            .iter()
            .map(|case| case.to_js_switch_case("__scrutinee"))
            .collect::<Result<Vec<_>, _>>()?;

        let switch_stmt = js::Stmt::Switch(js::SwitchStmt {
            span: DUMMY_SP,
            discriminant: Box::new(js::Expr::Member(js::MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(js::Expr::Ident(js::Ident::new(
                    "__scrutinee".into(),
                    DUMMY_SP,
                    SyntaxContext::empty(),
                ))),
                prop: js::MemberProp::Ident(js::IdentName { span: DUMMY_SP, sym: "tag".into() }),
            })),
            cases,
        });

        let arrow_fn = js::ArrowExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            params: vec![],
            body: Box::new(js::BlockStmtOrExpr::BlockStmt(js::BlockStmt {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                stmts: vec![var_decl, switch_stmt],
            })),
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
        };

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Paren(js::ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(js::Expr::Arrow(arrow_fn)),
            }))),
            args: vec![],
            type_args: None,
        }))
    }
}

/// Input:
///
/// ```
/// comatch {
///     .d1(x, y) => body1,
///     .d2(z) => body2,
/// }
/// ```
///
/// Output:
///
/// ```js
/// {
///     d1: function(x, y) { return body1; },
///     d2: function(z) { return body2; }
/// }
/// ```
impl ToJSExpr for ir::LocalComatch {
    fn to_js_expr(&self) -> Result<js::Expr, BackendError> {
        let Self { cases } = self;
        let props: Result<Vec<_>, _> =
            cases.iter().map(|case| case.to_js_object_method()).collect();

        Ok(js::Expr::Object(js::ObjectLit { span: DUMMY_SP, props: props? }))
    }
}

/// Input:
///
/// ```
/// panic!("error message")
/// ```
///
/// Output:
///
/// ```js
/// (() => { throw new Error("error message"); })()
/// ```
impl ToJSExpr for ir::Panic {
    fn to_js_expr(&self) -> Result<js::Expr, BackendError> {
        let Self { message } = self;
        let message = message.clone();

        // Generate IIFE: (() => { throw new Error("message"); })()
        let throw_stmt = js::Stmt::Throw(js::ThrowStmt {
            span: DUMMY_SP,
            arg: Box::new(js::Expr::New(js::NewExpr {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                callee: Box::new(js::Expr::Ident(js::Ident::new(
                    "Error".into(),
                    DUMMY_SP,
                    SyntaxContext::empty(),
                ))),
                args: Some(vec![js::ExprOrSpread {
                    spread: None,
                    expr: Box::new(js::Expr::Lit(js::Lit::Str(js::Str {
                        span: DUMMY_SP,
                        value: message.into(),
                        raw: None,
                    }))),
                }]),
                type_args: None,
            })),
        });

        let arrow_fn = js::ArrowExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            params: vec![],
            body: Box::new(js::BlockStmtOrExpr::BlockStmt(js::BlockStmt {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                stmts: vec![throw_stmt],
            })),
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
        };

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Paren(js::ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(js::Expr::Arrow(arrow_fn)),
            }))),
            args: vec![],
            type_args: None,
        }))
    }
}

impl ir::Case {
    pub fn to_js_switch_case(&self, scrutinee_name: &str) -> Result<js::SwitchCase, BackendError> {
        let Self { pattern, body } = self;
        let pattern_name = pattern.name.clone();
        let test = js::Expr::Lit(js::Lit::Str(js::Str {
            span: DUMMY_SP,
            value: pattern_name.into(),
            raw: None,
        }));

        let mut stmts = vec![];

        // Bind pattern parameters
        for (i, param) in pattern.params.iter().enumerate() {
            let param_name = param.clone();
            let var_decl = js::Stmt::Decl(js::Decl::Var(Box::new(js::VarDecl {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                kind: js::VarDeclKind::Const,
                declare: false,
                decls: vec![js::VarDeclarator {
                    span: DUMMY_SP,
                    name: js::Pat::Ident(js::BindingIdent {
                        id: js::Ident::new(param_name.into(), DUMMY_SP, SyntaxContext::empty()),
                        type_ann: None,
                    }),
                    init: Some(Box::new(js::Expr::Member(js::MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(js::Expr::Member(js::MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(js::Expr::Ident(js::Ident::new(
                                scrutinee_name.into(),
                                DUMMY_SP,
                                SyntaxContext::empty(),
                            ))),
                            prop: js::MemberProp::Ident(js::IdentName {
                                span: DUMMY_SP,
                                sym: "args".into(),
                            }),
                        })),
                        prop: js::MemberProp::Computed(js::ComputedPropName {
                            span: DUMMY_SP,
                            expr: Box::new(js::Expr::Lit(js::Lit::Num(js::Number {
                                span: DUMMY_SP,
                                value: i as f64,
                                raw: None,
                            }))),
                        }),
                    }))),
                    definite: false,
                }],
            })));
            stmts.push(var_decl);
        }

        // Generate case body
        let body_expr = body.to_js_expr()?;
        stmts.push(js::Stmt::Return(js::ReturnStmt {
            span: DUMMY_SP,
            arg: Some(Box::new(body_expr)),
        }));

        Ok(js::SwitchCase { span: DUMMY_SP, test: Some(Box::new(test)), cons: stmts })
    }

    pub fn to_js_object_method(&self) -> Result<js::PropOrSpread, BackendError> {
        let Self { pattern, body } = self;
        let method_name = pattern.name.clone();
        let params: Vec<_> = pattern
            .params
            .iter()
            .map(|p| js::Param {
                span: DUMMY_SP,
                decorators: vec![],
                pat: js::Pat::Ident(js::BindingIdent {
                    id: js::Ident::new(p.clone().into(), DUMMY_SP, SyntaxContext::empty()),
                    type_ann: None,
                }),
            })
            .collect();

        let body_expr = body.to_js_expr()?;
        let body_stmt =
            js::Stmt::Return(js::ReturnStmt { span: DUMMY_SP, arg: Some(Box::new(body_expr)) });

        Ok(js::PropOrSpread::Prop(Box::new(js::Prop::Method(js::MethodProp {
            key: js::PropName::Ident(js::IdentName { span: DUMMY_SP, sym: method_name.into() }),
            function: Box::new(js::Function {
                params,
                decorators: vec![],
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                body: Some(js::BlockStmt {
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::empty(),
                    stmts: vec![body_stmt],
                }),
                is_generator: false,
                is_async: false,
                type_params: None,
                return_type: None,
            }),
        }))))
    }
}
