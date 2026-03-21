use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::{esc, photos::PhotoState, ApiError};

#[derive(Clone, Serialize)]
pub struct Setting {
    pub setting: String,
    pub value: i64,
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, PhotoState>,
    setting: String,
    value: i64,
) -> Result<(), ApiError> {
    let conn = state.db.lock().unwrap();
    let mut state_settings = state.settings.lock().unwrap();

    if state_settings.contains_key(&setting) {
        conn.execute(format!(
            "UPDATE Setting SET value={0} WHERE setting='{1}'",
            value,
            esc(&setting)
        ))?;
    } else {
        conn.execute(format!(
            "INSERT INTO Setting VALUES ('{0}', '{1}', {2})",
            Uuid::new_v4(),
            setting,
            value
        ))?;
        state_settings.insert(setting.clone(), Setting { setting, value });
    }

    Ok(())
}

#[tauri::command]
pub fn get_setting(state: State<'_, PhotoState>, setting: String) -> Option<Setting> {
    let state_settings = state.settings.lock().unwrap();
    let s = state_settings.get(&setting);
    if s.is_some() {
        return Some(s.unwrap().clone());
    }
    None
}
