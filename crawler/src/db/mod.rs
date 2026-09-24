use postgres::{Client, Error, GenericClient, NoTls, Transaction};

pub mod channel;
pub mod message;

pub fn connect(url: &str) -> Result<Client, Error> {
    Client::connect(url, NoTls)
}

/// Commits if f returns Ok, rolls back (via Drop) if it returns Err.
pub fn with_tx<T>(
    client: &mut impl GenericClient,
    f: impl FnOnce(&mut Transaction<'_>) -> Result<T, Error>,
) -> Result<T, Error> {
    let mut tx = client.transaction()?;
    let out = f(&mut tx)?;
    tx.commit()?;
    Ok(out)
}

/// Formats a `postgres::Error` with its server-side details.
///
/// The crate's own `Display` for database errors is just "db error"; the
/// message, SQLSTATE and constraint only surface through `as_db_error()`.
pub fn error_string(e: &Error) -> String {
    match e.as_db_error() {
        Some(db) => format!(
            "{} (code: {}, constraint: {}, detail: {})",
            db.message(),
            db.code().code(),
            db.constraint().unwrap_or("-"),
            db.detail().unwrap_or("-")
        ),
        None => e.to_string(),
    }
}
