use std::{error::Error, str::FromStr};

use crate::BoxError;

pub type ParseResult<T> = Result<T, BoxError>;

pub trait Parser<T> {
    fn parse(&self, s: &str) -> ParseResult<T>;
}

pub fn from_str<T>(s: &str) -> ParseResult<T>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    s.parse::<T>().map_err(|e| e.into())
}

pub fn into<T>(s: &str) -> ParseResult<T>
where
    T: From<String>,
{
    Ok(s.to_owned().into())
}

pub fn try_into<T>(s: &str) -> ParseResult<T>
where
    T: TryFrom<String>,
    <T as TryFrom<String>>::Error: Error + Send + Sync + 'static,
{
    Ok(s.to_owned().try_into()?)
}

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
