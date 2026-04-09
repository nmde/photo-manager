use anyhow::Result;
use diesel::{dsl::insert_into, query_dsl::methods::FilterDsl, update, ExpressionMethods};
use diesel_async::RunQueryDsl;

use crate::{
    app::{ensure_db, DB},
    models::Setting,
    schema::settings,
};

pub mod api;

pub async fn set_setting(setting: &String, value: i32) -> Result<()> {
    ensure_db().await?;
    let existing = get_setting(setting).await?;
    if existing.is_some() {
        update(settings::table.filter(settings::setting.eq(setting)))
            .set(settings::value.eq(value))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    } else {
        insert_into(settings::table)
            .values(Setting {
                setting: setting.clone(),
                value,
            })
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    }

    Ok(())
}

pub async fn get_setting(setting: &String) -> Result<Option<Setting>> {
    ensure_db().await?;
    Ok(settings::table
        .filter(settings::setting.eq(setting))
        .first::<Setting>(DB.lock().await.as_mut().unwrap())
        .await
        .ok())
}
