use crate::{FromEnvError, FromEnvErrors, Parser};

pub fn required<T>(path: &str, var: &str, parser: &impl Parser<T>) -> Result<T, FromEnvError> {
    match read(path, var)? {
        Some(value) => parse(path, var, value, parser),
        None => Err(FromEnvError::MissingVar {
            path: path.to_string(),
            var: var.to_string(),
        }),
    }
}

pub fn optional<T>(
    path: &str,
    var: &str,
    parser: &impl Parser<T>,
) -> Result<Option<T>, FromEnvError> {
    // No silent fallback to the None case if parsing fails
    read(path, var)?
        .map(|value| parse(path, var, value, parser))
        .transpose()
}

pub fn with_default<T>(
    path: &str,
    var: &str,
    default: &str,
    parser: &impl Parser<T>,
) -> Result<T, FromEnvError> {
    let value = read(path, var)?.unwrap_or_else(|| default.to_owned());
    parse(path, var, value, parser)
}

fn read(path: &str, var: &str) -> Result<Option<String>, FromEnvError> {
    match std::env::var(var) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(ref error @ std::env::VarError::NotUnicode(ref malformed_value)) => {
            Err(FromEnvError::ParseError {
                path: path.to_string(),
                var: var.to_string(),
                value: malformed_value.to_string_lossy().to_string(),
                source: error.clone().into(),
            })
        }
    }
}

pub fn parse<T>(
    path: &str,
    var: &str,
    value: String,
    parser: &impl Parser<T>,
) -> Result<T, FromEnvError> {
    parser
        .parse(&value)
        .map_err(|error| FromEnvError::ParseError {
            path: path.to_string(),
            var: var.to_string(),
            value,
            source: error,
        })
}

pub struct ErrorCollector(Vec<FromEnvError>);

impl ErrorCollector {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn handle<T>(&mut self, result: Result<T, FromEnvError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.0.push(error);
                None
            }
        }
    }

    pub fn handle_nested<T>(
        &mut self,
        prefix: &str,
        result: Result<T, FromEnvErrors>,
    ) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(FromEnvErrors(errors)) => {
                self.0.extend(errors.into_iter().map(|mut error| {
                    match &mut error {
                        FromEnvError::MissingVar { path, .. }
                        | FromEnvError::ParseError { path, .. } => {
                            *path = format!("{prefix}.{path}");
                        }
                    }
                    error
                }));
                None
            }
        }
    }

    pub fn finish(self) -> FromEnvErrors {
        FromEnvErrors(self.0)
    }
}
