use anyhow::{Error, Result, anyhow};
use diesel::{SqliteConnection, sqlite::Sqlite};
use diesel_async::{
    AsyncConnection,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{Object, Pool},
    },
    sync_connection_wrapper::SyncConnectionWrapper,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::env;

pub type DbConn = SyncConnectionWrapper<SqliteConnection>;
pub type DbPool = Pool<DbConn>;
pub type PoolConn = Object<DbConn>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn connect(url: Option<String>) -> Result<DbPool> {
    let url = url
        .or(env::var("DATABASE_URL").ok())
        .ok_or(anyhow!("Could not get a database URL!"))?;

    Ok(Pool::builder(AsyncDieselConnectionManager::<DbConn>::new(url)).build()?)
}

pub async fn migrate(pool: &DbPool) -> Result<()> {
    run_migrations(pool.get().await?, MIGRATIONS).await?;

    Ok(())
}

async fn run_migrations<A>(async_connection: A, migrations: EmbeddedMigrations) -> Result<()>
where
    A: AsyncConnection<Backend = Sqlite> + 'static,
{
    let mut async_wrapper: AsyncConnectionWrapper<A> =
        AsyncConnectionWrapper::from(async_connection);

    tokio::task::spawn_blocking(move || {
        async_wrapper.run_pending_migrations(migrations).unwrap();
    })
    .await
    .map_err(|e| Error::from(e))
}
