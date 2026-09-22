use postgres::{Client, Error, GenericClient, NoTls, Transaction};

pub mod channel;
pub mod message;

pub fn connect() -> Result<Client, Error> {
    let url = "postgres://winter@127.0.0.1:5432/telewave";
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
