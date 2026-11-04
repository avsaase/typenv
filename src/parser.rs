//! Parsers that turn an environment variable's value into a field's type.
//!
//! Select one with `#[env(parser = ...)]`; fields without one use [`from_str`].

use std::str::FromStr;

use crate::BoxError;

/// The result of a [`Parser`]. Any error that is `Send + Sync + 'static` can be returned.
pub type ParseResult<T> = Result<T, BoxError>;

/// Converts a variable's string value into a `T`.
///
/// Implemented for every `Fn(&str) -> ParseResult<T>`, so plain functions and closures can be used
/// directly.
pub trait Parser<T> {
    /// Parses `s` into a `T`.
    fn parse(&self, s: &str) -> ParseResult<T>;
}

/// Parses with [`FromStr`]. This is the default parser.
///
/// `T::Err` can be any type convertible into [`BoxError`]: any `Error + Send + Sync + 'static`
/// type, as well as `String` and `&str`.
pub fn from_str<T>(s: &str) -> ParseResult<T>
where
    T: FromStr,
    T::Err: Into<BoxError>,
{
    s.parse::<T>().map_err(Into::into)
}

/// Converts with [`From<String>`]. Never fails.
pub fn into<T>(s: &str) -> ParseResult<T>
where
    T: From<String>,
{
    Ok(s.to_owned().into())
}

/// Converts with [`TryFrom<String>`]. The error type has the same requirements as for [`from_str`].
pub fn try_into<T>(s: &str) -> ParseResult<T>
where
    T: TryFrom<String>,
    <T as TryFrom<String>>::Error: Into<BoxError>,
{
    s.to_owned().try_into().map_err(Into::into)
}

/// Deserializes the value as JSON. Requires the `json` feature.
#[cfg(feature = "json")]
pub fn json<T>(s: &str) -> ParseResult<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(s).map_err(Into::into)
}

impl<T, F> Parser<T> for F
where
    F: for<'a> Fn(&'a str) -> ParseResult<T>,
{
    fn parse(&self, s: &str) -> ParseResult<T> {
        (self)(s)
    }
}
