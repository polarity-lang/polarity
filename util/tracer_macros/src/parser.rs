use std::{iter::Peekable, str::Chars};

use syn::parse::Parser;
use syn::punctuated::Punctuated;

use super::syntax::*;

pub fn parse(stream: proc_macro2::TokenStream) -> Result<Format, ParseError> {
    let args = parse_args(stream)?;
    if args.is_empty() {
        return Err(ParseError::MissingFormatString);
    }

    let string = parse_literal(&args[0])?;
    let args = args.into_iter().skip(1).collect();

    let res = Format { string, args, ret: None, ret_type: None };

    if res.expected_args_number() != res.actual_args_number() {
        return Err(ParseError::ArgumentNumberMismatch {
            expected: res.expected_args_number(),
            actual: res.actual_args_number(),
        });
    }

    Ok(res)
}

fn parse_args(stream: proc_macro2::TokenStream) -> Result<Vec<syn::Expr>, ParseError> {
    let parser = Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;

    parser
        .parse2(stream)
        .map_err(ParseError::Syn)
        .map(|punctuated| punctuated.into_iter().collect())
}

fn parse_literal(exp: &syn::Expr) -> Result<FormatStr, ParseError> {
    match exp {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(str_lit), .. }) => {
            parse_format_str(&str_lit.value())
        }
        _ => Err(ParseError::MissingFormatString),
    }
}

fn parse_format_str(s: &str) -> Result<FormatStr, ParseError> {
    let mut iter = s.chars().peekable();

    let mut items = Vec::new();

    while iter.peek().is_some() {
        items.push(parse_item(&mut iter)?);
    }

    Ok(FormatStr { items })
}

fn parse_item(iter: &mut Peekable<Chars>) -> Result<Item, ParseError> {
    let mut buf = String::new();

    // Consume as much whitespace as possible
    while let Some(' ') = iter.peek() {
        iter.next().unwrap();
        buf.push(' ');
    }

    if !buf.is_empty() {
        return Ok(Item::Whitespace(buf));
    }

    // Parse a curly brace (either escaped with another brace or marking the beginning of string interpolation)
    if let Some('{') = iter.peek() {
        iter.next().unwrap();
        if let Some('{') = iter.peek() {
            iter.next().unwrap();
            return Ok(Item::Separator("{".to_owned()));
        } else {
            let (arg, done) = parse_arg(iter)?;
            let spec = if done { None } else { Some(parse_spec(iter)?) };
            return Ok(Item::Interpolation(arg, spec));
        }
    }

    while iter.peek().is_some()
        && !matches!(iter.peek(), Some(' '))
        && !matches!(iter.peek(), Some('{'))
    {
        buf.push(iter.next().unwrap());
    }

    Ok(Item::Separator(buf))
}

fn parse_arg(iter: &mut Peekable<Chars>) -> Result<(Arg, bool), ParseError> {
    let done;
    let mut buf = String::new();
    loop {
        match iter.next() {
            Some('{') => return Err(ParseError::InvalidNesting),
            Some(':') => {
                done = false;
                break;
            }
            Some('}') => {
                done = true;
                break;
            }
            Some(c) => buf.push(c),
            None => return Err(ParseError::UnexpectedEOF),
        }
    }

    if buf.trim() == "return" {
        Ok((Arg::Return, done))
    } else {
        Ok((Arg::Other(buf), done))
    }
}

fn parse_spec(iter: &mut Peekable<Chars>) -> Result<Spec, ParseError> {
    let mut buf = String::new();

    loop {
        match iter.next() {
            Some('}') => break,
            Some('{') => return Err(ParseError::InvalidNesting),
            Some(c) => buf.push(c),
            None => return Err(ParseError::UnexpectedEOF),
        }
    }

    if buf.trim() == "P" {
        Ok(Spec::Pretty)
    } else {
        Ok(Spec::Other(buf))
    }
}

#[derive(Debug)]
pub enum ParseError {
    MissingFormatString,
    InvalidNesting,
    UnexpectedEOF,
    ArgumentNumberMismatch { expected: usize, actual: usize },
    Syn(syn::Error),
}
