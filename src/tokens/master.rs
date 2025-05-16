use super::models::MasterKey;
use crate::{db::DbPool, schema::master_keys, tokens::models_in::MasterKeyIn};
use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, dsl::insert_into};
use diesel_async::RunQueryDsl;
use random_string::charsets::ALPHANUMERIC;
use tracing::info;

pub async fn create_default_key(pool: &DbPool) -> Result<()> {
    let mut conn = pool.get().await?;

    if master_keys::table
        .filter(master_keys::is_init.eq(true))
        .select(MasterKey::as_select())
        .get_result(&mut conn)
        .await
        .is_err()
    {
        info!("Creating new master key...");

        let new = insert_into(master_keys::table)
            .values(MasterKeyIn {
                value: random_string::generate(32, ALPHANUMERIC),
                is_init: true,
            })
            .returning(MasterKey::as_returning())
            .get_result(&mut conn)
            .await?;

        info!(">> Your master key is: {}", new.value);
        info!(">> Write it down, it won't be displayed again!");
    }

    Ok(())
}
