use sqlite::Connection;
use std::sync::Mutex;

pub struct PhotoState {
    db: Mutex<Connection>,
}

impl Default for PhotoState {
    fn default() -> Self {
        Self {
            db: Mutex::new(sqlite::open(":memory:").ok().unwrap()),
        }
    }
}

#[tauri::command]
pub async fn set_working_dir(
    state: tauri::State<'_, PhotoState>,
    path: String,
) -> Result<(), String> {
    let conn = sqlite::open(format!("{path}/photos.db")).ok().unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS Activity (Id TEXT PRIMARY KEY, name TEXT, icon TEXT)")
        .unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Camera (Id TEXT PRIMARY KEY, name TEXT)")
        .unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Journal (Id TEXT PRIMARY KEY, date TEXT, mood INTEGER, text TEXT, activities TEXT, steps INTEGER, iv TEXT)").unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Layer (Id TEXT PRIMARY KEY, name TEXT, color TEXT)")
        .unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Person (Id TEXT PRIMARY KEY, name TEXT, photo TEXT, notes TEXT, category TEXT)").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS PersonCategory (Id TEXT PRIMARY KEY, name TEXT, color TEXT)",
    )
    .unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Photo (Id TEXT , name TEXT PRIMARY KEY, path TEXT, title TEXT, description TEXT, tags TEXT, isDuplicate INTEGER, rating INTEGER, location TEXT, thumbnail TEXT, video INTEGER, photoGroup TEXT, date TEXT, raw INTEGER, people TEXT, hideThumbnail INTEGER, photographer TEXT, camera TEXT)").unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS PhotoGroup (Id TEXT PRIMARY KEY, name TEXT)")
        .unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Place (Id TEXT PRIMARY KEY, name TEXT, lat INTEGER, lng INTEGER, layer TEXT, category TEXT, shape TEXT, tags TEXT, notes TEXT)").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Setting (Id TEXT , setting TEXT PRIMARY KEY, value INTEGER)",
    )
    .unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Shape (Id TEXT PRIMARY KEY, type TEXT, points TEXT, layer TEXT, name TEXT)").unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS Tag (Id TEXT , name TEXT PRIMARY KEY, color TEXT, prereqs TEXT, coreqs TEXT, incompatible TEXT)").unwrap();

    *state.db.lock().unwrap() = conn;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct Activity {
    id: String,
    name: String,
    icon: String,
}

#[tauri::command]
pub async fn get_activities(state: tauri::State<'_, PhotoState>) -> Result<Vec<Activity>, String> {
    let mut activities = Vec::<Activity>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Activity")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        activities.push(Activity {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            icon: row.read::<&str, _>("icon").to_string(),
        });
    }

    Ok(activities)
}

#[tauri::command]
pub async fn create_activity(
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Camera {
    id: String,
    name: String,
}

#[tauri::command]
pub async fn get_cameras(state: tauri::State<'_, PhotoState>) -> Result<Vec<Camera>, String> {
    let mut cameras = Vec::<Camera>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Camera")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        cameras.push(Camera {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    Ok(cameras)
}

#[tauri::command]
pub async fn create_camera(
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Group {
    id: String,
    name: String,
}

#[tauri::command]
pub async fn get_groups(state: tauri::State<'_, PhotoState>) -> Result<Vec<Group>, String> {
    let mut groups: Vec<Group> = Vec::<Group>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM PhotoGroup")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        groups.push(Group {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    Ok(groups)
}

#[tauri::command]
pub async fn create_group(
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Journal {
    id: String,
    date: String,
    mood: i64,
    text: String,
    activities: String,
    steps: i64,
    iv: String,
}

#[tauri::command]
pub async fn get_journals(state: tauri::State<'_, PhotoState>) -> Result<Vec<Journal>, String> {
    let mut journals = Vec::<Journal>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Journal")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        journals.push(Journal {
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Layer {
    id: String,
    name: String,
    color: String,
}

#[tauri::command]
pub async fn get_layers(state: tauri::State<'_, PhotoState>) -> Result<Vec<Layer>, String> {
    let mut layers: Vec<Layer> = Vec::<Layer>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Layer")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        layers.push(Layer {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
        });
    }

    Ok(layers)
}

#[tauri::command]
pub async fn create_layer(
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Person {
    id: String,
    name: String,
    photo: String,
    notes: String,
    category: String,
}

#[tauri::command]
pub async fn get_people(state: tauri::State<'_, PhotoState>) -> Result<Vec<Person>, String> {
    let mut people = Vec::<Person>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Person")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        people.push(Person {
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct PersonCategory {
    id: String,
    name: String,
    color: String,
}

#[tauri::command]
pub async fn get_person_categories(
    state: tauri::State<'_, PhotoState>,
) -> Result<Vec<PersonCategory>, String> {
    let mut categories = Vec::<PersonCategory>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM PersonCategories")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        categories.push(PersonCategory {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
        });
    }

    Ok(categories)
}

#[tauri::command]
pub async fn create_person_category(
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Place {
    id: String,
    name: String,
    lat: i64,
    lng: i64,
    layer: String,
    category: String,
    shape: String,
    tags: String,
    notes: String,
}

#[tauri::command]
pub async fn get_places(state: tauri::State<'_, PhotoState>) -> Result<Vec<Place>, String> {
    let mut places = Vec::<Place>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Place")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        places.push(Place {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            lat: row.read::<i64, _>("lat"),
            lng: row.read::<i64, _>("lng"),
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Setting {
    id: String,
    setting: String,
    value: String,
}

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, PhotoState>) -> Result<Vec<Setting>, String> {
    let mut settings = Vec::<Setting>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Setting")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        settings.push(Setting {
            id: row.read::<&str, _>("Id").to_string(),
            setting: row.read::<&str, _>("setting").to_string(),
            value: row.read::<&str, _>("value").to_string(),
        });
    }

    Ok(settings)
}

#[tauri::command]
pub async fn set_setting(
    state: tauri::State<'_, PhotoState>,
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
        connection.execute(format!(
            "INSERT INTO Settings VALUES ('{id}', '{setting}', '{value}')"
        ));
    } else {
        connection.execute(format!(
            "UPDATE Settings SET value='{value}' WHERE setting='{setting}'"
        ));
    }

    Ok(())
}

#[derive(serde::Serialize)]
pub struct Shape {
    id: String,
    shape_type: String,
    points: String,
    layer: String,
    name: String,
}

#[tauri::command]
pub async fn get_shapes(state: tauri::State<'_, PhotoState>) -> Result<Vec<Shape>, String> {
    let mut shapes: Vec<Shape> = Vec::<Shape>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Shape")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        shapes.push(Shape {
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Tag {
    id: String,
    name: String,
    color: String,
    prereqs: String,
    coreqs: String,
    incompatible: String,
}

#[tauri::command]
pub async fn get_tags(state: tauri::State<'_, PhotoState>) -> Result<Vec<Tag>, String> {
    let mut tags = Vec::<Tag>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Tag")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        tags.push(Tag {
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
pub async fn set_tag_str(
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct WikiPage {
    id: String,
    name: String,
    content: String,
    iv: String,
}

#[tauri::command]
pub async fn get_wiki_pages(state: tauri::State<'_, PhotoState>) -> Result<Vec<WikiPage>, String> {
    let mut pages = Vec::<WikiPage>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Journal")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        pages.push(WikiPage {
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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

#[derive(serde::Serialize)]
pub struct Photo {
    id: String,
    name: String,
    path: String,
    title: String,
    description: String,
    tags: String,
    is_duplicate: i64,
    rating: i64,
    location: String,
    thumbnail: String,
    video: i64,
    photo_group: String,
    date: String,
    raw: i64,
    people: String,
    hide_thumbnail: i64,
    photographer: String,
    camera: String,
}

#[tauri::command]
pub async fn get_photos(state: tauri::State<'_, PhotoState>) -> Result<Vec<Photo>, String> {
    let mut photos = Vec::<Photo>::new();

    let connection = state.db.lock().unwrap();
    let rows = connection
        .prepare("SELECT * FROM Photo")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    for row in rows {
        photos.push(Photo {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            path: row.read::<&str, _>("path").to_string(),
            title: row.read::<&str, _>("title").to_string(),
            description: row.read::<&str, _>("description").to_string(),
            tags: row.read::<&str, _>("tags").to_string(),
            is_duplicate: row.read::<i64, _>("isDuplicate"),
            rating: row.read::<i64, _>("rating"),
            location: row.read::<&str, _>("location").to_string(),
            thumbnail: row.read::<&str, _>("location").to_string(),
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
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
