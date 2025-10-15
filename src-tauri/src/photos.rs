use crate::types;
use regex::Regex;
use sqlite::Connection;
use sqlite::Row;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use tauri::Manager;
use unique_id::string::StringGenerator;
use unique_id::Generator;
use walkdir::WalkDir;

pub struct PhotoState {
    pub db: Mutex<Connection>,
}

impl Default for PhotoState {
    fn default() -> Self {
        Self {
            db: Mutex::new(sqlite::open(":memory:").ok().unwrap()),
        }
    }
}

// Data required for the initial application initialization after the user opens a folder
#[derive(serde::Serialize)]
pub struct OpenFolderResponse {
    photos: Vec<types::Photo>,
    deleted: Vec<String>,
    tags: Vec<types::Tag>,
    places: Vec<types::Place>,
    person_categories: Vec<types::PersonCategory>,
    people: Vec<types::Person>,
    cameras: Vec<types::Camera>,
    groups: Vec<types::Group>,
    layers: Vec<types::Layer>,
    shapes: Vec<types::Shape>,
    activities: Vec<types::Activity>,
    settings: Vec<types::Setting>,
    journals: Vec<types::Journal>,
    wiki_pages: Vec<types::WikiPage>,
}

fn rows_to_photos<T: Iterator<Item = Row>>(rows: T) -> Result<Vec<types::Photo>, String> {
    let mut photos = Vec::<types::Photo>::new();

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

/**
 * Sets the working folder path & initializes the SQLite database connection.
 * Returns initial information from the database.
 */
#[tauri::command]
pub async fn open_folder<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, PhotoState>,
    path: String,
) -> Result<OpenFolderResponse, String> {
    // Prepare the database
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
    conn.execute("CREATE TABLE IF NOT EXISTS WikiPage (Id TEXT PRIMARY KEY, name TEXT, content TEXT, iv TEXT)").unwrap();

    // Photos stored in the database, which does not necessarily reflect photos actually present in the folder
    let mut existing = HashMap::<String, types::Photo>::new();
    // The names of photos in the database, which will be used to detect deleted photos
    let mut unmatched = Vec::<String>::new();

    for row in conn
        .prepare("SELECT * FROM Photo")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        unmatched.push(row.read::<&str, _>("name").to_string());
        existing.insert(
            row.read::<&str, _>("name").to_string(),
            types::Photo {
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
            },
        );
    }

    let id_generator = StringGenerator::default();
    let data_dir = app.path().app_data_dir().unwrap().display().to_string();
    let thumbnail_dir = format!("{data_dir}/thumbnails/");
    fs::create_dir_all(&thumbnail_dir).unwrap();

    let mut tags = Vec::<types::Tag>::new();
    for row in conn
        .prepare("SELECT * FROM Tag")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        tags.push(types::Tag {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
            prereqs: row.read::<&str, _>("prereqs").to_string(),
            coreqs: row.read::<&str, _>("coreqs").to_string(),
            incompatible: row.read::<&str, _>("incompatible").to_string(),
        });
    }

    let mut places = Vec::<types::Place>::new();
    for row in conn
        .prepare("SELECT * FROM Place")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
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

    let mut people = Vec::<types::Person>::new();
    for row in conn
        .prepare("SELECT * FROM Person")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        people.push(types::Person {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            photo: row.read::<&str, _>("photo").to_string(),
            notes: row.read::<&str, _>("notes").to_string(),
            category: row.read::<&str, _>("category").to_string(),
        });
    }

    let mut cameras = Vec::<types::Camera>::new();
    for row in conn
        .prepare("SELECT * FROM Camera")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        cameras.push(types::Camera {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    // The processed list of extant photos in the folder, a combination of existing database entries and new empty objects for new files
    let mut photos = Vec::<types::Photo>::new();

    let re_raw = Regex::new(r"^.*\.(ORF|NRW|HEIC|TIFF|TIF)$").unwrap();
    let re_vid = Regex::new(r"^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM|FLV)$").unwrap();

    // Read the files in the selected folder
    for file in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if file.metadata().unwrap().is_file() {
            let filename = file.path().display();
            if existing.contains_key(&filename.to_string()) {
                let idx = unmatched.binary_search(&filename.to_string());
                if idx.is_ok() {
                    unmatched.remove(idx.unwrap());
                }
                photos.push(existing.get(&filename.to_string()).unwrap().clone());
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
                    tags: String::new(),
                    is_duplicate: 0i64,
                    rating: 0i64,
                    location: String::new(),
                    thumbnail: String::new(),
                    video: vid,
                    photo_group: String::new(),
                    date: String::new(),
                    raw,
                    people: String::new(),
                    hide_thumbnail: 0i64,
                    photographer: String::new(),
                    camera: String::new(),
                };
                conn.execute(format!("INSERT INTO Photo VALUES ('{0}', '{1}', '{2}', '{3}', '', '', 0, 0, '', '', {4}, '', '', {5}, '', 0, '', '')", photo.id, photo.name, photo.path, photo.title, photo.video, photo.raw)).unwrap();
                photos.push(photo);
            }
        }
    }

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

    let mut settings = Vec::<types::Setting>::new();
    for row in conn
        .prepare("SELECT * FROM Setting")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        settings.push(types::Setting {
            id: row.read::<&str, _>("Id").to_string(),
            setting: row.read::<&str, _>("setting").to_string(),
            value: row.read::<i64, _>("value"),
        });
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

    *state.db.lock().unwrap() = conn;

    Ok(OpenFolderResponse {
        photos,
        deleted: unmatched,
        tags,
        places,
        person_categories: categories,
        people,
        cameras,
        groups,
        layers,
        shapes,
        activities,
        settings,
        journals,
        wiki_pages,
    })
}

/**
 * Performs a search of the photos using the given query.
 * Returns a list of photos matching the query.
 * This is not currently used, I went through all the trouble of writing this just to find out that
 * re-loading the photos list in the UI is infinitely slower than doing the search calculations on the frontend.
 */
#[tauri::command]
pub async fn search_photos(
    state: tauri::State<'_, PhotoState>,
    query: Vec<String>,
) -> Result<Vec<types::Photo>, String> {
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
            let location = tmp_term.get(4..).unwrap();
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
            let person = tmp_term.get(6..).unwrap();
            if negated {
                statement.push_str(&format!(" AND people!='{person}'"));
            } else {
                statement.push_str(&format!(" AND people='{person}'"));
            }
        } else if tmp_term.get(0..3).unwrap().to_uppercase() == "BY:" {
            let person = tmp_term.get(4..).unwrap();
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
        } else {
            unmet_terms.push(term);
        }
    }

    let mut results = Vec::<types::Photo>::new();

    println!("Executing {}", statement);
    let connection = state.db.lock().unwrap();
    let photos = rows_to_photos(
        connection
            .prepare(statement)
            .unwrap()
            .into_iter()
            .map(|row| row.unwrap()),
    )
    .unwrap();

    // Terms that require additional processing and iterating over the photos (date:..., of:..., any tags)
    if unmet_terms.len() > 0 {
        for photo in &photos {
            for term in &unmet_terms {
                let mut chars = term.chars();
                let negated = term.get(0..1).unwrap() == "-";
                if negated {
                    chars.next();
                }
                let tmp_term = chars.as_str().to_string();
                if tmp_term.get(0..3).unwrap().to_uppercase() == "OF:" {
                    let person = tmp_term.get(4..).unwrap();
                    let in_photo = photo.people.split(',').any(|p| p == person);
                    if (negated && !in_photo) || (!negated && in_photo) {
                        results.push(photo.clone());
                    }
                } else if tmp_term.get(0..5).unwrap().to_uppercase() == "DATE:" {
                    // TODO
                } else {
                    // Treat all remaining terms as tags
                    let in_tags = photo.tags.split(',').any(|t| t == tmp_term);
                    if (negated && !in_tags) || (!negated && in_tags) {
                        results.push(photo.clone());
                    }
                }
            }
        }
    } else {
        results = photos;
    }
    println!("Returning {} photos", results.len());

    Ok(results)
}
