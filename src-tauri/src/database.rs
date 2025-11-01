use crate::photos;
use crate::types;
use std::collections::HashSet;

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
            "INSERT INTO Tag VALUES ('{0}', '{1}', '', '', '', '')",
            esc(&id),
            esc(&name)
        ))
        .unwrap();

    state.tags.lock().unwrap().insert(
        name.clone(),
        types::Tag {
            id,
            name,
            color: String::new(),
            prereqs: Vec::<String>::new(),
            coreqs: Vec::<String>::new(),
            incompatible: Vec::<String>::new(),
            count: 0,
        },
    );

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
            "UPDATE Tag SET {property}='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&tag)
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
            "UPDATE Photo SET {property}='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&photo)
        ))
        .unwrap();
    Ok(())
}

fn update_photographer_count(
    state: &tauri::State<'_, photos::PhotoState>,
    old_photographer: &String,
    new_photographer: &String,
) {
    let mut state_people = state.people.lock().unwrap();

    if old_photographer != new_photographer {
        if old_photographer.len() > 0 {
            state_people
                .get_mut(old_photographer)
                .unwrap()
                .photographer_count -= 1;
        }
        state_people
            .get_mut(new_photographer)
            .unwrap()
            .photographer_count += 1;
    }
}

#[tauri::command]
pub async fn set_photographer(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(&photo)))
        .unwrap()
        .into_iter()
        .map(|row| photos::row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_photographer_count(&state, &existing.photographer, &value);

    connection
        .execute(format!(
            "UPDATE Photo SET Photographer='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE PhotoGroup='{group}' AND Id!='{0}'",
                esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_photographer_count(&state, &photo.photographer, &value);
            let id = esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET Photographer='{0}' WHERE Id='{id}'",
                    esc(&value)
                ))
                .unwrap();
        }
    }
    Ok(())
}

fn update_people_counts(
    state: &tauri::State<'_, photos::PhotoState>,
    old_people: &Vec<String>,
    new_people: &Vec<String>,
) {
    let mut state_people = state.people.lock().unwrap();

    if old_people.len() > 0 {
        for person in old_people.iter().filter(|p| !new_people.contains(p)) {
            state_people.get_mut(person).unwrap().photo_count -= 1;
        }
        for person in new_people.iter().filter(|p| !old_people.contains(p)) {
            state_people.get_mut(person).unwrap().photo_count += 1;
        }
    } else {
        for person in new_people {
            state_people.get_mut(person).unwrap().photo_count += 1;
        }
    }
}

#[tauri::command]
pub async fn set_photo_people(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(&photo)))
        .unwrap()
        .into_iter()
        .map(|row| photos::row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_people_counts(&state, &existing.people, &value);

    let joined_people = esc(&value.join(","));
    connection
        .execute(format!(
            "UPDATE Photo SET People='{joined_people}' WHERE Id='{0}'",
            esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE PhotoGroup='{group}' AND Id!='{0}'",
                esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_people_counts(&state, &photo.people, &value);
            let id = esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET People='{joined_people}' WHERE Id='{id}'"
                ))
                .unwrap();
        }
    }

    Ok(())
}

fn update_camera_count(
    state: &tauri::State<'_, photos::PhotoState>,
    old_camera: &String,
    new_camera: &String,
) {
    let mut state_cameras = state.cameras.lock().unwrap();

    if old_camera != new_camera {
        if old_camera.len() > 0 {
            state_cameras.get_mut(old_camera).unwrap().count -= 1;
        }
        state_cameras.get_mut(new_camera).unwrap().count += 1;
    }
}

#[tauri::command]
pub async fn set_photo_camera(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(&photo)))
        .unwrap()
        .into_iter()
        .map(|row| photos::row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_camera_count(&state, &existing.camera, &value);

    connection
        .execute(format!(
            "UPDATE Photo SET Camera='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE PhotoGroup='{group}' AND Id!='{0}'",
                esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_camera_count(&state, &photo.camera, &value);
            let id = esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET Camera='{0}' WHERE Id='{id}'",
                    esc(&value)
                ))
                .unwrap();
        }
    }

    Ok(())
}

fn update_location_count(
    state: &tauri::State<'_, photos::PhotoState>,
    old_location: &String,
    new_location: &String,
) {
    let mut state_places = state.places.lock().unwrap();

    if old_location != new_location {
        if old_location.len() > 0 {
            state_places.get_mut(old_location).unwrap().count -= 1;
        }
        state_places.get_mut(new_location).unwrap().count += 1;
    }
}

#[tauri::command]
pub async fn set_photo_location(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(&photo)))
        .unwrap()
        .into_iter()
        .map(|row| photos::row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_location_count(&state, &existing.location, &value);

    connection
        .execute(format!(
            "UPDATE Photo SET Location='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE PhotoGroup='{group}' AND Id!='{0}'",
                esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_location_count(&state, &photo.location, &value);
            let id = esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET Location='{0}' WHERE Id='{id}'",
                    esc(&value),
                ))
                .unwrap();
        }
    }

    Ok(())
}

fn update_tag_counts(
    state: &tauri::State<'_, photos::PhotoState>,
    old_tags: &Vec<String>,
    new_tags: &Vec<String>,
) {
    let mut state_tags = state.tags.lock().unwrap();

    if old_tags.len() > 0 {
        for tag in old_tags.iter().filter(|t| !new_tags.contains(t)) {
            state_tags.get_mut(tag).unwrap().count -= 1;
        }
        for tag in new_tags.iter().filter(|t| !old_tags.contains(t)) {
            state_tags.get_mut(tag).unwrap().count += 1;
        }
    } else {
        for tag in new_tags {
            state_tags.get_mut(tag).unwrap().count += 1;
        }
    }
}

#[tauri::command]
pub async fn set_photo_tags(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(&photo)))
        .unwrap()
        .into_iter()
        .map(|row| photos::row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    photos::create_new_tags(&state, &value);
    update_tag_counts(&state, &existing.tags, &value);

    let validation = photos::validate_tags(&state.tags.lock().unwrap(), &value);

    let mut state_photos = state.photos.lock().unwrap();
    match state_photos.binary_search_by_key(&photo, |p| p.name.clone()) {
        Ok(pos) => {
            state_photos[pos].tags = value.clone();
            state_photos[pos].valid_tags = validation.is_valid;
            state_photos[pos].validation_msg = validation.message;
        }
        Err(_pos) => {}
    }

    let tags_str = esc(&value.join(","));
    connection
        .execute(format!(
            "UPDATE Photo SET Tags='{tags_str}' WHERE Id='{0}'",
            esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE PhotoGroup='{group}' AND Id!='{0}'",
                esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_tag_counts(&state, &photo.tags, &value);
            let id = esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET Tags='{tags_str}' WHERE Id='{id}'"
                ))
                .unwrap();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_date(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(&photo)))
        .unwrap()
        .into_iter()
        .map(|row| photos::row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    connection
        .execute(format!(
            "UPDATE Photo SET Date='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = esc(&existing.photo_group);
        connection
            .execute(format!(
                "UPDATE Photo SET Date='{0}' WHERE PhotoGroup='{group}' AND Id!='{1}'",
                esc(&value),
                esc(&photo)
            ))
            .unwrap();
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_group(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    if value.len() == 0 {
        connection
            .execute(format!(
                "UPDATE Photo SET PhotoGroup='' WHERE Id='{0}'",
                esc(&photo)
            ))
            .unwrap();
    } else {
        let results = connection
            .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(&photo)))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()));
        let existing = results.last().unwrap();

        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE PhotoGroup='{0}'",
                esc(&value)
            ))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()));

        let mut collected_tags = HashSet::new();
        let mut collected_people = HashSet::new();
        let mut collected_location = existing.location.clone();
        let mut collected_photographer = existing.photographer.clone();
        let mut collected_camera = existing.camera.clone();
        let mut collected_date = existing.date;
        for photo in in_group {
            for tag in &photo.tags {
                collected_tags.insert(tag.to_string());
            }
            for person in &photo.people {
                collected_people.insert(person.to_string());
            }
            if collected_location.len() == 0 && photo.location.len() > 0 {
                collected_location = photo.location;
            }
            if collected_photographer.len() == 0 && photo.photographer.len() > 0 {
                collected_photographer = photo.photographer;
            }
            if collected_camera.len() == 0 && photo.camera.len() > 0 {
                collected_camera = photo.camera;
            }
            if collected_date.len() == 0 && photo.date.len() > 0 {
                collected_date = photo.date;
            }
        }
        for tag in &existing.tags {
            collected_tags.insert(tag.to_string());
        }
        for person in &existing.people {
            collected_people.insert(person.to_string());
        }

        let group_tags = collected_tags.into_iter().collect::<Vec<String>>();
        let group_people = collected_people.into_iter().collect::<Vec<String>>();

        photos::create_new_tags(&state, &group_tags);
        update_tag_counts(&state, &existing.tags, &group_tags);
        update_people_counts(&state, &existing.people, &group_people);
        update_location_count(&state, &existing.location, &collected_location);
        update_photographer_count(&state, &existing.photographer, &collected_photographer);
        update_camera_count(&state, &existing.camera, &collected_camera);

        let validation = photos::validate_tags(&state.tags.lock().unwrap(), &group_tags);
        let mut state_photos = state.photos.lock().unwrap();
        // TODO - this needs to update all the other properties for every photo in the group too
        match state_photos.binary_search_by_key(&photo, |p| p.name.clone()) {
            Ok(pos) => {
                state_photos[pos].valid_tags = validation.is_valid;
                state_photos[pos].validation_msg = validation.message;
            }
            Err(_pos) => {}
        }

        let group_tags_str = esc(&group_tags.join(","));
        let group_people_str = esc(&group_people.join(","));
        collected_location = esc(&collected_location);
        collected_photographer = esc(&collected_photographer);
        collected_camera = esc(&collected_camera);
        collected_date = esc(&collected_date);
        connection.execute(
            format!(
                "UPDATE Photo SET Group='{0}', Tags='{group_tags_str}', People='{group_people_str}', Location='{collected_location}', Photographer='{collected_photographer}', Camera='{collected_camera}', Date='{collected_date}' WHERE Id='{1}'",
                esc(&value),
                esc(&photo)
            )
        ).unwrap();

        for photo in connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE PhotoGroup='{0}' AND Id!='{1}'",
                esc(&value),
                esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| photos::row_to_photo(&state, row.unwrap()))
        {
            update_tag_counts(&state, &photo.tags, &group_tags);
            update_people_counts(&state, &photo.people, &group_people);
            update_location_count(&state, &photo.location, &collected_location);
            update_photographer_count(&state, &photo.photographer, &collected_photographer);
            update_camera_count(&state, &photo.camera, &collected_camera);

            let id = esc(&photo.id);
            connection.execute(format!("UPDATE Photo SET Tags='{group_tags_str}', People='{group_people_str}', Location='{collected_location}', Photographer='{collected_photographer}', Camera='{collected_camera}', Date='{collected_date}' WHERE Id='{id}'")).unwrap();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_rating(
    state: tauri::State<'_, photos::PhotoState>,
    photo: String,
    rating: i64,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Photo SET rating={rating} WHERE Id='{0}'",
            esc(&photo)
        ))
        .unwrap();

    // TODO - make sure the state is updated for every db function
    let mut state_photos = state.photos.lock().unwrap();
    match state_photos.binary_search_by_key(&photo, |p| p.id.clone()) {
        Ok(pos) => {
            state_photos[pos].rating = rating;
        }
        Err(_pos) => {}
    }

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
            "UPDATE Photo SET {property}={value} WHERE Id='{0}'",
            esc(&photo)
        ))
        .unwrap();
    Ok(())
}
