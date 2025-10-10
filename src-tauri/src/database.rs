use crate::photos;
use crate::types;

#[tauri::command]
pub async fn get_activities(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Activity>, String> {
    let mut activities = Vec::<types::Activity>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Activity")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        activities.push(types::Activity {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            icon: row.read::<&str, _>("icon").to_string(),
        });
    }

    Ok(activities)
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
            "INSERT INTO Activity VALUES ('{id}', '{name}', '{icon}')"
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn get_cameras(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Camera>, String> {
    let mut cameras = Vec::<types::Camera>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Camera")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        cameras.push(types::Camera {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    Ok(cameras)
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
pub async fn get_groups(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Group>, String> {
    let mut groups = Vec::<types::Group>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM PhotoGroup")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        groups.push(types::Group {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    Ok(groups)
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
pub async fn get_journals(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Journal>, String> {
    let mut journals = Vec::<types::Journal>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Journal")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        journals.push(types::Journal {
            id: row.read::<&str, _>("Id").to_string(),
            date: row.read::<&str, _>("date").to_string(),
            mood: row.read::<i64, _>("mood"),
            text: row.read::<&str, _>("text").to_string(),
            activities: row.read::<&str, _>("activities").to_string(),
            steps: row.read::<i64, _>("steps"),
            iv: row.read::<&str, _>("iv").to_string(),
        });
    }

    Ok(journals)
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
pub async fn get_layers(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Layer>, String> {
    let mut layers = Vec::<types::Layer>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Layer")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        layers.push(types::Layer {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
        });
    }

    Ok(layers)
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
pub async fn get_people(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Person>, String> {
    let mut people = Vec::<types::Person>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Person")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        people.push(types::Person {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            photo: row.read::<&str, _>("photo").to_string(),
            notes: row.read::<&str, _>("notes").to_string(),
            category: row.read::<&str, _>("category").to_string(),
        });
    }

    Ok(people)
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
pub async fn get_person_categories(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::PersonCategory>, String> {
    let mut categories = Vec::<types::PersonCategory>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM PersonCategory")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        categories.push(types::PersonCategory {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
        });
    }

    Ok(categories)
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
pub async fn get_places(state: tauri::State<'_, photos::PhotoState>) -> Result<Vec<types::Place>, String> {
    let mut places = Vec::<types::Place>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Place")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        places.push(types::Place {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            lat: row.read::<f64, _>("lat"),
            lng: row.read::<f64, _>("lng"),
            layer: row.read::<&str, _>("layer").to_string(),
            category: row.read::<&str, _>("category").to_string(),
            shape: row.read::<&str, _>("shape").to_string(),
            tags: row.read::<&str, _>("tags").to_string(),
            notes: row.read::<&str, _>("notes").to_string(),
        });
    }

    Ok(places)
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
pub async fn get_settings(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Setting>, String> {
    let mut settings = Vec::<types::Setting>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Setting")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        settings.push(types::Setting {
            id: row.read::<&str, _>("Id").to_string(),
            setting: row.read::<&str, _>("setting").to_string(),
            value: row.read::<i64, _>("value"),
        });
    }

    Ok(settings)
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
pub async fn get_shapes(state: tauri::State<'_, photos::PhotoState>) -> Result<Vec<types::Shape>, String> {
    let mut shapes = Vec::<types::Shape>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Shape")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        shapes.push(types::Shape {
            id: row.read::<&str, _>("Id").to_string(),
            shape_type: row.read::<&str, _>("type").to_string(),
            points: row.read::<&str, _>("points").to_string(),
            layer: row.read::<&str, _>("layer").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    Ok(shapes)
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
pub async fn get_tags(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::Tag>, String> {
    let mut tags = Vec::<types::Tag>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Tag")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        tags.push(types::Tag {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
            prereqs: row.read::<&str, _>("prereqs").to_string(),
            coreqs: row.read::<&str, _>("coreqs").to_string(),
            incompatible: row.read::<&str, _>("incompatible").to_string(),
        });
    }

    Ok(tags)
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
pub async fn get_wiki_pages(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec<types::WikiPage>, String> {
    let mut pages = Vec::<types::WikiPage>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM WikiPage")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        pages.push(types::WikiPage {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            content: row.read::<&str, _>("content").to_string(),
            iv: row.read::<&str, _>("iv").to_string(),
        });
    }

    Ok(pages)
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
pub async fn get_photos(state: tauri::State<'_, photos::PhotoState>) -> Result<Vec<types::Photo>, String> {
    let mut photos = Vec::<types::Photo>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Photo")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        photos.push(types::Photo {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            path: row.read::<&str, _>("path").to_string(),
            title: row.read::<&str, _>("title").to_string(),
            description: row.read::<&str, _>("description").to_string(),
            tags: row.read::<&str, _>("tags").to_string(),
            is_duplicate: row.read::<i64, _>("isDuplicate"),
            rating: row.read::<i64, _>("rating"),
            location: row.read::<&str, _>("location").to_string(),
            thumbnail: row.read::<&str, _>("thumbnail").to_string(),
            video: row.read::<i64, _>("video"),
            photo_group: row.read::<&str, _>("photoGroup").to_string(),
            date: row.read::<&str, _>("date").to_string(),
            raw: row.read::<i64, _>("raw"),
            people: row.read::<&str, _>("people").to_string(),
            hide_thumbnail: row.read::<i64, _>("hideThumbnail"),
            photographer: row.read::<&str, _>("photographer").to_string(),
            camera: row.read::<&str, _>("camera").to_string(),
        });
    }

    Ok(photos)
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
