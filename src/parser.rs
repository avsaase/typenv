//! Parsers that turn an environment variable's value into a field's type.
//!
//! Select one with `#[env(parser = ...)]`; fields without one use [`from_str`].

use std::{error::Error, str::FromStr};

use crate::BoxError;

/// The result of a [`Parser`]. Any error that is `Send + Sync + 'static` can be returned.
pub type ParseResult<T> = Result<T, BoxError>;

/// Converts a variable's string value into a `T`.
///
/// Implemented for every `Fn(&str) -> ParseResult<T>`, so plain functions and
/// closures can be used directly.
pub trait Parser<T> {
    /// Parses `s` into a `T`.
    fn parse(&self, s: &str) -> ParseResult<T>;
}

/// Parses with [`FromStr`]. This is the default parser.
pub fn from_str<T>(s: &str) -> ParseResult<T>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    s.parse::<T>().map_err(|e| e.into())
}

/// Converts with [`From<String>`]. Never fails.
pub fn into<T>(s: &str) -> ParseResult<T>
where
    T: From<String>,
{
    Ok(s.to_owned().into())
}

/// Converts with [`TryFrom<String>`].
pub fn try_into<T>(s: &str) -> ParseResult<T>
where
    T: TryFrom<String>,
    <T as TryFrom<String>>::Error: Error + Send + Sync + 'static,
{
    Ok(s.to_owned().try_into()?)
}

/// Deserializes the value as JSON. Requires the `json` feature.
#[cfg(feature = "json")]
pub fn json<T>(s: &str) -> ParseResult<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(s).map_err(|e| e.into())
}

impl<T, F> Parser<T> for F
where
    F: for<'a> Fn(&'a str) -> ParseResult<T>,
{
    fn parse(&self, s: &str) -> ParseResult<T> {
        (self)(s)
    }
}
