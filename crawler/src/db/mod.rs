use std::{error::Error as StdError, fmt};

use postgres::{Client, GenericClient, NoTls, Transaction};

pub mod channel;
pub mod message;

#[derive(Debug)]
pub struct Error {
    operation: String,
    source: postgres::Error,
}

impl Error {
    pub fn new(operation: impl Into<String>, source: postgres::Error) -> Self {
        Self {
            operation: operation.into(),
            source,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.operation, error_string(&self.source))
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

pub fn connect(url: &str) -> Result<Client, postgres::Error> {
    Client::connect(url, NoTls)
}

/// Commits if f returns Ok, rolls back (via Drop) if it returns Err.
pub fn with_tx<T>(
    client: &mut impl GenericClient,
    f: impl FnOnce(&mut Transaction<'_>) -> Result<T, Error>,
) -> Result<T, Error> {
    let mut tx = client.transaction()
        .map_err(|e| Error::new("db::with_tx: begin transaction", e))?;
    let out = f(&mut tx)?;
    tx.commit().map_err(|e| Error::new("db::with_tx: commit transaction", e))?;
    Ok(out)
}

/// Formats a `postgres::Error` with server details or its underlying causes.
///
/// The crate's own `Display` for database errors is just "db error"; the
/// message, SQLSTATE and constraint only surface through `as_db_error()`.
pub fn error_string(e: &postgres::Error) -> String {
    match e.as_db_error() {
        Some(db) => format!(
            "{} (code: {}, constraint: {}, detail: {})",
            db.message(),
            db.code().code(),
            db.constraint().unwrap_or("-"),
            db.detail().unwrap_or("-")
        ),
        None => {
            let mut message = e.to_string();
            let mut source = e.source();
            while let Some(cause) = source {
                message.push_str(": ");
                message.push_str(&cause.to_string());
                source = cause.source();
            }
            message
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_error_keeps_operation_and_underlying_cause() {
        // Invalid configuration fails before a connection is attempted.
        let source = match connect("postgresql://localhost/telewave?port=invalid") {
            Err(error) => error,
            Ok(_) => panic!("invalid port should fail"),
        };
        let error = Error::new("claim_channel_due_for_crawl: select candidate", source);

        let message = error.to_string();
        assert!(message.contains("claim_channel_due_for_crawl: select candidate"));
        assert!(message.contains("invalid value for option `port`"), "{message}");
        assert!(error.source().unwrap().is::<postgres::Error>());
    }
}
