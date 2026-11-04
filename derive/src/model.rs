use proc_macro2::Span;
use syn::{Expr, Generics, Ident, Type, Visibility};

pub struct Container {
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Vec<Field>,
}

pub struct Field {
    /// The field's identifier, used when constructing `Self { .. }`.
    pub ident: Ident,
    /// The field's name without `r#`, used as its path in error messages.
    pub path: String,
    /// Span of the field's type, used for the generated code so that type
    /// errors point at the field.
    pub span: Span,
    pub kind: FieldKind,
}

pub enum FieldKind {
    /// `#[env(nested)]`: loaded through the type's own `FromEnv` impl.
    Nested { ty: Type },
    /// Read from a single environment variable.
    Value(ValueField),
}

pub struct ValueField {
    /// The fully resolved environment variable name.
    pub var: String,
    /// The type the parser produces. For `Option<T>` fields this is `T`.
    pub ty: Type,
    /// A custom `#[env(parser = ..)]`, or `None` to use `FromStr`.
    pub parser: Option<Expr>,
    pub mode: Mode,
}

pub enum Mode {
    /// A missing variable is an error.
    Required,
    /// The field is `Option<T>` and a missing variable yields `None`.
    Optional,
    /// A missing variable falls back to parsing this string.
    Default(String),
    /// A missing variable falls back to this typed expression, which is only
    /// evaluated when needed and never passed through the parser.
    DefaultValue(Expr),
}
