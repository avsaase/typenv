use std::error::Error as StdError;
use std::fmt::Display;

pub type BoxError = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug)]
pub struct FromEnvErrors(pub(crate) Vec<FromEnvError>);

#[derive(Debug)]
#[non_exhaustive]
pub enum FromEnvError {
    MissingVar {
        path: String,
        var: String,
    },
    ParseError {
        path: String,
        var: String,
        value: String,
        source: BoxError,
    },
    InvalidDefault {
        path: String,
        var: String,
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
