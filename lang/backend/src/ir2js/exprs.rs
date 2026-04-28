//! JavaScript code generation for IR expressions using SWC AST.

use swc_core::common::{DUMMY_SP, SyntaxContext};
use swc_core::ecma::ast as js;
use swc_core::quote_expr;

use crate::ir;
use crate::ir2js::traits::ToJSStmt;
use crate::ir2js::util::{force_expr, paren_expr, thunk_block};
use crate::result::BackendResult;

use super::tokens::*;
use super::traits::ToJSExpr;

impl ToJSExpr for ir::Exp {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        match self {
            ir::Exp::Variable(var) => var.to_js_expr(),
            ir::Exp::CtorCall(call) => call.to_js_ctor_record(),
            ir::Exp::CodefCall(call) => call.to_js_function_call(),
            ir::Exp::LetCall(call) => call.to_js_function_call(),
            ir::Exp::ExternCall(call) => call.to_js_extern_function_call(),
            ir::Exp::DtorCall(dot_call) => dot_call.to_js_record_member_call(),
            ir::Exp::DefCall(dot_call) => dot_call.to_js_function_call_with_self(),
            ir::Exp::LocalMatch(local_match) => local_match.to_js_expr(),
            ir::Exp::LocalComatch(local_comatch) => local_comatch.to_js_expr(),
            ir::Exp::Panic(panic) => panic.to_js_expr(),
            ir::Exp::LocalLet(local_let) => local_let.to_js_expr(),
            ir::Exp::DoBlock(do_block) => do_block.to_js_expr(),
            ir::Exp::Literal(lit) => lit.to_js_expr(),
            ir::Exp::ZST => Ok(paren_expr(*js::Expr::undefined(DUMMY_SP))),
        }
    }
}

/// Input:
///
/// ```text
/// x
/// ```
///
/// Output:
///
/// ```js
/// x
/// ```
impl ToJSExpr for ir::Variable {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        let Self { name } = self;
        Ok(js::Expr::Ident(name.to_string().into()))
    }
}

impl ir::Call {
    /// Input:
    ///
    /// ```text
    /// C(x, y)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// { tag: "C", args: [〚 x, y 〛] }
    /// ```
    fn to_js_ctor_record(&self) -> BackendResult<js::Expr> {
        let Self { name, module_uri: _, args } = self;
        let args = args_to_js_array(args)?;

        let props = vec![
            js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(js::KeyValueProp {
                key: js::PropName::Ident(CTOR_TAG.into()),
                value: Box::new(js_str_lit(name.to_string())),
            }))),
            js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(js::KeyValueProp {
                key: js::PropName::Ident(CTOR_ARGS.into()),
                value: Box::new(js::Expr::Array(args)),
            }))),
        ];

        Ok(js::Expr::Object(js::ObjectLit { span: DUMMY_SP, props }))
    }

    /// Input:
    ///
    /// ```text
    /// f(x, y)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// f(〚 x, y 〛)
    /// ```
    fn to_js_function_call(&self) -> BackendResult<js::Expr> {
        let Self { name, module_uri: _, args } = self;
        let args = args_to_js_exprs(args)?;

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Ident(name.to_string().into()))),
            args,
            type_args: None,
        }))
    }

    /// Handle builtin extern calls and pass the rest to [Self::to_js_function_call].
    fn to_js_extern_function_call(&self) -> BackendResult<js::Expr> {
        let Self { name, module_uri: _, args } = self;
        let args = args_to_js_exprs(args)?.into_iter().map(|arg| *arg.expr).collect();

        match extern_call_to_js_expr(name.to_string().as_str(), args) {
            Some(expr) => Ok(*expr),
            None => self.to_js_function_call(),
        }
    }
}

impl ir::DotCall {
    /// Input:
    ///
    /// ```text
    /// exp.d(x, y)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// 〚 exp 〛.d(〚 x, y 〛)
    /// ```
    fn to_js_record_member_call(&self) -> BackendResult<js::Expr> {
        let Self { exp, module_uri: _, name, args } = self;
        let obj_expr = exp.to_js_expr()?;
        let args = args_to_js_exprs(args)?;

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Member(js::MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(obj_expr),
                prop: js::MemberProp::Ident(name.to_string().into()),
            }))),
            args,
            type_args: None,
        }))
    }

    /// Input:
    ///
    /// ```text
    /// e.def_name(x, y)
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// def_name(〚 e 〛, 〚 x, y 〛)
    /// ```
    fn to_js_function_call_with_self(&self) -> BackendResult<js::Expr> {
        let Self { exp, module_uri: _, name, args } = self;
        let exp = exp.to_js_expr()?;

        let mut all_args = vec![js::ExprOrSpread { spread: None, expr: Box::new(exp.clone()) }];
        all_args.extend(args_to_js_exprs(args)?);

        Ok(js::Expr::Call(js::CallExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: js::Callee::Expr(Box::new(js::Expr::Ident(name.to_string().into()))),
            args: all_args,
            type_args: None,
        }))
    }
}

/// Input:
///
/// ```text
/// on_exp.match {
///     cases
/// }
/// ```
///
/// Output:
///
/// ```js
/// (() => {
///     const __scrutinee = 〚 on_exp 〛;
///     switch (__scrutinee.tag) {
///         〚 cases 〛
///     }
/// })()
/// ```
impl ToJSExpr for ir::LocalMatch {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        let Self { on_exp, cases } = self;
        let on_expr = on_exp.to_js_expr()?;

        // Generate IIFE: (() => { const __scrutinee = ...; switch (...) { ... } })()
        let match_var = js::VarDeclarator {
            span: DUMMY_SP,
            name: js::Pat::Ident(js::BindingIdent {
                id: js::Ident::new(SCRUTINEE_NAME.into(), DUMMY_SP, SyntaxContext::empty()),
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
            .map(|case| case.to_js_switch_case(SCRUTINEE_NAME))
            .collect::<Result<Vec<_>, _>>()?;

        let switch_stmt = js::Stmt::Switch(js::SwitchStmt {
            span: DUMMY_SP,
            discriminant: Box::new(js::Expr::Member(js::MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(js::Expr::Ident(js::Ident::new(
                    SCRUTINEE_NAME.into(),
                    DUMMY_SP,
                    SyntaxContext::empty(),
                ))),
                prop: js::MemberProp::Ident(js::IdentName { span: DUMMY_SP, sym: CTOR_TAG.into() }),
            })),
            cases,
        });

        let thunk = thunk_block(vec![var_decl, switch_stmt]);
        Ok(force_expr(thunk))
    }
}

/// Input:
///
/// ```text
/// comatch {
///     cocases
/// }
/// ```
///
/// Output:
///
/// ```js
/// {
///     〚 cocases 〛,
/// }
/// ```
impl ToJSExpr for ir::LocalComatch {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        let Self { cases } = self;
        let props: Result<Vec<_>, _> =
            cases.iter().map(|case| case.to_js_object_method()).collect();

        Ok(js::Expr::Object(js::ObjectLit { span: DUMMY_SP, props: props? }))
    }
}

/// Input:
///
/// ```text
/// panic!("error message")
/// ```
///
/// Output:
///
/// ```js
/// (() => { throw new Error("error message"); })()
/// ```
impl ToJSExpr for ir::Panic {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        let Self { message } = self;
        let message = js_str_lit(message.as_str());

        Ok(*quote_expr!(r#"(() => { throw new Error($msg)})()"#, msg: Expr = message))
    }
}

/// Input:
///
/// ```text
/// let x := foo;
/// body
/// ```
///
/// Output:
///
/// ```js
/// ((x) => 〚 body 〛)(〚 foo 〛)
/// ```
impl ToJSExpr for ir::LocalLet {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        let Self { name, bound, body } = self;

        let body = body.to_js_expr()?;
        let bound = bound.to_js_expr()?;

        Ok(*quote_expr!(
            "(($name) => ($body))($bound)",
            name = name.to_string().into(),
            body: Expr = body,
            bound: Expr = bound
        ))
    }
}

/// Input:
///
/// ```text
/// do { b; ... b; e }
/// ```
///
/// Output:
///
/// ```js
/// (() => {
///     〚 b; ... b; 〛
///     return 〚 foo 〛();
/// })
/// ```
impl ToJSExpr for ir::DoBlock {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        let Self { bindings, return_exp } = self;

        let mut js_bindings = Vec::with_capacity(bindings.len());
        for binding in bindings {
            let js_binding = binding.to_js_stmt()?;
            js_bindings.push(js_binding);
        }

        let js_return_stmt = js::Stmt::Return(js::ReturnStmt {
            span: DUMMY_SP,
            arg: Some(Box::new(force_expr(paren_expr(return_exp.to_js_expr()?)))),
        });

        let mut js_stmts = js_bindings;
        js_stmts.push(js_return_stmt);

        Ok(thunk_block(js_stmts))
    }
}

/// Input:
///
/// ```text
/// let x := e1;
/// y <- e2;
/// ```
///
/// Output:
///
/// ```js
/// const x = 〚 e1 〛;
/// const y = 〚 e2 〛();
/// ```
impl ToJSStmt for ir::DoBinding {
    fn to_js_stmt(&self) -> BackendResult<js::Stmt> {
        let var_declarator = match self {
            ir::DoBinding::Let { name, bound } => js::VarDeclarator {
                span: DUMMY_SP,
                name: js::Pat::Ident(js::BindingIdent::from(js::Ident::from(name.to_string()))),
                init: Some(Box::new(bound.to_js_expr()?)),
                definite: false,
            },
            ir::DoBinding::Bind { name, bound } => js::VarDeclarator {
                span: DUMMY_SP,
                name: js::Pat::Ident(js::BindingIdent::from(js::Ident::from(name.to_string()))),
                init: Some(Box::new(force_expr(bound.to_js_expr()?))),
                definite: false,
            },
        };

        let var_decl = js::VarDecl {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            kind: js::VarDeclKind::Const,
            declare: false,
            decls: vec![var_declarator],
        };

        Ok(js::Stmt::Decl(js::Decl::Var(Box::new(var_decl))))
    }
}

/// Input:
///
/// ```text
/// 42
/// 42.42
/// 'a'
/// "somestring"
/// ```
///
/// Output:
///
/// ```js
/// 42n
/// 42.42
/// 97
/// "somestring"
/// ```
impl ToJSExpr for ir::Literal {
    fn to_js_expr(&self) -> BackendResult<js::Expr> {
        Ok(match self {
            ir::Literal::I64(int) => js_bigint_lit(*int),
            ir::Literal::F64(float) => js_num_lit(*float),
            ir::Literal::Char(c) => js_num_lit(*c as usize),
            ir::Literal::String(string) => js_str_lit(string.as_str()),
        })
    }
}

impl ir::Case {
    /// Input:
    ///
    /// ```text
    /// C(x, y) => body,
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// case "C":
    ///     const x = __scrutinee.args[0];
    ///     const y = __scrutinee.args[1];
    ///     return 〚 body 〛;
    /// ```
    pub fn to_js_switch_case(&self, scrutinee_name: &str) -> BackendResult<js::SwitchCase> {
        let Self { pattern, body } = self;
        let pattern_name = pattern.name.clone();
        let test = js_str_lit(pattern_name.to_string());

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
                    name: js::Pat::Ident(js::BindingIdent::from(js::Ident::from(
                        param_name.to_string(),
                    ))),
                    init: Some(Box::new(js::Expr::Member(js::MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(js::Expr::Member(js::MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(js::Expr::Ident(scrutinee_name.into())),
                            prop: js::MemberProp::Ident(CTOR_ARGS.into()),
                        })),
                        prop: js::MemberProp::Computed(js::ComputedPropName {
                            span: DUMMY_SP,
                            expr: Box::new(js_num_lit(i)),
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

    /// Input:
    ///
    /// ```text
    /// .d(x, y) => body,
    /// ```
    ///
    /// Output:
    ///
    /// ```js
    /// d: (x, y) => (〚 body 〛):
    /// ```
    pub fn to_js_object_method(&self) -> BackendResult<js::PropOrSpread> {
        let Self { pattern, body } = self;
        let method_name = pattern.name.clone();
        let params: Vec<_> = pattern
            .params
            .iter()
            .map(|p| js::Pat::Ident(js::BindingIdent::from(js::Ident::from(p.to_string()))))
            .collect();

        let body_expr = body.to_js_expr()?;

        // Wrap the body expression in parentheses.
        // Without them, returning some expressions (such as objects literals) from an arrow function is not valid JavaScript syntax.
        let paren_body = paren_expr(body_expr);

        let arrow = js::Expr::Arrow(js::ArrowExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            params,
            body: Box::new(js::BlockStmtOrExpr::Expr(Box::new(paren_body))),
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
        });

        Ok(js::PropOrSpread::Prop(Box::new(js::Prop::KeyValue(js::KeyValueProp {
            key: js::PropName::Ident(js::IdentName::from(method_name.to_string())),
            value: Box::new(arrow),
        }))))
    }
}

fn args_to_js_array(args: &[ir::Exp]) -> BackendResult<js::ArrayLit> {
    let elems = args_to_js_exprs(args)?.into_iter().map(Some).collect::<Vec<_>>();
    Ok(js::ArrayLit { span: DUMMY_SP, elems })
}

fn args_to_js_exprs(args: &[ir::Exp]) -> BackendResult<Vec<js::ExprOrSpread>> {
    args.iter()
        .map(|arg| {
            arg.to_js_expr().map(|expr| js::ExprOrSpread { spread: None, expr: Box::new(expr) })
        })
        .collect()
}

fn js_str_lit(str: impl Into<js::Str>) -> js::Expr {
    js::Expr::Lit(js::Lit::Str(str.into()))
}

fn js_num_lit(num: impl Into<js::Number>) -> js::Expr {
    js::Expr::Lit(js::Lit::Num(num.into()))
}

fn js_bigint_lit(bigint: impl Into<js::BigIntValue>) -> js::Expr {
    let bigint: js::BigIntValue = bigint.into();
    js::Expr::Lit(js::Lit::BigInt(bigint.into()))
}

fn extern_call_to_js_expr(name: &str, args: Vec<js::Expr>) -> Option<Box<js::Expr>> {
    Some(match name {
        "add_i64" => {
            let (x, y) = take2(args);
            quote_expr!("BigInt.asIntN(64, $x + $y)", x: Expr = x, y: Expr = y)
        }
        "sub_i64" => {
            let (x, y) = take2(args);
            quote_expr!("BigInt.asIntN(64, $x - $y)", x: Expr = x, y: Expr = y)
        }
        "mul_i64" => {
            let (x, y) = take2(args);
            quote_expr!("BigInt.asIntN(64, $x * $y)", x: Expr = x, y: Expr = y)
        }
        "div_i64" => {
            let (x, y) = take2(args);
            quote_expr!("BigInt.asIntN(64, $x / $y)", x: Expr = x, y: Expr = y)
        }
        "add_f64" => {
            let (x, y) = take2(args);
            quote_expr!("($x + $y)", x: Expr = x, y: Expr = y)
        }
        "sub_f64" => {
            let (x, y) = take2(args);
            quote_expr!("($x - $y)", x: Expr = x, y: Expr = y)
        }
        "mul_f64" => {
            let (x, y) = take2(args);
            quote_expr!("($x * $y)", x: Expr = x, y: Expr = y)
        }
        "div_f64" => {
            let (x, y) = take2(args);
            quote_expr!("($x / $y)", x: Expr = x, y: Expr = y)
        }
        "concat" => {
            let (x, y) = take2(args);
            quote_expr!("$x.concat($y)", x: Expr = x, y: Expr = y)
        }
        "append_char" => {
            let (c, s) = take2(args);
            quote_expr!("$s.concat(String.fromCodePoint($c))", s: Expr = s, c: Expr = c)
        }
        "unit" => js::Expr::undefined(DUMMY_SP),
        "return_io" => {
            let x = take1(args);
            quote_expr!("(() => $x)", x: Expr = x)
        }
        "println" => {
            let s = take1(args);
            quote_expr!("(() => { console.log($s); return void 0; })", s: Expr = s)
        }
        _ => return None,
    })
}

// Get the value of a Vec with *exactly* one element.
fn take1(mut args: Vec<js::Expr>) -> js::Expr {
    debug_assert_eq!(args.len(), 1);
    args.swap_remove(0)
}

// Get the values of a Vec with *exactly* two elements.
fn take2(mut args: Vec<js::Expr>) -> (js::Expr, js::Expr) {
    debug_assert_eq!(args.len(), 2);
    let y = args.swap_remove(1);
    let x = args.swap_remove(0);
    (x, y)
}
