use crate::photos;

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
            "INSERT INTO Activity VALUES ('{id}', '{name}', '{icon}')"
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
        .execute(format!("INSERT INTO Camera VALUES ('{id}', '{name}')"))
        .unwrap();
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
        .execute(format!("INSERT INTO PhotoGroup VALUES ('{id}', '{name}')"))
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
    state.db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO JournalEntry VALUES ('{id}', '{date}', {mood}, '{text}', '{activities}', {steps}, '{iv}')"
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
            "UPDATE Journal SET {property}='{value}' WHERE Id='{journal}'"
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
            "UPDATE Journal SET mood='{mood}' WHERE Id='{journal}'"
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
            "INSERT INTO Layer VALUES ('{id}', '{name}', '{color}')"
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
            "UPDATE Layer SET color='{color}' WHERE Id='{layer}'"
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
            "INSERT INTO Person VALUES ('{id}', '{name}', '{photo}', '{notes}', '{category}')"
        ))
        .unwrap();
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
            "UPDATE Person SET {property}='{value}' WHERE Id='{person}'"
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
            "INSERT INTO PersonCategory VALUES ('{id}', '{name}', '{color}')"
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_place(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    lat: i32,
    lng: i32,
    layer: String,
    category: String,
) -> Result<(), String> {
    state.db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Place VALUES ('{id}', '{name}', {lat}, {lng}, '{layer}', '{category}'. '', '', '')"
        ))
        .unwrap();
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
            "UPDATE Place SET {property}='{value}' WHERE Id='{place}'"
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_place_position(
    state: tauri::State<'_, photos::PhotoState>,
    place: String,
    lat: i32,
    lng: i32,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Place SET (lat, lng)=({lat}, {lng}) WHERE Id='{place}'"
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
        .execute(format!("DELETE FROM Place WHERE Id='{place}'"))
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
        .prepare(format!("SELECT * FROM Setting WHERE setting='{setting}'"))
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    if rows.count() == 0 {
        connection
            .execute(format!(
                "INSERT INTO Setting VALUES ('{id}', '{setting}', '{value}')"
            ))
            .unwrap();
    } else {
        connection
            .execute(format!(
                "UPDATE Setting SET value='{value}' WHERE setting='{setting}'"
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
            "INSERT INTO Shape VALUES ('{id}', '{shape_type}', '{points}', '{layer}'. '{name}')"
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
            "UPDATE Shape SET {property}='{value}' WHERE Id='{shape}'"
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
        .execute(format!("DELETE FROM Shape WHERE Id='{shape}'"))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_tag(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Tag VALUES ('{id}', '{name}', '', '', '', '')"
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_tag_str(
    state: tauri::State<'_, photos::PhotoState>,
    tag: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Tag SET {property}='{value}' WHERE Id='{tag}'"
        ))
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
            "INSERT INTO WikiPage VALUES ('{id}', '{name}', '{content}', '{iv}')"
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
            "UPDATE WikiPage SET {property}='{value}' WHERE Id='{page}'"
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_photo_str(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Photo SET {property}='{value}' WHERE Id='{photo}'"
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_photo_rating(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    rating: i32,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Photo SET rating={rating} WHERE Id='{photo}'"
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_photo_bool(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    property: String,
    value: bool,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Photo SET {property}={value} WHERE Id='{photo}'"
        ))
        .unwrap();
    Ok(())
}
