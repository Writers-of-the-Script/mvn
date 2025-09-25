use anyhow::{Error, Result, anyhow};
use diesel::{ConnectionError, ConnectionResult, backend::Backend};
use diesel_async::{
    AsyncConnection, AsyncPgConnection,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{
        AsyncDieselConnectionManager, ManagerConfig, RecyclingMethod,
        deadpool::{Object, Pool},
    },
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use futures_util::{FutureExt, future::BoxFuture};
use std::env;

pub type DbConn = AsyncPgConnection;
pub type DbPool = Pool<DbConn>;
pub type PoolConn = Object<DbConn>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn connect(url: Option<String>) -> Result<DbPool> {
    let url = url
        .or(env::var("DATABASE_URL").ok())
        .ok_or(anyhow!("Could not get a database URL!"))?;

    let mut config = ManagerConfig::default();

    config.custom_setup = Box::new(establish_connection);
    config.recycling_method = RecyclingMethod::Verified;

    Ok(Pool::builder(AsyncDieselConnectionManager::new_with_config(url, config)).build()?)
}

pub async fn connect_single(url: Option<String>) -> Result<DbConn> {
    let url = url
        .or(env::var("DATABASE_URL").ok())
        .ok_or(anyhow!("Could not get a database URL!"))?;

    Ok(establish_connection(&url).await?)
}

fn establish_connection(config: &'_ str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        #[cfg(not(debug_assertions))]
        let rustls_config = <rustls::ClientConfig as rustls_platform_verifier::ConfigVerifierExt>::with_platform_verifier().unwrap();

        #[cfg(debug_assertions)]
        let rustls_config =
            crate::tls::CustomVerifiers::with_ignore_hosts_verifier(rustls::ClientConfig::builder())
                .unwrap()
                .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);

        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;

        AsyncPgConnection::try_from_client_and_connection(client, conn).await
    };

    fut.boxed()
}

pub async fn migrate(pool: &DbPool) -> Result<()> {
    run_migrations(pool.get().await?, MIGRATIONS).await?;

    Ok(())
}

async fn run_migrations<A: AsyncConnection<Backend = B> + 'static, B: Backend>(
    conn: A,
    migrations: EmbeddedMigrations,
) -> Result<()>
where
    AsyncConnectionWrapper<A>: MigrationHarness<B>,
{
    let mut async_wrapper: AsyncConnectionWrapper<A> = AsyncConnectionWrapper::from(conn);

    tokio::task::spawn_blocking(move || {
        async_wrapper.run_pending_migrations(migrations).unwrap();
    })
    .await
    .map_err(|e| Error::from(e))
}
