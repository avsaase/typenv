use std::error::Error as StdError;
use std::fmt::Display;

/// A boxed error, as returned by a [`Parser`](crate::parser::Parser).
pub type BoxError = Box<dyn StdError + Send + Sync + 'static>;

/// Every error encountered while loading a struct, returned by
/// [`FromEnv::from_env`](crate::FromEnv::from_env).
///
/// Loading does not stop at the first problem, so this reports all missing and invalid variables at
/// once. Its `Display` output lists each error together with its cause.
#[derive(Debug)]
pub struct FromEnvErrors(pub(crate) Vec<FromEnvError>);

/// A single problem with one field.
///
/// Every variant has a `path`, the field's location from the struct
/// [`from_env`](crate::FromEnv::from_env) was called on, with nested fields joined by dots (e.g.
/// `database.port`), and a `var`, the environment variable that was read.
#[derive(Debug)]
#[non_exhaustive]
pub enum FromEnvError {
    /// A required variable is not set.
    MissingVar { path: String, var: String },
    /// A variable is set, but its value could not be parsed. The parser's error is available
    /// through [`source`](StdError::source).
    ParseError {
        path: String,
        var: String,
        /// The variable's value. Values that are not valid Unicode are converted lossily.
        value: String,
        source: BoxError,
    },
    /// A variable is not set and the field's `#[env(default = "...")]` could not be parsed. This is
    /// a bug in the code, not in the environment. The parser's error is available through
    /// [`source`](StdError::source).
    InvalidDefault {
        path: String,
        var: String,
        /// The default value from the attribute.
        default: String,
        source: BoxError,
    },
}

impl Display for FromEnvErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Configuration errors:")?;

        for error in &self.0 {
            write!(f, "  - {error}")?;

            // `FromEnvError` leaves its cause to `source()`, so render the
            // whole chain here to keep the list self-contained.
            let mut source = error.source();
            while let Some(cause) = source {
                write!(f, ": {cause}")?;
                source = cause.source();
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Display for FromEnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromEnvError::MissingVar { path, var } => write!(
                f,
                "`{path}`: Missing required environment variable: `{var}`"
            ),
            FromEnvError::ParseError {
                path, var, value, ..
            } => write!(
                f,
                "`{path}`: Failed to parse environment variable `{var}=\"{value}\"`"
            ),
            FromEnvError::InvalidDefault {
                path, var, default, ..
            } => write!(
                f,
                "`{path}`: Invalid default value \"{default}\" for `{var}` (variable is not set)"
            ),
        }
    }
}

impl StdError for FromEnvErrors {}

impl StdError for FromEnvError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            FromEnvError::MissingVar { .. } => None,
            FromEnvError::ParseError { source, .. }
            | FromEnvError::InvalidDefault { source, .. } => Some(source.as_ref()),
        }
    }
}
