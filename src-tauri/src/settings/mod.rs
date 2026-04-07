use anyhow::Result;
use diesel::{dsl::insert_into, query_dsl::methods::FilterDsl, update, ExpressionMethods};
use diesel_async::RunQueryDsl;

use crate::{models::Setting, schema::settings, PhotoManager};

pub mod api;

impl PhotoManager {
    pub async fn set_setting(&mut self, setting: &String, value: i32) -> Result<()> {
        let existing = self.get_setting(setting).await?;
        if existing.is_some() {
            update(settings::table.filter(settings::setting.eq(setting)))
                .set(settings::value.eq(value))
                .execute(&mut self.db)
                .await?;
        } else {
            insert_into(settings::table)
                .values(Setting {
                    setting: setting.clone(),
                    value,
                })
                .execute(&mut self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn get_setting(&mut self, setting: &String) -> Result<Option<Setting>> {
        Ok(settings::table
            .filter(settings::setting.eq(setting))
            .first::<Setting>(&mut self.db)
            .await
            .ok())
    }
}
