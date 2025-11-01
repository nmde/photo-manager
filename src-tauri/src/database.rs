use crate::photos;
use crate::types;

pub fn esc(value: &String) -> String {
    String::from(value).replace("\"", "\"\"").to_string()
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
pub async fn create_layer(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Layer VALUES ('{0}', '{1}', '{2}')",
            esc(&id),
            esc(&name),
            esc(&color)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_layer_color(
    state: tauri::State<'_, photos::PhotoState>,
    layer: String,
    color: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Layer SET color='{0}' WHERE Id='{1}'",
            esc(&color),
            esc(&layer)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_person(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    photo: String,
    notes: String,
    category: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Person VALUES ('{0}', '{1}', '{2}', '{3}', '{4}')",
            esc(&id),
            esc(&name),
            esc(&photo),
            esc(&notes),
            esc(&category)
        ))
        .unwrap();

    state.people.lock().unwrap().insert(
        id.clone(),
        types::Person {
            id,
            name,
            photo,
            notes,
            category,
            photographer_count: 0,
            photo_count: 0,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn set_person_str(
    state: tauri::State<'_, photos::PhotoState>,
    person: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Person SET {property}='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&person)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_person_category(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO PersonCategory VALUES ('{0}', '{1}', '{2}')",
            esc(&id),
            esc(&name),
            esc(&color)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_place(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    lat: f64,
    lng: f64,
    layer: String,
    category: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Place VALUES ('{0}', '{1}', {lat}, {lng}, '{2}', '{3}', '', '', '')",
            esc(&id),
            esc(&name),
            esc(&layer),
            esc(&category)
        ))
        .unwrap();

    state.places.lock().unwrap().insert(
        id.clone(),
        types::Place {
            id: id.clone(),
            name,
            lat,
            lng,
            layer,
            category,
            shape: String::new(),
            tags: String::new(),
            notes: String::new(),
            count: 0,
        },
    );

    *state.newest_place.lock().unwrap() = id;

    Ok(())
}

#[tauri::command]
pub async fn set_place_str(
    state: tauri::State<'_, photos::PhotoState>,
    place: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Place SET {property}='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&place)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_place_position(
    state: tauri::State<'_, photos::PhotoState>,
    place: String,
    lat: f32,
    lng: f32,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Place SET (lat, lng)=({lat}, {lng}) WHERE Id='{0}'",
            esc(&place)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn delete_place(
    state: tauri::State<'_, photos::PhotoState>,
    place: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!("DELETE FROM Place WHERE Id='{0}'", esc(&place)))
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
pub async fn create_shape(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    shape_type: String,
    points: String,
    layer: String,
    name: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Shape VALUES ('{0}', '{1}', '{2}', '{3}', '{4}')",
            esc(&id),
            esc(&shape_type),
            esc(&points),
            esc(&layer),
            esc(&name)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_shape_str(
    state: tauri::State<'_, photos::PhotoState>,
    shape: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Shape SET {property}='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&shape)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn delete_shape(
    state: tauri::State<'_, photos::PhotoState>,
    shape: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!("DELETE FROM Shape WHERE Id='{0}'", esc(&shape)))
        .unwrap();
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
