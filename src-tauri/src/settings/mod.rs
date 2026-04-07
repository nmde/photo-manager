use diesel::{dsl::insert_into, query_dsl::methods::FilterDsl, update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use log::debug;
use tauri::State;

use crate::{models::Setting, photos::PhotoState, schema::settings, ApiError};

#[tauri::command]
pub async fn set_setting(
    state: State<'_, PhotoState>,
    setting: String,
    value: i32,
) -> Result<(), ApiError> {
    debug!("Setting setting {setting} to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    let mut state_settings = state.settings.lock().await;

    if state_settings.contains_key(&setting) {
        update(settings::table.filter(settings::setting.eq(setting.clone())))
            .set(settings::value.eq(value))
            .execute(conn)
            .await?;
    } else {
        let new_setting = Setting {
            setting: setting.clone(),
            value,
        };
        insert_into(settings::table)
            .values(new_setting.clone())
            .execute(conn)
            .await?;
        state_settings.insert(setting.clone(), new_setting);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_setting(
    state: State<'_, PhotoState>,
    setting: String,
) -> Result<Option<Setting>, ApiError> {
    let state_settings = state.settings.lock().await;
    let s = state_settings.get(&setting);
    if s.is_some() {
        return Ok(Some(s.unwrap().clone()));
    }
    Ok(None)
}
