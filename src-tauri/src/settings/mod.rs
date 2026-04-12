use std::collections::HashMap;

use anyhow::Result;
use diesel::{dsl::insert_into, query_dsl::methods::FilterDsl, update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use serde::Serialize;
use strum::{Display, EnumString};
use tokio::sync::Mutex;

use crate::{
    app::{ensure_db, DB},
    models::Setting,
    schema::settings,
};

pub mod api;

lazy_static! {
    pub static ref SETTINGS: Mutex::<HashMap<Settings, Setting>> = Mutex::new(HashMap::new());
}

#[derive(Display, EnumString, Eq, Hash, PartialEq)]
pub enum Settings {
    #[strum(ascii_case_insensitive)]
    Theme,
    #[strum(ascii_case_insensitive)]
    Colors,
}

#[derive(Display, EnumString, Serialize)]
pub enum ThemeSetting {
    #[strum(ascii_case_insensitive)]
    Dark,
    #[strum(ascii_case_insensitive)]
    Light,
}

pub async fn set_setting(setting: Settings, value: String) -> Result<()> {
    ensure_db().await?;

    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();
    let mut settings_cache = SETTINGS.lock().await;

    if settings_cache.contains_key(&setting) {
        update(settings::table.filter(settings::setting.eq(setting.to_string())))
            .set(settings::value.eq(value.clone()))
            .execute(conn)
            .await?;
        settings_cache.get_mut(&setting).unwrap().value = value;
    } else {
        let new_setting = Setting {
            setting: setting.to_string(),
            value,
        };
        insert_into(settings::table)
            .values(new_setting.clone())
            .execute(conn)
            .await?;
        settings_cache.insert(setting, new_setting);
    }

    Ok(())
}

pub async fn get_setting(setting: Settings) -> Result<String> {
    ensure_db().await?;
    Ok(settings::table
        .filter(settings::setting.eq(setting.to_string()))
        .first::<Setting>(DB.lock().await.as_mut().unwrap())
        .await?
        .value)
}

pub async fn get_colors() -> Result<Vec<String>> {
    let set_colors = get_setting(Settings::Colors).await;
    if set_colors.is_ok() {
        return Ok(set_colors
            .unwrap()
            .split(",")
            .map(|c| c.to_string())
            .collect::<Vec<String>>());
    }
    // Some default colors based on material design
    Ok(vec![
        "#F44336", "#E91E63", "#9C27B0", "#673AB7", "#3F51B5", "#2196F3", "#03A9F4", "#00BCD4",
        "#009688", "#4CAF50", "#8BC34A", "#CDDC39", "#FFEB3B", "#FFC107", "#FF9800", "#FF5722",
    ]
    .into_iter()
    .map(|c| c.to_string())
    .collect::<Vec<String>>())
}

pub async fn promote_color(color: &String) -> Result<Vec<String>> {
    let mut colors = get_colors().await?;
    let idx = colors.iter().position(|c| c == color);
    if idx.is_some() {
        colors.remove(idx.unwrap());
    }
    colors.insert(0, color.clone());
    set_setting(Settings::Colors, colors.join(",")).await?;
    Ok(colors)
}

pub async fn add_color(color: &String) -> Result<Vec<String>> {
    let mut colors = get_colors().await?;
    colors.insert(0, color.clone());
    set_setting(Settings::Colors, colors.join(",")).await?;
    Ok(colors)
}
