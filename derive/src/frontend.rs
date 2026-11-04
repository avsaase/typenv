use darling::{
    FromDeriveInput, FromField, ast,
    util::{Flag, SpannedValue},
};
use proc_macro2::Span;
use syn::{
    DeriveInput, Expr, GenericArgument, Generics, Ident, PathArguments, Type, ext::IdentExt,
    spanned::Spanned,
};

use crate::model::{Container, Field, FieldKind, Mode, ValueField};

pub fn parse(ast: &DeriveInput) -> darling::Result<Container> {
    let receiver = FromEnvReceiver::from_derive_input(ast)?;

    let prefix = receiver.prefix.unwrap_or_default();
    let fields = receiver
        .data
        .take_struct()
        .expect("darling only accepts structs with named fields")
        .fields
        .into_iter()
        .map(|field| field.into_model(&prefix))
        .collect();

    Ok(Container {
        ident: receiver.ident,
        generics: receiver.generics,
        fields,
    })
}

#[derive(FromDeriveInput)]
#[darling(attributes(env), supports(struct_named))]
struct FromEnvReceiver {
    // vis: Visibility,
    ident: Ident,
    generics: Generics,
    data: ast::Data<(), FieldReceiver>,
    /// Prepended verbatim to the variable names of this struct's fields,
    /// except fields with an explicit `var`. Not inherited by nested structs.
    prefix: Option<String>,
}

#[derive(FromField)]
#[darling(attributes(env), and_then = Self::validate)]
struct FieldReceiver {
    ident: Option<Ident>, // We rejected unnamed fields above so this is always Some.
    ty: Type,
    var: Option<SpannedValue<String>>,
    default: Option<SpannedValue<String>>,
    /// A typed default; string literals are kept as literals, not parsed as code.
    #[darling(default, with = darling::util::parse_expr::preserve_str_literal, map = Some)]
    default_value: Option<Expr>,
    rename: Option<SpannedValue<String>>,
    /// Any expression implementing `from_env::Parser<T>`, e.g. a function path or closure.
    parser: Option<Expr>,
    nested: Flag,
}

fn error(span: Span, message: &str) -> darling::Error {
    darling::Error::custom(message).with_span(&span)
}

impl FieldReceiver {
    fn validate(self) -> darling::Result<Self> {
        let mut errors = darling::Error::accumulator();

        let is_optional = option_inner(&self.ty).is_some();
        let has_var_settings = self.var.is_some()
            || self.default.is_some()
            || self.default_value.is_some()
            || self.rename.is_some()
            || self.parser.is_some();

        if self.var.is_some()
            && let Some(rename) = &self.rename
        {
            errors.push(error(rename.span(), "`rename` cannot be used with `var`"));
        }

        if is_optional && let Some(default) = &self.default {
            errors.push(error(
                default.span(),
                "`default` cannot be used on an optional field",
            ));
        }

        if let Some(default_value) = &self.default_value {
            if self.default.is_some() {
                errors.push(error(
                    default_value.span(),
                    "`default_value` cannot be used with `default`",
                ));
            }

            if is_optional {
                errors.push(error(
                    default_value.span(),
                    "`default_value` cannot be used on an optional field",
                ));
            }
        }

        if self.nested.is_present() {
            if has_var_settings {
                errors.push(error(
                    self.nested.span(),
                    "`nested` cannot be used with other attributes",
                ));
            }

            if is_optional {
                errors.push(error(
                    self.nested.span(),
                    "`nested` cannot be used on an optional field",
                ));
            }
        }

        errors.finish_with(self)
    }

    /// Converts a field that has passed [`Self::validate`] into the model.
    ///
    /// `prefix` is the struct's `#[env(prefix = ..)]`, or empty.
    fn into_model(self, prefix: &str) -> Field {
        let ident = self.ident.expect("darling only accepts named fields");
        let path = ident.unraw().to_string();
        let span = self.ty.span();

        let kind = if self.nested.is_present() {
            FieldKind::Nested { ty: self.ty }
        } else {
            let var = match (self.var, self.rename) {
                (Some(var), _) => var.into_inner(),
                (None, Some(rename)) => format!("{prefix}{}", rename.to_uppercase()),
                (None, None) => format!("{prefix}{}", path.to_uppercase()),
            };

            let (ty, mode) = match (option_inner(&self.ty), self.default, self.default_value) {
                (Some(inner), _, _) => (inner.clone(), Mode::Optional),
                (None, Some(default), _) => (self.ty, Mode::Default(default.into_inner())),
                (None, None, Some(value)) => (self.ty, Mode::DefaultValue(value)),
                (None, None, None) => (self.ty, Mode::Required),
            };

            FieldKind::Value(ValueField {
                var,
                ty,
                parser: self.parser,
                mode,
            })
        };

        Field {
            ident,
            path,
            span,
            kind,
        }
    }
}

/// If `ty` is written as `Option<T>`, returns `T`.
fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let last = path.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::parse_quote;

    use super::*;

    fn parse_fields(input: DeriveInput) -> Vec<Field> {
        parse(&input).unwrap_or_else(|e| panic!("{e}")).fields
    }

    fn value(field: &Field) -> &ValueField {
        match &field.kind {
            FieldKind::Value(value) => value,
            FieldKind::Nested { .. } => panic!("`{}` is nested", field.path),
        }
    }

    fn tokens(t: &impl ToTokens) -> String {
        t.to_token_stream().to_string()
    }

    #[test]
    fn var_name_resolution() {
        let fields = parse_fields(parse_quote! {
            struct Config {
                db_url: String,
                #[env(var = "exact_Name")]
                a: String,
                #[env(rename = "renamed")]
                b: String,
                r#type: String,
            }
        });

        let vars: Vec<&str> = fields.iter().map(|f| value(f).var.as_str()).collect();
        assert_eq!(vars, ["DB_URL", "exact_Name", "RENAMED", "TYPE"]);
        assert_eq!(fields[3].path, "type");
        assert_eq!(fields[3].ident.to_string(), "r#type");
    }

    #[test]
    fn prefix() {
        let fields = parse_fields(parse_quote! {
            #[env(prefix = "App_")]
            struct Config {
                db_url: String,
                #[env(rename = "renamed")]
                a: String,
                #[env(var = "EXACT")]
                b: String,
                #[env(nested)]
                c: Nested,
            }
        });

        let vars: Vec<&str> = fields[..3].iter().map(|f| value(f).var.as_str()).collect();
        assert_eq!(vars, ["App_DB_URL", "App_RENAMED", "EXACT"]);
        assert!(matches!(fields[3].kind, FieldKind::Nested { .. }));
    }

    #[test]
    fn modes() {
        let fields = parse_fields(parse_quote! {
            struct Config {
                required: u16,
                optional: Option<u16>,
                #[env(default = "5")]
                defaulted: u16,
                #[env(default_value = 5)]
                typed: u16,
                #[env(default_value = "five")]
                typed_str: &'static str,
            }
        });

        assert!(matches!(value(&fields[0]).mode, Mode::Required));
        assert!(matches!(value(&fields[1]).mode, Mode::Optional));
        assert!(matches!(&value(&fields[2]).mode, Mode::Default(d) if d == "5"));
        assert!(matches!(&value(&fields[3]).mode, Mode::DefaultValue(e) if tokens(e) == "5"));
        assert!(
            matches!(&value(&fields[4]).mode, Mode::DefaultValue(e) if tokens(e) == "\"five\""),
            "string literals must stay string literals"
        );
    }

    #[test]
    fn optional_field_uses_inner_type() {
        let fields = parse_fields(parse_quote! {
            struct Config {
                a: Option<Vec<u8>>,
                b: std::option::Option<u8>,
            }
        });

        assert_eq!(tokens(&value(&fields[0]).ty), "Vec < u8 >");
        assert_eq!(tokens(&value(&fields[1]).ty), "u8");
    }

    #[test]
    fn parser_is_kept() {
        let fields = parse_fields(parse_quote! {
            struct Config {
                #[env(parser = my::parser)]
                a: u8,
                b: u8,
            }
        });

        assert_eq!(
            value(&fields[0]).parser.as_ref().map(tokens).as_deref(),
            Some("my :: parser")
        );
        assert!(value(&fields[1]).parser.is_none());
    }

    #[test]
    fn nested_field() {
        let fields = parse_fields(parse_quote! {
            struct Config {
                #[env(nested)]
                database: Database,
            }
        });

        let FieldKind::Nested { ty } = &fields[0].kind else {
            panic!("expected nested field");
        };
        assert_eq!(tokens(ty), "Database");
    }

    #[test]
    fn all_field_errors_are_reported() {
        let input: DeriveInput = parse_quote! {
            struct Config {
                #[env(var = "A", rename = "b")]
                a: String,
                #[env(default = "1")]
                b: Option<u8>,
            }
        };

        let Err(error) = parse(&input) else {
            panic!("expected an error");
        };
        assert_eq!(error.len(), 2);
    }
}
