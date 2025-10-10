use crate::types;
use regex::Regex;
use sqlite::Connection;
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
    deleted: Vec<String>
}

/**
 * Sets the working folder path & initializes the SQLite database connection.
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

    // The processed list of extant photos in the folder, a combination of existing database entries and new empty objects for new files
    let mut photos = Vec::<types::Photo>::new();

    let re_raw = Regex::new(r"^.*\.(ORF|NRW|HEIC|TIFF|TIF)$").unwrap();
    let re_vid = Regex::new(r"^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM|FLV)$").unwrap();

    // Read the files in the selected folder
    for file in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if file.metadata().unwrap().is_file() {
            let filename = file.path().display();
            if existing.contains_key(&filename.to_string()) {
                unmatched.remove(unmatched.binary_search(&filename.to_string()).unwrap());
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
                println!("{}", format!("https://asset.localhost/{asset_path}"));
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

    *state.db.lock().unwrap() = conn;

    Ok(OpenFolderResponse {
        photos,
        deleted: unmatched
    })
}
