//! Generates the `ENV_VARS` consts: one line per environment variable, including the
//! variables of nested structs.
//!
//! A struct's own variables are listed without a source (`APP_PORT (optional)`). Variables of
//! nested structs name the field they are loaded into (`DB_URL, from Database::url`). To get both,
//! every struct also generates `__NESTED_ENV_VARS`, its list with sources, which parents include.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{GenericParam, Generics, Type, ext::IdentExt};

use crate::model::{Container, FieldKind, Mode, ValueField};

/// A piece of the generated string: either text known to the macro, or a nested struct's
/// `ENV_VARS`, which is only known once that struct's own derive has run.
enum Part<'a> {
    Text(String),
    Nested(&'a Type),
}

/// Where a struct's list is shown.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Placement {
    /// As the struct's own `ENV_VARS`: its own variables have no source.
    TopLevel,
    /// Inside a parent's list, as `__NESTED_ENV_VARS`: every variable has a source.
    Nested,
}

/// An expression of type `&'static str` documenting every variable `container` reads.
pub(super) fn env_vars(container: &Container, placement: Placement) -> TokenStream {
    let generic_idents = generic_idents(&container.generics);
    let struct_name = container.ident.unraw().to_string();
    let mut parts: Vec<Part> = Vec::new();

    for field in &container.fields {
        let source = format!("{struct_name}::{}", field.path);
        match &field.kind {
            FieldKind::Value(value) => {
                let source = (placement == Placement::Nested).then_some(source.as_str());
                push_text(&mut parts, value_line(value, source));
            }
            // Nested parts are joined into an array whose length must be known without the
            // struct's generic parameters, so a type that uses them is only referenced.
            FieldKind::Nested { ty } if mentions(ty.to_token_stream(), &generic_idents) => {
                push_text(
                    &mut parts,
                    format!(
                        "variables of `{}`, from {source}\n",
                        display(ty.to_token_stream())
                    ),
                );
            }
            FieldKind::Nested { ty } => parts.push(Part::Nested(ty)),
        }
    }

    if !parts.iter().any(|part| matches!(part, Part::Nested(_))) {
        let text: String = parts
            .iter()
            .map(|part| match part {
                Part::Text(text) => text.as_str(),
                Part::Nested(_) => unreachable!(),
            })
            .collect();
        return quote! { #text };
    }

    let parts = parts.iter().map(|part| match part {
        Part::Text(text) => quote! { #text },
        Part::Nested(ty) => quote! { <#ty as ::typenv::FromEnv>::__NESTED_ENV_VARS },
    });

    quote! {
        {
            const PARTS: &[&str] = &[#(#parts),*];
            const LEN: usize = ::typenv::__private::concat_len(PARTS);
            const BYTES: [u8; LEN] = ::typenv::__private::concat::<LEN>(PARTS);
            match ::core::str::from_utf8(&BYTES) {
                ::core::result::Result::Ok(env_vars) => env_vars,
                ::core::result::Result::Err(_) => {
                    ::core::panic!("concatenating `&str`s always produces valid UTF-8")
                }
            }
        }
    }
}

/// Appends `text`, merging it with a preceding text part.
fn push_text(parts: &mut Vec<Part>, text: String) {
    match parts.last_mut() {
        Some(Part::Text(last)) => last.push_str(&text),
        _ => parts.push(Part::Text(text)),
    }
}

/// E.g. `APP_PORT (optional)`, or `DB_URL, from Database::url` with a source. Required
/// variables have no parenthetical.
fn value_line(value: &ValueField, source: Option<&str>) -> String {
    let note = match &value.mode {
        Mode::Required => String::new(),
        Mode::Optional => " (optional)".to_owned(),
        Mode::Default(default) => format!(" (default: {default})"),
        Mode::DefaultValue(expr) => format!(" (default: {})", display(expr.to_token_stream())),
    };
    match source {
        Some(source) => format!("{}{note}, from {source}\n", value.var),
        None => format!("{}{note}\n", value.var),
    }
}

/// The identifiers of the struct's type, const and lifetime parameters.
fn generic_idents(generics: &Generics) -> Vec<String> {
    generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(param) => param.ident.to_string(),
            GenericParam::Const(param) => param.ident.to_string(),
            GenericParam::Lifetime(param) => param.lifetime.ident.to_string(),
        })
        .collect()
}

/// Whether `tokens` contain any of `idents`.
fn mentions(tokens: TokenStream, idents: &[String]) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(ident) => idents.iter().any(|name| ident == name),
        TokenTree::Group(group) => mentions(group.stream(), idents),
        TokenTree::Punct(_) | TokenTree::Literal(_) => false,
    })
}

/// Renders tokens the way they are usually written, e.g. `Option<Vec<u8>>` rather than the
/// `Option < Vec < u8 > >` produced by `TokenStream::to_string`.
fn display(tokens: TokenStream) -> String {
    let mut out = String::new();
    // Whether the previous token was a word, so the next word needs a space before it.
    let mut after_word = false;

    for token in tokens {
        match token {
            TokenTree::Ident(ident) => {
                if after_word {
                    out.push(' ');
                }
                out.push_str(&ident.to_string());
                after_word = true;
            }
            TokenTree::Literal(literal) => {
                if after_word {
                    out.push(' ');
                }
                out.push_str(&literal.to_string());
                after_word = true;
            }
            TokenTree::Punct(punct) => {
                match punct.as_char() {
                    ',' | ';' => {
                        out.push(punct.as_char());
                        out.push(' ');
                    }
                    '+' | '=' if punct.spacing() == Spacing::Alone => {
                        out.push(' ');
                        out.push(punct.as_char());
                        out.push(' ');
                    }
                    c => out.push(c),
                }
                after_word = false;
            }
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Brace => ("{ ", " }"),
                    Delimiter::None => ("", ""),
                };
                if group.delimiter() == Delimiter::Brace && after_word {
                    out.push(' ');
                }
                out.push_str(open);
                out.push_str(display(group.stream()).trim_end());
                out.push_str(close);
                after_word = true;
            }
        }
    }

    out.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use syn::{Expr, Type, parse_quote};

    use super::*;

    fn ty(ty: Type) -> String {
        display(ty.to_token_stream())
    }

    fn expr(expr: Expr) -> String {
        display(expr.to_token_stream())
    }

    #[test]
    fn displays_types() {
        assert_eq!(ty(parse_quote!(u16)), "u16");
        assert_eq!(ty(parse_quote!(Option<Vec<u8>>)), "Option<Vec<u8>>");
        assert_eq!(
            ty(parse_quote!(std::collections::HashMap<String, u8>)),
            "std::collections::HashMap<String, u8>"
        );
        assert_eq!(ty(parse_quote!(&'static str)), "&'static str");
        assert_eq!(ty(parse_quote!([u8; 4])), "[u8; 4]");
        assert_eq!(
            ty(parse_quote!(Box<dyn Error + Send>)),
            "Box<dyn Error + Send>"
        );
    }

    #[test]
    fn displays_expressions() {
        assert_eq!(expr(parse_quote!(5)), "5");
        assert_eq!(expr(parse_quote!(-1)), "-1");
        assert_eq!(
            expr(parse_quote!(Duration::from_secs(5))),
            "Duration::from_secs(5)"
        );
        assert_eq!(
            expr(parse_quote!(vec!["a".to_owned(), "b".to_owned()])),
            r#"vec!["a".to_owned(), "b".to_owned()]"#
        );
        assert_eq!(expr(parse_quote!(T::default())), "T::default()");
    }

    #[test]
    fn detects_generic_parameters() {
        let generics: Generics = parse_quote!(<'a, T, const N: usize>);
        let idents = generic_idents(&generics);

        let uses = |ty: Type| mentions(ty.to_token_stream(), &idents);
        assert!(uses(parse_quote!(Inner<T>)));
        assert!(uses(parse_quote!(Inner<'a>)));
        assert!(uses(parse_quote!([u8; N])));
        assert!(!uses(parse_quote!(Database)));
        assert!(!uses(parse_quote!(Vec<Tee>)));
    }
}
