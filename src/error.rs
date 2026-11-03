use std::fmt::Display;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct FromEnvErrors(pub(crate) Vec<FromEnvError>);

#[derive(Debug)]
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
}

impl Display for FromEnvErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Configuration errors:")?;

        for error in &self.0 {
            writeln!(f, "  - {error}")?;
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
                path,
                var,
                value,
                source,
            } => write!(
                f,
                "`{path}`: Failed to parse environment variable `{var}=\"{value}\"`: {source}"
            ),
        }
    }
}

impl std::error::Error for FromEnvErrors {}
impl std::error::Error for FromEnvError {}
