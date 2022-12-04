use quote::{quote, ToTokens};

use super::syntax::*;

const FAILED_TO_WRITE: &str = "Failed to write to stdout";
const FAILED_TO_SET_COLOR: &str = "Failed to set terminal color";

impl ToTokens for Format {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote! {
            use std::io::Write;
            use printer::PrintExt;
            use printer::WriteColor;
            let mut __tracer_stdout = printer::StandardStream::stdout(printer::ColorChoice::Auto);
        });

        let mut args_iter = self.args.iter();

        for item in &self.string.items {
            match item {
                Item::Separator(s) => tokens.extend(print_separator(s)),
                Item::Whitespace(s) => tokens.extend(print_whitespace(s)),
                Item::Interpolation(arg_str, spec) => {
                    match print_arg_expr(
                        arg_str,
                        args_iter.next().unwrap(),
                        self.ret.as_ref(),
                        self.ret_type.as_ref(),
                    ) {
                        Some(expr) => tokens.extend(print_interpolation(spec.as_ref(), &expr)),
                        None => tokens.extend(print_separator("?")),
                    }
                }
            }
        }

        tokens.extend(quote! {
            __tracer_stdout.set_color(&printer::ColorSpec::default())
                .expect(#FAILED_TO_SET_COLOR);
            println!();
        })
    }
}

fn print_arg_expr(
    arg: &Arg,
    arg_expr: &syn::Expr,
    ret: Option<&proc_macro2::TokenStream>,
    ret_type: Option<&syn::Type>,
) -> Option<proc_macro2::TokenStream> {
    match arg {
        Arg::Return => ret.map(|ret_val| {
            quote! {
                {
                    // Help type inference figure out the input type of the closure and its lifetime
                    fn __tracer_constrain_cls<'a, T1, T2, F>(f: F) -> F
                    where
                        T1: 'a,
                        F: FnOnce(T1) -> T2,
                    {
                        f
                    }
                    (__tracer_constrain_cls::<'_, &#ret_type, _, _>(#arg_expr))(#ret_val)
                }
            }
        }),
        Arg::Other(_) => Some(quote! { #arg_expr }),
    }
}

fn print_interpolation(
    spec: Option<&Spec>,
    expr: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match spec {
        Some(Spec::Pretty) => print_pretty_interpolation(expr),
        Some(Spec::Other(spec)) => print_plain_interpolation(spec, expr),
        None => print_plain_interpolation("", expr),
    }
}

fn print_whitespace(s: &str) -> proc_macro2::TokenStream {
    quote! {
        __tracer_stdout.set_color(&printer::ColorSpec::default())
            .expect(#FAILED_TO_SET_COLOR);
        write!(&mut __tracer_stdout, #s)
            .expect(#FAILED_TO_WRITE);
    }
}

fn print_separator(s: &str) -> proc_macro2::TokenStream {
    quote! {
        __tracer_stdout.set_color(&printer::ColorSpec::new()
            .set_fg(Some(printer::Color::Black))
            .set_bg(Some(printer::Color::White)))
            .expect(#FAILED_TO_SET_COLOR);
        write!(&mut __tracer_stdout, #s)
            .expect(#FAILED_TO_WRITE);
    }
}

fn print_plain_interpolation(
    spec: &str,
    expr: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let fmt_string = format!("{{:{}}}", spec);

    quote! {
        __tracer_stdout.set_color(&printer::ColorSpec::default())
            .expect(#FAILED_TO_SET_COLOR);
        write!(&mut __tracer_stdout, #fmt_string, #expr)
            .expect(#FAILED_TO_WRITE);
    }
}

fn print_pretty_interpolation(expr: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        __tracer_stdout.set_color(&printer::ColorSpec::default())
            .expect(#FAILED_TO_SET_COLOR);
        #expr
            .print_colored(&printer::PrintCfg{ de_bruijn: true, ..Default::default() }, &mut __tracer_stdout)
            .expect("Failed to print to stdout");
    }
}
