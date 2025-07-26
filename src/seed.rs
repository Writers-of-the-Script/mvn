use crate::{
    db::DbPool,
    router::models::{RouteData, RouteDataIn},
    schema::{master_keys, route_data},
    tokens::{models::MasterKey, models_in::MasterKeyIn},
};
use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, dsl::insert_into};
use diesel_async::RunQueryDsl;
use random_string::charsets::ALPHANUMERIC;
use tracing::info;

pub async fn seed_db(pool: &DbPool, master_key: Option<String>) -> Result<()> {
    create_default_key(pool, master_key).await?;
    create_default_route_data(pool).await?;

    Ok(())
}

async fn create_default_key(pool: &DbPool, master_key: Option<String>) -> Result<()> {
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
                value: master_key
                    .clone()
                    .unwrap_or_else(|| random_string::generate(32, ALPHANUMERIC)),
                is_init: true,
            })
            .returning(MasterKey::as_returning())
            .get_result(&mut conn)
            .await?;

        if master_key.is_none() {
            info!(">> Your master key is: {}", new.value);
            info!(">> Write it down, it won't be displayed again!");
        }
    }

    Ok(())
}

async fn create_default_route_data(pool: &DbPool) -> Result<()> {
    let mut conn = pool.get().await?;

    if route_data::table
        .filter(route_data::path.eq("/"))
        .select(RouteData::as_select())
        .get_result(&mut conn)
        .await
        .is_err()
    {
        info!("Creating '/' route data...");

        insert_into(route_data::table)
            .values(RouteDataIn {
                path: "/".into(),
                visibility: 0,
            })
            .returning(RouteData::as_returning())
            .get_result(&mut conn)
            .await?;

        info!("Created '/' route data!");
    }

    Ok(())
}
