use crate::photos;
use crate::types;

pub fn esc(value: &String) -> String {
    String::from(value).replace("\'", "\'\'").to_string()
}

#[tauri::command]
pub async fn create_activity(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    icon: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Activity VALUES ('{0}', '{1}', '{2}')",
            esc(&id),
            esc(&name),
            esc(&icon)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_camera(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Camera VALUES ('{0}', '{1}')",
            esc(&id),
            esc(&name)
        ))
        .unwrap();

    state
        .cameras
        .lock()
        .unwrap()
        .insert(id.clone(), types::Camera { id, name, count: 0 });

    Ok(())
}

#[tauri::command]
pub async fn create_group(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO PhotoGroup VALUES ('{0}', '{1}')",
            esc(&id),
            esc(&name)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_journal_entry(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    date: String,
    mood: i32,
    text: String,
    activities: String,
    steps: i32,
    iv: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO JournalEntry VALUES ('{0}', '{1}', {mood}, '{2}', '{3}', {steps}, '{4}')",
            esc(&id),
            esc(&date),
            esc(&text),
            esc(&activities),
            esc(&iv)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_journal_str(
    state: tauri::State<'_, photos::PhotoState>,
    journal: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Journal SET {property}='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&journal)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_journal_mood(
    state: tauri::State<'_, photos::PhotoState>,
    journal: String,
    mood: i32,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Journal SET mood={mood} WHERE Id='{0}'",
            esc(&journal)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_setting(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    setting: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare(format!(
            "SELECT * FROM Setting WHERE setting='{0}'",
            esc(&setting)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    if rows.count() == 0 {
        connection
            .execute(format!(
                "INSERT INTO Setting VALUES ('{0}', '{1}', '{2}')",
                esc(&id),
                esc(&setting),
                esc(&value)
            ))
            .unwrap();
    } else {
        connection
            .execute(format!(
                "UPDATE Setting SET value='{0}' WHERE setting='{1}'",
                esc(&value),
                esc(&setting)
            ))
            .unwrap();
    }

    Ok(())
}

#[tauri::command]
pub async fn create_wiki_page(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    content: String,
    iv: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO WikiPage VALUES ('{0}', '{1}', '{2}', '{3}')",
            esc(&id),
            esc(&name),
            esc(&content),
            esc(&iv)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_wiki_str(
    state: tauri::State<'_, photos::PhotoState>,
    page: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE WikiPage SET {property}='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&page)
        ))
        .unwrap();
    Ok(())
}
