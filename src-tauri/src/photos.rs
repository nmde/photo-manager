use crate::database;
use crate::tags;
use crate::types;
use regex::Regex;
use sqlite::Connection;
use sqlite::Row;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
use unique_id::string::StringGenerator;
use unique_id::Generator;
use walkdir::WalkDir;

const PHOTO_MANAGER_VERSION: i64 = 1;

pub struct PhotoState {
    pub db: Mutex<Connection>,
    pub photos: Mutex<Vec<types::Photo>>,
    pub people: Mutex<HashMap<String, types::Person>>,
    pub cameras: Mutex<HashMap<String, types::Camera>>,
    pub places: Mutex<HashMap<String, types::Place>>,
    pub newest_place: Mutex<String>,
    pub tags: Mutex<HashMap<String, tags::Tag>>,
}

impl Default for PhotoState {
    fn default() -> Self {
        Self {
            db: Mutex::new(sqlite::open(":memory:").ok().unwrap()),
            photos: Mutex::new(Vec::<types::Photo>::new()),
            people: Mutex::new(HashMap::<String, types::Person>::new()),
            cameras: Mutex::new(HashMap::<String, types::Camera>::new()),
            places: Mutex::new(HashMap::<String, types::Place>::new()),
            newest_place: Mutex::new(String::new()),
            tags: Mutex::new(HashMap::<String, tags::Tag>::new()),
        }
    }
}

pub fn row_to_photo(state: &tauri::State<'_, PhotoState>, row: Row) -> types::Photo {
    let mut tags = Vec::<String>::new();
    let tags_row = row.read::<&str, _>("tags").to_string();
    if tags_row.len() > 0 {
        tags = tags_row.split(",").map(str::to_string).collect();
    }
    let mut people = Vec::<String>::new();
    let people_row = row.read::<&str, _>("people").to_string();
    if people_row.len() > 0 {
        people = people_row.split(",").map(str::to_string).collect();
    }

    let validation = tags::validate_tags(&state.tags.lock().unwrap(), &tags);
    types::Photo {
        id: row.read::<&str, _>("Id").to_string(),
        name: row.read::<&str, _>("name").to_string(),
        path: row.read::<&str, _>("path").to_string(),
        title: row.read::<&str, _>("title").to_string(),
        description: row.read::<&str, _>("description").to_string(),
        tags,
        is_duplicate: row.read::<i64, _>("isDuplicate"),
        rating: row.read::<i64, _>("rating"),
        location: row.read::<&str, _>("location").to_string(),
        thumbnail: row.read::<&str, _>("thumbnail").to_string(),
        video: row.read::<i64, _>("video"),
        photo_group: row.read::<&str, _>("photoGroup").to_string(),
        date: row.read::<&str, _>("date").to_string(),
        raw: row.read::<i64, _>("raw"),
        people,
        hide_thumbnail: row.read::<i64, _>("hideThumbnail"),
        photographer: row.read::<&str, _>("photographer").to_string(),
        camera: row.read::<&str, _>("camera").to_string(),
        valid_tags: validation.is_valid,
        validation_msg: validation.message,
    }
}

// Data required for the initial application initialization after the user opens a folder
#[derive(serde::Serialize)]
pub struct OpenFolderResponse {
    deleted: Vec<String>,
    tags: HashMap<String, tags::Tag>,
    person_categories: Vec<types::PersonCategory>,
    groups: Vec<types::Group>,
    layers: Vec<types::Layer>,
    shapes: Vec<types::Shape>,
    activities: Vec<types::Activity>,
    settings: Vec<types::Setting>,
    journals: Vec<types::Journal>,
    wiki_pages: Vec<types::WikiPage>,
    photo_count: usize,
}

/**
 * Sets the working folder path & initializes the SQLite database connection.
 * Returns initial information from the database.
 */
#[tauri::command]
pub async fn open_folder<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
    state: tauri::State<'_, PhotoState>,
    path: String,
) -> Result<OpenFolderResponse, String> {
    // Prepare the database
    let conn = sqlite::open(format!("{path}/photos.db")).ok().unwrap();
    // This additional connection is created here for the create_new_tags function
    *state.db.lock().unwrap() = sqlite::open(format!("{path}/photos.db")).ok().unwrap();

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
    conn.execute("CREATE TABLE IF NOT EXISTS WikiPage (Id TEXT PRIMARY KEY, name TEXT, content TEXT, iv TEXT)").unwrap();

    // Photos stored in the database, which does not necessarily reflect photos actually present in the folder
    let mut existing = HashMap::<String, types::Photo>::new();

    let id_generator = StringGenerator::default();
    let data_dir = app.path().app_data_dir().unwrap().display().to_string();
    let thumbnail_dir = format!("{data_dir}/thumbnails/");
    fs::create_dir_all(&thumbnail_dir).unwrap();

    let mut tags = HashMap::<String, tags::Tag>::new();
    for row in conn
        .prepare("SELECT * FROM Tag")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let name = row.read::<&str, _>("name").to_string();
        let mut prereqs = Vec::<String>::new();
        let prereqs_row = row.read::<&str, _>("prereqs").to_string();
        if prereqs_row.len() > 0 {
            prereqs = prereqs_row.split(",").map(str::to_string).collect();
        }
        let mut coreqs = Vec::<String>::new();
        let coreqs_row = row.read::<&str, _>("coreqs").to_string();
        if coreqs_row.len() > 0 {
            coreqs = coreqs_row.split(",").map(str::to_string).collect();
        }
        let mut incompatible = Vec::<String>::new();
        let incompatible_row = row.read::<&str, _>("incompatible").to_string();
        if incompatible_row.len() > 0 {
            incompatible = incompatible_row.split(",").map(str::to_string).collect();
        }
        tags.insert(
            name.clone(),
            tags::Tag {
                id: row.read::<&str, _>("Id").to_string(),
                name,
                color: row.read::<&str, _>("color").to_string(),
                prereqs,
                coreqs,
                incompatible,
                count: 0,
            },
        );
    }

    for row in conn
        .prepare("SELECT * FROM Photo")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let mut photo_tags = Vec::<String>::new();
        let tags_row = row.read::<&str, _>("tags").to_string();
        if tags_row.len() > 0 {
            photo_tags = tags_row.split(",").map(str::to_string).collect();
            for tag in &photo_tags {
                if !tags.contains_key(tag) {
                    let id = id_generator.next_id();
                    conn.execute(format!(
                        "INSERT INTO Tag VALUES ('{0}', '{1}', '', '', '', '')",
                        database::esc(&id),
                        database::esc(&tag)
                    ))
                    .unwrap();
                    tags.insert(
                        tag.to_string(),
                        tags::Tag {
                            id,
                            name: tag.to_string(),
                            color: String::new(),
                            prereqs: Vec::<String>::new(),
                            coreqs: Vec::<String>::new(),
                            incompatible: Vec::<String>::new(),
                            count: 0,
                        },
                    );
                }
            }
        }
        let mut people = Vec::<String>::new();
        let people_row = row.read::<&str, _>("people").to_string();
        if people_row.len() > 0 {
            people = people_row.split(",").map(str::to_string).collect();
        }

        let validation = tags::validate_tags(&state.tags.lock().unwrap(), &photo_tags);
        let name = row.read::<&str, _>("name").to_string();
        existing.insert(
            name.clone(),
            types::Photo {
                id: row.read::<&str, _>("Id").to_string(),
                name,
                path: row.read::<&str, _>("path").to_string(),
                title: row.read::<&str, _>("title").to_string(),
                description: row.read::<&str, _>("description").to_string(),
                tags: photo_tags,
                is_duplicate: row.read::<i64, _>("isDuplicate"),
                rating: row.read::<i64, _>("rating"),
                location: row.read::<&str, _>("location").to_string(),
                thumbnail: row.read::<&str, _>("thumbnail").to_string(),
                video: row.read::<i64, _>("video"),
                photo_group: row.read::<&str, _>("photoGroup").to_string(),
                date: row.read::<&str, _>("date").to_string(),
                raw: row.read::<i64, _>("raw"),
                people,
                hide_thumbnail: row.read::<i64, _>("hideThumbnail"),
                photographer: row.read::<&str, _>("photographer").to_string(),
                camera: row.read::<&str, _>("camera").to_string(),
                valid_tags: validation.is_valid,
                validation_msg: validation.message,
            },
        );
    }

    let mut places = HashMap::<String, types::Place>::new();
    for row in conn
        .prepare("SELECT * FROM Place")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let id = row.read::<&str, _>("Id").to_string();
        places.insert(
            id.clone(),
            types::Place {
                id,
                name: row.read::<&str, _>("name").to_string(),
                lat: row.read::<f64, _>("lat"),
                lng: row.read::<f64, _>("lng"),
                layer: row.read::<&str, _>("layer").to_string(),
                category: row.read::<&str, _>("category").to_string(),
                shape: row.read::<&str, _>("shape").to_string(),
                tags: row.read::<&str, _>("tags").to_string(),
                notes: row.read::<&str, _>("notes").to_string(),
                count: 0,
            },
        );
    }

    let mut categories = Vec::<types::PersonCategory>::new();
    for row in conn
        .prepare("SELECT * FROM PersonCategory")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        categories.push(types::PersonCategory {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
        });
    }

    let mut people = HashMap::<String, types::Person>::new();
    for row in conn
        .prepare("SELECT * FROM Person")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let id = row.read::<&str, _>("Id").to_string();
        people.insert(
            id.clone(),
            types::Person {
                id,
                name: row.read::<&str, _>("name").to_string(),
                photo: row.read::<&str, _>("photo").to_string(),
                notes: row.read::<&str, _>("notes").to_string(),
                category: row.read::<&str, _>("category").to_string(),
                photographer_count: 0,
                photo_count: 0,
            },
        );
    }

    let mut cameras = HashMap::<String, types::Camera>::new();
    for row in conn
        .prepare("SELECT * FROM Camera")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let id = row.read::<&str, _>("Id").to_string();
        cameras.insert(
            id.clone(),
            types::Camera {
                id,
                name: row.read::<&str, _>("name").to_string(),
                count: 0,
            },
        );
    }

    // The processed list of extant photos in the folder, a combination of existing database entries and new empty objects for new files
    let mut photos = Vec::<types::Photo>::new();

    let re_raw = Regex::new(r"^.*\.(ORF|NRW|HEIC|TIFF|TIF)$").unwrap();
    let re_vid = Regex::new(r"^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM|FLV)$").unwrap();

    // Read the files in the selected folder
    let total_files = fs::read_dir("./").unwrap().count();
    let mut count = 0;
    for file in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if file.metadata().unwrap().is_file() {
            let filename = file.path().display();
            if existing.contains_key(&filename.to_string()) {
                let existing_photo = existing.get(&filename.to_string()).unwrap();
                for person in &existing_photo.people {
                    people.get_mut(person).unwrap().photo_count += 1;
                }
                if existing_photo.photographer.len() > 0 {
                    people
                        .get_mut(&existing_photo.photographer)
                        .unwrap()
                        .photographer_count += 1;
                }
                if existing_photo.camera.len() > 0 {
                    cameras.get_mut(&existing_photo.camera).unwrap().count += 1;
                }
                if existing_photo.location.len() > 0 {
                    places.get_mut(&existing_photo.location).unwrap().count += 1;
                }
                for tag in &existing_photo.tags {
                    if tags.contains_key(tag) {
                        tags.get_mut(tag).unwrap().count += 1;
                    } else {
                        tags.insert(
                            tag.to_string(),
                            tags::Tag {
                                id: id_generator.next_id(),
                                name: tag.to_string(),
                                color: String::new(),
                                prereqs: Vec::<String>::new(),
                                coreqs: Vec::<String>::new(),
                                incompatible: Vec::<String>::new(),
                                count: 1,
                            },
                        );
                    }
                }
                match photos.binary_search_by_key(&existing_photo.name, |p| p.name.clone()) {
                    Ok(_pos) => {}
                    Err(pos) => photos.insert(pos, existing_photo.clone()),
                }
                existing.remove(&filename.to_string());
            } else {
                let tmp = &filename.to_string();
                let thumbnail_path = format!("{thumbnail_dir}/{tmp}.jpg");
                let mut raw = 0i64;
                let mut vid = 0i64;
                // TODO generate thumbnail here
                if re_raw.is_match(&filename.to_string()) {
                    raw = 1i64;
                    // Convert to jpg, then shrink the jpg
                    Command::new("magick")
                        .args([&filename.to_string(), &thumbnail_path])
                        .output()
                        .expect(&format!("Could not generate thumbnail for {tmp}"));
                    Command::new("magick")
                        .args([&thumbnail_path, "-resize", "800x800", &thumbnail_path])
                        .output()
                        .expect(&format!("Could not resize thumbnail for {tmp}"));
                }
                if re_vid.is_match(&filename.to_string()) {
                    vid = 1i64;
                    Command::new("ffmpeg")
                        .args([
                            "-i",
                            &filename.to_string(),
                            "-ss",
                            "00:00:01.00",
                            "-vframes",
                            "1",
                            &thumbnail_path,
                        ])
                        .output()
                        .expect(&format!("Could not generate thumbnail for {tmp}"));
                }
                let asset_path = url_escape::encode_component(tmp);
                let photo = types::Photo {
                    id: id_generator.next_id(),
                    name: filename.to_string(),
                    path: format!("https://asset.localhost/{asset_path}"),
                    title: filename.to_string(),
                    description: String::new(),
                    tags: Vec::<String>::new(),
                    is_duplicate: 0i64,
                    rating: 0i64,
                    location: String::new(),
                    thumbnail: String::new(),
                    video: vid,
                    photo_group: String::new(),
                    date: String::new(),
                    raw,
                    people: Vec::<String>::new(),
                    hide_thumbnail: 0i64,
                    photographer: String::new(),
                    camera: String::new(),
                    valid_tags: true,
                    validation_msg: String::new(),
                };
                conn.execute(
                    format!(
                        "INSERT INTO Photo VALUES ('{0}', '{1}', '{2}', '{3}', '', '', 0, 0, '', '', {4}, '', '', {5}, '', 0, '', '')",
                        database::esc(&photo.id),
                        database::esc(&photo.name),
                        database::esc(&photo.path),
                        database::esc(&photo.title),
                        photo.video,
                        photo.raw
                    )
                ).unwrap();
                match photos.binary_search_by_key(&photo.name, |p| p.name.clone()) {
                    Ok(_pos) => {}
                    Err(pos) => photos.insert(pos, photo),
                }
            }
            count += 1;
            window.emit("load_progress", count / total_files).unwrap();
        }
    }
    *state.photos.lock().unwrap() = photos.clone();
    *state.people.lock().unwrap() = people;
    *state.cameras.lock().unwrap() = cameras;
    *state.places.lock().unwrap() = places;
    *state.tags.lock().unwrap() = tags.clone();

    let mut groups = Vec::<types::Group>::new();
    for row in conn
        .prepare("SELECT * FROM PhotoGroup")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        groups.push(types::Group {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    let mut layers = Vec::<types::Layer>::new();
    for row in conn
        .prepare("SELECT * FROM Layer")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        layers.push(types::Layer {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
        });
    }

    let mut shapes = Vec::<types::Shape>::new();
    for row in conn
        .prepare("SELECT * FROM Shape")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        shapes.push(types::Shape {
            id: row.read::<&str, _>("Id").to_string(),
            shape_type: row.read::<&str, _>("type").to_string(),
            points: row.read::<&str, _>("points").to_string(),
            layer: row.read::<&str, _>("layer").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    let mut activities = Vec::<types::Activity>::new();
    for row in conn
        .prepare("SELECT * FROM Activity")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        activities.push(types::Activity {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            icon: row.read::<&str, _>("icon").to_string(),
        });
    }

    let mut project_version = 0;
    let mut settings = Vec::<types::Setting>::new();
    for row in conn
        .prepare("SELECT * FROM Setting")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let setting = types::Setting {
            id: row.read::<&str, _>("Id").to_string(),
            setting: row.read::<&str, _>("setting").to_string(),
            value: row.read::<i64, _>("value"),
        };
        settings.push(setting.clone());
        if setting.setting == "version" {
            project_version = setting.value;
        }
    }

    if project_version < PHOTO_MANAGER_VERSION {
        println!("Database needs upgrade!");
    }

    let mut journals = Vec::<types::Journal>::new();
    for row in conn
        .prepare("SELECT * FROM Journal")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
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

    let mut wiki_pages = Vec::<types::WikiPage>::new();
    for row in conn
        .prepare("SELECT * FROM WikiPage")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        wiki_pages.push(types::WikiPage {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            content: row.read::<&str, _>("content").to_string(),
            iv: row.read::<&str, _>("iv").to_string(),
        });
    }

    let existing_keys = existing.keys().cloned().map(String::from).collect();
    Ok(OpenFolderResponse {
        deleted: existing_keys,
        tags,
        person_categories: categories,
        groups,
        layers,
        shapes,
        activities,
        settings,
        journals,
        wiki_pages,
        photo_count: photos.len(),
    })
}

/**
 * Performs a search of the photos using the given query.
 */
#[tauri::command]
pub async fn search_photos(
    state: tauri::State<'_, PhotoState>,
    query: Vec<String>,
    sort: String,
) -> Result<(), String> {
    let mut unmet_terms = Vec::<String>::new();

    // Construct a SQL statement using terms that require no additional processing (is:..., at:..., only:..., by:..., has:...)
    let mut statement = "SELECT * FROM Photo WHERE isDuplicate=0".to_string();
    for term in query {
        let mut chars = term.chars();
        let negated = term.get(0..1).unwrap() == "-";
        if negated {
            chars.next();
        }
        let tmp_term = chars.as_str().to_string();
        if tmp_term.get(0..3).unwrap().to_uppercase() == "AT:" {
            let location = database::esc(&tmp_term.get(4..).unwrap().to_string());
            if negated {
                statement.push_str(&format!(" AND location!='{location}'"));
            } else {
                statement.push_str(&format!(" AND location='{location}'"));
            }
        } else if tmp_term.get(0..3).unwrap().to_uppercase() == "IS:" {
            if tmp_term.get(4..).unwrap().to_uppercase() == "VIDEO" {
                if negated {
                    statement.push_str(" AND video=0");
                } else {
                    statement.push_str(" AND video=1");
                }
            } else if tmp_term.get(4..).unwrap().to_uppercase() == "RAW" {
                if negated {
                    statement.push_str(" AND raw=0");
                } else {
                    statement.push_str(" AND raw=1");
                }
            }
        } else if tmp_term.get(0..5).unwrap().to_uppercase() == "ONLY:" {
            let person = database::esc(&tmp_term.get(6..).unwrap().to_string());
            if negated {
                statement.push_str(&format!(" AND people!='{person}'"));
            } else {
                statement.push_str(&format!(" AND people='{person}'"));
            }
        } else if tmp_term.get(0..3).unwrap().to_uppercase() == "BY:" {
            let person = database::esc(&tmp_term.get(4..).unwrap().to_string());
            if negated {
                statement.push_str(&format!(" AND photographer!='{person}'"));
            } else {
                statement.push_str(&format!(" AND photographer='{person}'"));
            }
        } else if tmp_term.get(0..4).unwrap().to_uppercase() == "HAS:" {
            if tmp_term.get(5..).unwrap().to_uppercase() == "RATING" {
                if negated {
                    statement.push_str(" AND rating=0");
                } else {
                    statement.push_str(" AND rating>0");
                }
            } else if tmp_term.get(5..).unwrap().to_uppercase() == "PHOTOGRAPHER" {
                if negated {
                    statement.push_str(" AND length(photographer)=0");
                } else {
                    statement.push_str(" AND length(photographer)>0");
                }
            } else if tmp_term.get(5..).unwrap().to_uppercase() == "DATE" {
                if negated {
                    statement.push_str(" AND length(date)=0");
                } else {
                    statement.push_str(" AND length(date)>0");
                }
            } else if tmp_term.get(5..).unwrap().to_uppercase() == "LOCATION" {
                if negated {
                    statement.push_str(" AND length(location)=0");
                } else {
                    statement.push_str(" AND length(location)>0");
                }
            }
        } else if tmp_term.get(0..5).unwrap().to_uppercase() == "NAME:" {
            let name = database::esc(&tmp_term.get(5..).unwrap().to_string());
            if negated {
                statement.push_str(&format!(" AND Name NOT LIKE '%{name}%'"));
            } else {
                statement.push_str(&format!(" AND Name LIKE '%{name}%'"));
            }
        } else {
            unmet_terms.push(term);
        }
    }

    let mut results = Vec::<types::Photo>::new();

    println!("Executing {}", statement);
    let connection = state.db.lock().unwrap();
    let photos = connection
        .prepare(statement)
        .unwrap()
        .into_iter()
        .map(|row| row_to_photo(&state, row.unwrap()));

    // I *want* to use SQL ORDER BY to sort the results, but it seems the results lose their order somewhere in the above statement

    // Terms that require additional processing and iterating over the photos (date:..., of:..., any tags)
    let mut first = true;
    if unmet_terms.len() > 0 {
        for photo in photos {
            if first {
                println!("First row: {}", photo.name);
                first = false;
            }
            for term in &unmet_terms {
                let mut chars = term.chars();
                let negated = term.get(0..1).unwrap() == "-";
                if negated {
                    chars.next();
                }
                let tmp_term = chars.as_str().to_string();
                if tmp_term.get(0..3).unwrap().to_uppercase() == "OF:" {
                    let in_photo = photo
                        .people
                        .contains(&tmp_term.get(4..).unwrap().to_string());
                    if (negated && !in_photo) || (!negated && in_photo) {
                        if sort == "name" || sort == "name_desc" {
                            match results.binary_search_by_key(&photo.name, |p| p.name.clone()) {
                                Ok(_pos) => {}
                                Err(pos) => results.insert(pos, photo.clone()),
                            }
                        } else if sort == "rating" || sort == "rating_desc" {
                            match results.binary_search_by_key(&photo.rating, |p| p.rating) {
                                Ok(_pos) => {}
                                Err(pos) => results.insert(pos, photo.clone()),
                            }
                        }
                    }
                } else if tmp_term.get(0..5).unwrap().to_uppercase() == "DATE:" {
                    // TODO
                } else {
                    // Treat all remaining terms as tags
                    let in_tags = photo.tags.contains(&tmp_term);
                    if (negated && !in_tags) || (!negated && in_tags) {
                        if sort == "name" || sort == "name_desc" {
                            match results.binary_search_by_key(&photo.name, |p| p.name.clone()) {
                                Ok(_pos) => {}
                                Err(pos) => results.insert(pos, photo.clone()),
                            }
                        } else if sort == "rating" || sort == "rating_desc" {
                            match results.binary_search_by_key(&photo.rating, |p| p.rating) {
                                Ok(_pos) => {}
                                Err(pos) => results.insert(pos, photo.clone()),
                            }
                        }
                    }
                }
            }
        }
    } else {
        results = photos.collect::<Vec<types::Photo>>();
        if sort == "name" || sort == "name_desc" {
            results.sort_by_key(|p| p.name.clone());
        } else if sort == "rating" || sort == "rating_desc" {
            results.sort_by_key(|p| p.rating);
        }
    }

    if sort == "name_desc" || sort == "rating_desc" {
        results.reverse();
    }

    println!("Returning {} photos", results.len());
    println!("First result: {}", results.first().unwrap().name);

    let mut state_photos = state.photos.lock().unwrap();

    *state_photos = results;

    Ok(())
}

#[derive(serde::Serialize)]
pub struct PhotoGridResponse {
    photos: Vec<types::Photo>,
    total: usize,
}

#[tauri::command]
pub async fn photo_grid(
    state: tauri::State<'_, PhotoState>,
    start: i64,
    count: i64,
) -> Result<PhotoGridResponse, String> {
    let state_photos = state.photos.lock().unwrap();
    let mut index = start as usize;
    let count_u = count as usize;

    // TODO: Group raws & groups
    /*
    for (const raw of raws) {
      const baseName = raw.name.replace('.ORF', '').replace('.NRW', '');
      const jpg = this.files[`${baseName}.JPG`];
      const png = this.files[`${baseName}.PNG`];
      const base = this.files[raw.name];
      if (jpg && base) {
        jpg.rawFile = raw.thumbnail;
        base.hidden = true;
      } else if (png && base) {
        png.rawFile = raw.thumbnail;
        base.hidden = true;
      }
    } */
    if index > state_photos.len() {
        index = 0;
        println!(
            "Warning: Photo grid start index is greater than the number of photos; {0}",
            start
        );
    }
    let mut slice_end = index + count_u;
    if slice_end > state_photos.len() {
        println!(
            "Warning: attempting to search for more photos than exist: {0}",
            slice_end
        );
        slice_end = state_photos.len() - 1;
    }
    Ok(PhotoGridResponse {
        photos: state_photos[index..slice_end].to_vec(),
        total: state_photos.len(),
    })
}

// TODO: Removing deleted causes the subsequent photo_grid request to fail
#[tauri::command]
pub async fn remove_deleted(
    state: tauri::State<'_, PhotoState>,
    deleted: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();

    for name in deleted {
        connection
            .execute(format!(
                "DELETE FROM Photo WHERE Name='{0}'",
                database::esc(&name)
            ))
            .unwrap();
        match state_photos.binary_search_by_key(&name, |p| p.name.clone()) {
            Ok(pos) => {
                state_photos.remove(pos);
            }
            Err(_pos) => {}
        }
    }

    Ok(())
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
            "UPDATE Photo SET {property}='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&photo)
        ))
        .unwrap();
    Ok(())
}

fn update_photographer_count(
    state: &tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_photographer_count(&state, &existing.photographer, &value);

    connection
        .execute(format!(
            "UPDATE Photo SET photographer='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = database::esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE photoGroup='{group}' AND Id!='{0}'",
                database::esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_photographer_count(&state, &photo.photographer, &value);
            let id = database::esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET photographer='{0}' WHERE Id='{id}'",
                    database::esc(&value)
                ))
                .unwrap();
        }
    }
    Ok(())
}

fn update_people_counts(
    state: &tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_people_counts(&state, &existing.people, &value);

    let joined_people = database::esc(&value.join(","));
    connection
        .execute(format!(
            "UPDATE Photo SET people='{joined_people}' WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = database::esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE photoGroup='{group}' AND Id!='{0}'",
                database::esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_people_counts(&state, &photo.people, &value);
            let id = database::esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET people='{joined_people}' WHERE Id='{id}'"
                ))
                .unwrap();
        }
    }

    Ok(())
}

fn update_camera_count(
    state: &tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_camera_count(&state, &existing.camera, &value);

    connection
        .execute(format!(
            "UPDATE Photo SET camera='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = database::esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE photoGroup='{group}' AND Id!='{0}'",
                database::esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_camera_count(&state, &photo.camera, &value);
            let id = database::esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET camera='{0}' WHERE Id='{id}'",
                    database::esc(&value)
                ))
                .unwrap();
        }
    }

    Ok(())
}

fn update_location_count(
    state: &tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    update_location_count(&state, &existing.location, &value);

    connection
        .execute(format!(
            "UPDATE Photo SET location='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = database::esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE photoGroup='{group}' AND Id!='{0}'",
                database::esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_location_count(&state, &photo.location, &value);
            let id = database::esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET location='{0}' WHERE Id='{id}'",
                    database::esc(&value),
                ))
                .unwrap();
        }
    }

    Ok(())
}

fn update_tag_counts(
    state: &tauri::State<'_, PhotoState>,
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
    state: tauri::State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    tags::create_new_tags(&state, &value);
    update_tag_counts(&state, &existing.tags, &value);

    let validation = tags::validate_tags(&state.tags.lock().unwrap(), &value);

    let mut state_photos = state.photos.lock().unwrap();
    match state_photos.binary_search_by_key(&photo, |p| p.name.clone()) {
        Ok(pos) => {
            state_photos[pos].tags = value.clone();
            state_photos[pos].valid_tags = validation.is_valid;
            state_photos[pos].validation_msg = validation.message;
        }
        Err(_pos) => {}
    }

    let tags_str = database::esc(&value.join(","));
    connection
        .execute(format!(
            "UPDATE Photo SET tags='{tags_str}' WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = database::esc(&existing.photo_group);
        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE photoGroup='{group}' AND Id!='{0}'",
                database::esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()));
        for photo in in_group {
            update_tag_counts(&state, &photo.tags, &value);
            let id = database::esc(&photo.id);
            connection
                .execute(format!(
                    "UPDATE Photo SET tags='{tags_str}' WHERE Id='{id}'"
                ))
                .unwrap();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_date(
    state: tauri::State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    let results = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row_to_photo(&state, row.unwrap()));
    let existing = results.last().unwrap();

    connection
        .execute(format!(
            "UPDATE Photo SET date='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&photo)
        ))
        .unwrap();

    if existing.photo_group.len() > 0 {
        let group = database::esc(&existing.photo_group);
        connection
            .execute(format!(
                "UPDATE Photo SET date='{0}' WHERE photoGroup='{group}' AND Id!='{1}'",
                database::esc(&value),
                database::esc(&photo)
            ))
            .unwrap();
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_group(
    state: tauri::State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();

    if value.len() == 0 {
        connection
            .execute(format!(
                "UPDATE Photo SET photoGroup='' WHERE Id='{0}'",
                database::esc(&photo)
            ))
            .unwrap();
    } else {
        let results = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE Id='{0}'",
                database::esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()));
        let existing = results.last().unwrap();

        let in_group = connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE photoGroup='{0}'",
                database::esc(&value)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()));

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

        tags::create_new_tags(&state, &group_tags);
        update_tag_counts(&state, &existing.tags, &group_tags);
        update_people_counts(&state, &existing.people, &group_people);
        update_location_count(&state, &existing.location, &collected_location);
        update_photographer_count(&state, &existing.photographer, &collected_photographer);
        update_camera_count(&state, &existing.camera, &collected_camera);

        let validation = tags::validate_tags(&state.tags.lock().unwrap(), &group_tags);
        let mut state_photos = state.photos.lock().unwrap();
        // TODO - this needs to update all the other properties for every photo in the group too
        match state_photos.binary_search_by_key(&photo, |p| p.name.clone()) {
            Ok(pos) => {
                state_photos[pos].valid_tags = validation.is_valid;
                state_photos[pos].validation_msg = validation.message;
            }
            Err(_pos) => {}
        }

        let group_tags_str = database::esc(&group_tags.join(","));
        let group_people_str = database::esc(&group_people.join(","));
        collected_location = database::esc(&collected_location);
        collected_photographer = database::esc(&collected_photographer);
        collected_camera = database::esc(&collected_camera);
        collected_date = database::esc(&collected_date);
        connection.execute(
            format!(
                "UPDATE Photo SET photoGroup='{0}', tags='{group_tags_str}', people='{group_people_str}', location='{collected_location}', photographer='{collected_photographer}', camera='{collected_camera}', date='{collected_date}' WHERE Id='{1}'",
                database::esc(&value),
                database::esc(&photo)
            )
        ).unwrap();

        for photo in connection
            .prepare(format!(
                "SELECT * FROM Photo WHERE photoGroup='{0}' AND Id!='{1}'",
                database::esc(&value),
                database::esc(&photo)
            ))
            .unwrap()
            .into_iter()
            .map(|row| row_to_photo(&state, row.unwrap()))
        {
            update_tag_counts(&state, &photo.tags, &group_tags);
            update_people_counts(&state, &photo.people, &group_people);
            update_location_count(&state, &photo.location, &collected_location);
            update_photographer_count(&state, &photo.photographer, &collected_photographer);
            update_camera_count(&state, &photo.camera, &collected_camera);

            let id = database::esc(&photo.id);
            connection.execute(format!("UPDATE Photo SET tags='{group_tags_str}', people='{group_people_str}', location='{collected_location}', photographer='{collected_photographer}', camera='{collected_camera}', date='{collected_date}' WHERE Id='{id}'")).unwrap();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_rating(
    state: tauri::State<'_, PhotoState>,
    photo: String,
    rating: i64,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Photo SET rating={rating} WHERE Id='{0}'",
            database::esc(&photo)
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
            "UPDATE Photo SET {property}={value} WHERE Id='{0}'",
            database::esc(&photo)
        ))
        .unwrap();
    Ok(())
}
