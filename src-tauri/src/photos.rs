use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    path::PathBuf,
    process::Command,
    sync::Mutex,
    time::Duration,
};

use chrono::NaiveDate;
use regex::Regex;
use rusty_pool::{JoinHandle, ThreadPool};
use serde::Serialize;
use sqlite::{Connection, Row};
use tauri::{AppHandle, Manager, Runtime, State};
use thiserror::Error;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    esc,
    group::Group,
    people::{row_to_person, Person},
    places::{row_to_layer, row_to_place, Layer, Place},
    row_to_vec,
    settings::Setting,
    tags::{row_to_tag, validate_tags, Tag, ValidationResult},
    ApiError,
};

const PHOTO_MANAGER_VERSION: i64 = 1;
const DATE_FORMAT: &str = "%F";

#[derive(Serialize, Clone)]
pub struct Photo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub is_duplicate: i64,
    pub rating: i64,
    pub location: String,
    pub thumbnail: String,
    pub video: i64,
    pub photo_group: String,
    pub date: Option<NaiveDate>,
    pub raw: i64,
    pub people: Vec<String>,
    pub hide_thumbnail: i64,
    pub photographer: String,
    pub valid_tags: bool,
    pub validation_msg: String,
}

impl Photo {
    fn new(id: String, filename: String) -> Self {
        Self {
            id,
            name: filename.clone(),
            path: format!(
                "https://asset.localhost/{0}",
                url_escape::encode_component(&filename)
            ),
            title: String::new(),
            description: String::new(),
            tags: Vec::<String>::new(),
            is_duplicate: 0,
            rating: 0,
            location: String::new(),
            thumbnail: String::new(),
            video: 0,
            photo_group: String::new(),
            date: None,
            raw: 0,
            people: Vec::<String>::new(),
            hide_thumbnail: 0,
            photographer: String::new(),
            valid_tags: true,
            validation_msg: String::new(),
        }
    }
}

pub struct PhotoState {
    pub db: Mutex<Connection>,
    pub photos: Mutex<HashMap<String, Photo>>,
    pub people: Mutex<HashMap<String, Person>>,
    pub places: Mutex<HashMap<String, Place>>,
    pub layers: Mutex<HashMap<String, Layer>>,
    pub tags: Mutex<HashMap<String, Tag>>,
    pub groups: Mutex<HashMap<String, Group>>,
    pub settings: Mutex<HashMap<String, Setting>>,
}

impl Default for PhotoState {
    fn default() -> Self {
        Self {
            db: Mutex::new(sqlite::open(":memory:").ok().unwrap()),
            photos: Mutex::new(HashMap::<String, Photo>::new()),
            people: Mutex::new(HashMap::<String, Person>::new()),
            places: Mutex::new(HashMap::<String, Place>::new()),
            layers: Mutex::new(HashMap::<String, Layer>::new()),
            tags: Mutex::new(HashMap::<String, Tag>::new()),
            groups: Mutex::new(HashMap::<String, Group>::new()),
            settings: Mutex::new(HashMap::<String, Setting>::new()),
        }
    }
}

fn parse_date(date: &String) -> NaiveDate {
    NaiveDate::parse_from_str(date, "")
}

fn clean_thumbnail_path(path: &String) -> String {
    path.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_string()
            } else {
                "_".to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

pub fn row_to_photo(row: &Row) -> Photo {
    let mut date: Option<NaiveDate> = None;
    let parsed_date = NaiveDate::parse_from_str(&row.read::<&str, _>("date").to_string(), &DATE_FORMAT);
    if parsed_date.is_ok() {
        date = Some(parsed_date.unwrap());
    }
    Photo {
        id: row.read::<&str, _>("Id").to_string(),
        name: row.read::<&str, _>("name").to_string(),
        path: row.read::<&str, _>("path").to_string(),
        title: row.read::<&str, _>("title").to_string(),
        description: row.read::<&str, _>("description").to_string(),
        tags: row_to_vec(row, "tags"),
        is_duplicate: row.read::<i64, _>("isDuplicate"),
        rating: row.read::<i64, _>("rating"),
        location: row.read::<&str, _>("location").to_string(),
        thumbnail: row.read::<&str, _>("thumbnail").to_string(),
        video: row.read::<i64, _>("video"),
        photo_group: row.read::<&str, _>("photoGroup").to_string(),
        date,
        raw: row.read::<i64, _>("raw"),
        people: row_to_vec(row, "people"),
        hide_thumbnail: row.read::<i64, _>("hideThumbnail"),
        photographer: row.read::<&str, _>("photographer").to_string(),
        valid_tags: true,
        validation_msg: String::new(),
    }
}

#[derive(Debug, Error)]
pub enum LoadPhotoError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlite::Error),
    #[error("Application error: {0}")]
    AppError(#[from] tauri::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Error reading directory: {0}")]
    WalkdirError(#[from] walkdir::Error),
}

impl Serialize for LoadPhotoError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct LoadedPhotos {
    photos: HashMap<String, Photo>,
    removed: Vec<String>,
}

fn load_photos(
    conn: &Connection,
    tags: &mut HashMap<String, Tag>,
    people: &mut HashMap<String, Person>,
    places: &mut HashMap<String, Place>,
    path: &String,
    thumbnail_dir: &PathBuf,
    counts: bool,
) -> Result<LoadedPhotos, LoadPhotoError> {
    // Photos stored in the database, which does not necessarily reflect photos actually present in the folder
    let mut existing = HashMap::<String, Photo>::new();

    for row in conn.prepare("SELECT * FROM Photo")? {
        let row = row?;
        let mut photo = row_to_photo(&row);
        for tag in &photo.tags {
            if counts && tags.contains_key(tag) {
                tags.get_mut(tag).unwrap().count += 1;
            }
            if !tags.contains_key(tag) {
                let mut t = Tag::new(tag);
                t.count = 1;
                tags.insert(tag.clone(), t);
            }
        }
        let validation = validate_tags(&tags, &photo.tags);
        photo.valid_tags = validation.is_valid;
        photo.validation_msg = validation.message;
        existing.insert(photo.name.clone(), photo);
    }

    // The processed list of extant photos in the folder, a combination of existing database entries and new empty objects for new files
    let mut photos = HashMap::<String, Photo>::new();

    let re_raw = Regex::new(r"^.*\.(ORF|NRW|HEIC|TIFF|TIF)$")?;
    let re_vid = Regex::new(r"^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM|FLV)$")?;

    // Read the files in the selected folder
    let mut file_queue = VecDeque::<PathBuf>::new();
    for entry in fs::read_dir(&path)? {
        file_queue.push_back(entry?.path());
    }
    let pool = ThreadPool::new(4, 4, Duration::from_millis(50));
    let mut threads = Vec::<JoinHandle<Photo>>::new();
    for file in WalkDir::new(path) {
        let file = file?;
        if file.metadata().unwrap().is_file() {
            let filename = file.path().display().to_string();
            if existing.contains_key(&filename) {
                let existing_photo = existing.get(&filename.to_string()).unwrap();
                if counts {
                    for person in &existing_photo.people {
                        if people.contains_key(person) {
                            people.get_mut(person).unwrap().photo_count += 1;
                        }
                    }
                    if existing_photo.photographer.len() > 0 {
                        if people.contains_key(&existing_photo.photographer) {
                            people
                                .get_mut(&existing_photo.photographer)
                                .unwrap()
                                .photographer_count += 1;
                        }
                    }
                    if places.contains_key(&existing_photo.location) {
                        places.get_mut(&existing_photo.location).unwrap().count += 1;
                    }
                    for tag in &existing_photo.tags {
                        if tags.contains_key(tag) {
                            tags.get_mut(tag).unwrap().count += 1;
                        }
                    }
                }
                photos.insert(existing_photo.id.clone(), existing_photo.clone());
                existing.remove(&filename.to_string());
            } else {
                let thumbnail_path = thumbnail_dir.join(clean_thumbnail_path(&filename));
                let id = Uuid::new_v4().to_string();
                if re_raw.is_match(&filename.to_uppercase()) {
                    if !thumbnail_path.exists() {
                        // Convert to jpg
                        threads.push(pool.complete(async move {
                            let output = Command::new("magick")
                                .args([&filename, thumbnail_path.to_str().unwrap()])
                                .output();
                            if output.is_err() {
                                println!(
                                    "ERROR: Could not generate thumbnail for {0}: {1}",
                                    &filename.to_string(),
                                    &output.err().unwrap().to_string()
                                );
                            }
                            let mut photo = Photo::new(id, filename);
                            photo.thumbnail = thumbnail_path.to_str().unwrap().to_string();
                            photo.raw = 1;
                            photo
                        }));
                    }
                } else if re_vid.is_match(&filename.to_uppercase()) {
                    if !thumbnail_path.exists() {
                        threads.push(pool.complete(async move {
                            let output = Command::new("ffmpeg")
                                .args([
                                    "-i",
                                    &filename.to_string(),
                                    "-ss",
                                    "00:00:01.00",
                                    "-vframes",
                                    "1",
                                    &thumbnail_path.to_str().unwrap(),
                                ])
                                .output();
                            if output.is_err() {
                                println!(
                                    "ERROR: Could not generate thumbnail for {0}: {1}",
                                    &filename.to_string(),
                                    &output.err().unwrap().to_string()
                                );
                            }
                            let mut photo = Photo::new(id, filename);
                            photo.thumbnail = thumbnail_path.to_str().unwrap().to_string();
                            photo.video = 1;
                            photo
                        }));
                    }
                } else {
                    let photo = Photo::new(id, filename);
                    conn.execute(
                        format!(
                            "INSERT INTO Photo VALUES ('{0}', '{1}', '{2}', '', '', '', 0, 0, '', '{3}', 0, '', '', 0, '', 0, '', '')",
                            esc(&photo.id),
                            esc(&photo.name),
                            esc(&photo.path),
                            esc(&photo.thumbnail),
                        )
                    )?;
                    photos.insert(photo.id.clone(), photo);
                }
            }
        }
    }

    for thread in threads {
        let result = thread.await_complete();
        conn.execute(
            format!(
                "INSERT INTO Photo VALUES ('{0}', '{1}', '{2}', '', '', '', 0, 0, '', '{3}', {4}, '', '', {5}, '', 0, '', '')",
                esc(&result.id),
                esc(&result.name),
                esc(&result.path),
                esc(&result.thumbnail),
                result.video,
                result.raw,
            )
        )?;
        photos.insert(result.id.clone(), result);
    }

    Ok(LoadedPhotos {
        photos,
        removed: existing.keys().cloned().map(String::from).collect(),
    })
}

/// Sets the working folder path & initializes the SQLite database connection.
/// Returns initial information from the database.
#[tauri::command]
pub fn initialize<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PhotoState>,
    path: String,
) -> Result<Vec<String>, ApiError> {
    // Prepare the database
    let conn = sqlite::open(format!("{path}/photos.db"))?;

    for statement in [
        "CREATE TABLE IF NOT EXISTS Activity (Id TEXT PRIMARY KEY, name TEXT, icon TEXT)",
        "CREATE TABLE IF NOT EXISTS Camera (Id TEXT PRIMARY KEY, name TEXT)",
        "CREATE TABLE IF NOT EXISTS Journal (Id TEXT PRIMARY KEY, date TEXT, mood INTEGER, text TEXT, activities TEXT, steps INTEGER, iv TEXT)",
        "CREATE TABLE IF NOT EXISTS Layer (Id TEXT PRIMARY KEY, name TEXT, color TEXT)",
        "CREATE TABLE IF NOT EXISTS Person (Id TEXT PRIMARY KEY, name TEXT, photo TEXT, notes TEXT, category TEXT)",
        "CREATE TABLE IF NOT EXISTS PersonCategory (Id TEXT PRIMARY KEY, name TEXT, color TEXT)",
        "CREATE TABLE IF NOT EXISTS Photo (Id TEXT , name TEXT PRIMARY KEY, path TEXT, title TEXT, description TEXT, tags TEXT, isDuplicate INTEGER, rating INTEGER, location TEXT, thumbnail TEXT, video INTEGER, photoGroup TEXT, date TEXT, raw INTEGER, people TEXT, hideThumbnail INTEGER, photographer TEXT, camera TEXT)",
        "CREATE TABLE IF NOT EXISTS PhotoGroup (Id TEXT PRIMARY KEY, name TEXT)",
        "CREATE TABLE IF NOT EXISTS Place (Id TEXT PRIMARY KEY, name TEXT, lat INTEGER, lng INTEGER, layer TEXT, category TEXT, shape TEXT, tags TEXT, notes TEXT)",
        "CREATE TABLE IF NOT EXISTS Setting (Id TEXT , setting TEXT PRIMARY KEY, value INTEGER)",
        "CREATE TABLE IF NOT EXISTS Shape (Id TEXT PRIMARY KEY, type TEXT, points TEXT, layer TEXT, name TEXT)",
        "CREATE TABLE IF NOT EXISTS Tag (Id TEXT , name TEXT PRIMARY KEY, color TEXT, prereqs TEXT, coreqs TEXT, incompatible TEXT)",
        "CREATE TABLE IF NOT EXISTS WikiPage (Id TEXT PRIMARY KEY, name TEXT, content TEXT, iv TEXT)",
    ] {
        conn.execute(statement)?;
    }

    let thumbnail_dir = app.path().app_data_dir()?.join("thumbnails");
    if !thumbnail_dir.exists() {
        fs::create_dir_all(&thumbnail_dir)?;
    }

    let mut tags = HashMap::<String, Tag>::new();
    for row in conn.prepare("SELECT * FROM Tag")? {
        let row = row?;
        let tag = row_to_tag(&row);
        tags.insert(tag.name.clone(), tag);
    }

    let mut layers = HashMap::<String, Layer>::new();
    for row in conn.prepare("SELECT * FROM Layer")? {
        let row = row?;
        let layer = row_to_layer(&row);
        layers.insert(layer.id.clone(), layer);
    }

    let mut places = HashMap::<String, Place>::new();
    for row in conn.prepare("SELECT * FROM Place")? {
        let row = row_to_place(&row?);
        if layers.contains_key(&row.layer) {
            layers.get_mut(&row.layer).unwrap().count += 1;
        }
        places.insert(row.id.clone(), row);
    }

    let mut people = HashMap::<String, Person>::new();
    for row in conn.prepare("SELECT * FROM Person")? {
        let person = row_to_person(&row?);
        people.insert(person.id.clone(), person);
    }

    let photo_load = load_photos(
        &conn,
        &mut tags,
        &mut people,
        &mut places,
        &path,
        &thumbnail_dir,
        true,
    )?;

    *state.photos.lock().unwrap() = photo_load.photos;
    *state.people.lock().unwrap() = people;
    *state.places.lock().unwrap() = places;
    *state.layers.lock().unwrap() = layers;
    *state.tags.lock().unwrap() = tags;

    let mut groups = HashMap::<String, Group>::new();
    for row in conn
        .prepare("SELECT * FROM PhotoGroup")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let id = row.read::<&str, _>("Id").to_string();
        groups.insert(
            id.clone(),
            Group {
                id,
                name: row.read::<&str, _>("name").to_string(),
            },
        );
    }

    *state.groups.lock().unwrap() = groups;

    let mut project_version = 0;
    let mut settings = HashMap::<String, Setting>::new();
    for row in conn
        .prepare("SELECT * FROM Setting")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        let setting = Setting {
            setting: row.read::<&str, _>("setting").to_string(),
            value: row.read::<i64, _>("value"),
        };
        if setting.setting == "version" {
            project_version = setting.value;
        }
        settings.insert(setting.setting.clone(), setting);
    }

    *state.settings.lock().unwrap() = settings.clone();

    if project_version < PHOTO_MANAGER_VERSION {
        println!("Database needs upgrade!");
    }

    *state.db.lock().unwrap() = conn;

    Ok(photo_load.removed)
}

#[derive(Debug)]
pub struct SearchTermError {}

impl Error for SearchTermError {}
impl Display for SearchTermError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid search term")
    }
}

fn term_matches(term: &String, test: &str) -> bool {
    if term.len() > test.len() {
        let substr = term.get(0..test.len());
        if substr.is_none() {
            return false;
        }
        return substr.unwrap().to_uppercase() == test;
    }
    false
}

fn try_get_term(term: &String, start: usize) -> Result<String, SearchTermError> {
    let val = term.get(start..);
    if val.is_none() {
        return Err(SearchTermError {});
    }
    Ok(val.unwrap().to_string())
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlite::Error),
    #[error("Search term error: {0}")]
    TermError(#[from] SearchTermError),
    #[error("Could not parse provided date: {0}")]
    DateError(#[from] chrono::ParseError),
}

impl Serialize for SearchError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Performs a search of the photos using the given query.
fn search_photos(
    connection: &sqlite::Connection,
    query: Vec<String>,
    sort: String,
    tags: &HashMap<String, Tag>,
    people: &HashMap<String, Person>,
) -> Result<Vec<Photo>, SearchError> {
    let mut unmet_terms = Vec::<String>::new();

    // Construct a SQL statement using terms that require no additional processing (is:..., at:..., only:..., by:..., has:...)
    let mut statement = "SELECT * FROM Photo WHERE isDuplicate=0".to_string();
    let mut has_date = sort == "date" || sort == "date_desc";
    let mut has_date_negated = false;
    for term in query {
        let mut chars = term.chars();
        let negated = term.get(0..1).unwrap() == "-";
        if negated {
            chars.next();
        }
        let tmp_term = chars.as_str().to_string();
        if term_matches(&tmp_term, "AT:") {
            statement.push_str(&format!(
                " AND location{0}'{1}'",
                if negated { "!=" } else { "=" },
                esc(&try_get_term(&tmp_term, 3)?)
            ));
        } else if term_matches(&tmp_term, "IS:") {
            if try_get_term(&tmp_term, 4)?.to_uppercase() == "VIDEO" {
                statement.push_str(&format!(" AND video={}", if negated { 0 } else { 1 }));
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "RAW" {
                statement.push_str(&format!(" AND raw={}", if negated { 0 } else { 1 }));
            }
        } else if term_matches(&tmp_term, "ONLY:") {
            statement.push_str(&format!(
                " AND people{0}'{1}'",
                if negated { "!=" } else { "=" },
                esc(&try_get_term(&tmp_term, 6)?)
            ));
        } else if term_matches(&tmp_term, "BY:") {
            statement.push_str(&format!(
                " AND photographer{0}'{1}'",
                if negated { "!=" } else { "=" },
                esc(&try_get_term(&tmp_term, 4)?)
            ));
        } else if term_matches(&tmp_term, "HAS:") {
            if try_get_term(&tmp_term, 4)?.to_uppercase() == "RATING" {
                statement.push_str(&format!(" AND rating{0}0", if negated { "=" } else { ">" }));
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "PHOTOGRAPHER" {
                statement.push_str(&format!(
                    " AND length(photographer){0}0",
                    if negated { "=" } else { ">" }
                ));
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "DATE" {
                has_date = true;
                has_date_negated = negated;
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "LOCATION" {
                statement.push_str(&format!(
                    " AND length(location){0}0",
                    if negated { "=" } else { ">" }
                ));
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "PEOPLE" {
                statement.push_str(&format!(
                    " AND length(people){0}0",
                    if negated { "=" } else { ">" }
                ));
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "TAGS" {
                statement.push_str(&format!(
                    " AND length(tags){0}0",
                    if negated { "=" } else { ">" }
                ));
            }
        } else if term_matches(&tmp_term, "NAME:") {
            statement.push_str(&format!(
                " AND Name {0} LIKE '%{1}%'",
                if negated { "NOT" } else { "" },
                esc(&try_get_term(&tmp_term, 5)?)
            ));
        } else if term_matches(&tmp_term, "RATING=") {
            statement.push_str(&format!(
                " AND rating{0}{1}",
                if negated { "!=" } else { "=" },
                try_get_term(&tmp_term, 7)?
            ));
        } else {
            unmet_terms.push(term);
        }
    }

    if has_date {
        statement.push_str(&format!(
            " AND length(date){0}0",
            if has_date_negated { "=" } else { ">" }
        ));
    }

    let mut results = Vec::<Photo>::new();
    let mut photos = Vec::<Photo>::new();
    for row in connection.prepare(statement)? {
        let mut photo = row_to_photo(&row?);
        let validation = validate_tags(tags, &photo.tags);
        photo.valid_tags = validation.is_valid;
        photo.validation_msg = validation.message;
        photos.push(photo);
    }

    // I *want* to use SQL ORDER BY to sort the results, but it seems the results lose their order somewhere in the above statement
    // TODO test again with sql ordering

    // Terms that require additional processing and iterating over the photos (date:..., of:..., any tags)
    if unmet_terms.len() > 0 {
        for photo in photos {
            let mut meets_terms = true;
            for term in &unmet_terms {
                let mut chars = term.chars();
                let negated = term.get(0..1).unwrap() == "-";
                if negated {
                    chars.next();
                }
                let tmp_term = chars.as_str().to_string();
                if term_matches(&tmp_term, "OF:") {
                    let q = &tmp_term.get(3..).unwrap().to_string();
                    let mut in_photo = photo.people.contains(q);
                    if !in_photo {
                        in_photo = photo
                            .people
                            .iter()
                            .map(|id| people.get(id).unwrap().name.to_uppercase())
                            .collect::<Vec<String>>()
                            .contains(&q.to_uppercase());
                    }
                    meets_terms = meets_terms && ((negated && !in_photo) || (!negated && in_photo));
                }
                if term_matches(&tmp_term, "DATE:") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, "&F")?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() != comp_date)
                                || (!negated && photo.date.unwrap() == comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE>=") {
                    let comp_date = NaiveDate::parse_from_str(&try_get_term(&tmp_term, 6)?, "%F")?;
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() < comp_date)
                                || (!negated && photo.date.unwrap() >= comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE<=") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 6)?, "%F")?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() > comp_date)
                                || (!negated && photo.date.unwrap() <= comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE>") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, "%F")?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() < comp_date)
                                || (!negated && photo.date.unwrap() > comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE<") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, "%F")?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() > comp_date)
                                || (!negated && photo.date.unwrap() < comp_date));
                    }
                } else {
                    // Treat all remaining terms as tags
                    let in_tags = photo.tags.contains(&tmp_term);
                    meets_terms = meets_terms && ((negated && !in_tags) || (!negated && in_tags));
                }
            }
            if meets_terms {
                if sort == "name" || sort == "name_desc" {
                    match results.binary_search_by_key(&photo.name, |p| p.name.clone()) {
                        Ok(_pos) => {}
                        Err(pos) => results.insert(pos, photo.clone()),
                    }
                } else if sort == "rating" || sort == "rating_desc" {
                    match results.binary_search_by_key(&photo.rating, |p| p.rating) {
                        Ok(pos) => results.insert(pos, photo.clone()),
                        Err(pos) => results.insert(pos, photo.clone()),
                    }
                } else if sort == "date" || sort == "date_desc" {
                    match results.binary_search_by_key(&photo.date, |p| p.date) {
                        Ok(pos) => results.insert(pos, photo.clone()),
                        Err(pos) => results.insert(pos, photo.clone()),
                    }
                }
            }
        }
    } else {
        results = photos.into_iter().collect::<Vec<Photo>>();
        if sort == "name" || sort == "name_desc" {
            results.sort_by_key(|p| p.name.clone());
        } else if sort == "rating" || sort == "rating_desc" {
            results.sort_by_key(|p| p.rating);
        } else if sort == "date" || sort == "date_desc" {
            results.sort_by_key(|p| p.date);
        }
    }

    if sort == "name_desc" || sort == "rating_desc" || sort == "date_desc" {
        results.reverse();
    }

    Ok(results)
}

#[tauri::command]
pub fn photo_grid(
    state: State<'_, PhotoState>,
    query: Vec<String>,
    sort: String,
) -> Result<Vec<Photo>, SearchError> {
    let connection = state.db.lock().unwrap();

    Ok(search_photos(
        &connection,
        query,
        sort,
        &state.tags.lock().unwrap(),
        &state.people.lock().unwrap(),
    )?)
}

#[tauri::command]
pub fn remove_deleted(state: State<'_, PhotoState>, deleted: Vec<String>) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();
    let cloned = state_photos.clone();

    for name in deleted {
        connection.execute(format!("DELETE FROM Photo WHERE Name='{0}'", esc(&name)))?;

        let test = cloned
            .iter()
            .find_map(|(key, value)| if name == value.name { Some(key) } else { None });
        if test.is_some() {
            state_photos.remove(test.unwrap());
        }
    }
    Ok(())
}

pub fn get_photo_targets(id: &String, connection: &Connection) -> Result<Vec<Photo>, ApiError> {
    let mut targets = Vec::<Photo>::new();
    targets.push(row_to_photo(
        &connection
            .prepare(format!("SELECT * FROM Photo WHERE Id='{0}'", esc(id)))?
            .into_iter()
            .map(|row| row.unwrap())
            .last()
            .unwrap(),
    ));
    let existing_group = &targets.first().unwrap().photo_group;
    if existing_group.len() > 0 {
        for row in connection.prepare(format!(
            "SELECT * FROM Photo WHERE photoGroup='{existing_group}' AND Id!='{0}'",
            esc(id)
        ))? {
            targets.push(row_to_photo(&row?));
        }
    }
    Ok(targets)
}

#[tauri::command]
pub fn set_photo_str(
    state: State<'_, PhotoState>,
    photo: String,
    property: String,
    value: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Photo SET {property}='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&photo)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn set_photographer(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();
    let mut state_people = state.people.lock().unwrap();

    let targets = get_photo_targets(&photo, &connection)?;
    for target in &targets {
        connection.execute(format!(
            "UPDATE Photo SET photographer='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&target.id)
        ))?;
        if state_photos.contains_key(&target.id) {
            state_photos.get_mut(&target.id).unwrap().photographer = value.clone();
        }
    }

    let count = targets.len() as i64;
    let existing_photographer = &targets.first().unwrap().photographer;
    if *existing_photographer != value {
        if state_people.contains_key(existing_photographer) {
            state_people
                .get_mut(existing_photographer)
                .unwrap()
                .photographer_count -= count;
        }
        state_people.get_mut(&value).unwrap().photographer_count += count;
    }

    Ok(())
}

#[tauri::command]
pub fn set_photo_people(
    state: State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();
    let mut state_people = state.people.lock().unwrap();

    let targets = get_photo_targets(&photo, &connection)?;
    let joined_people = esc(&value.join(","));
    for target in &targets {
        connection.execute(format!(
            "UPDATE Photo SET people='{joined_people}' WHERE Id='{0}'",
            esc(&target.id)
        ))?;
        if state_photos.contains_key(&target.id) {
            state_photos.get_mut(&target.id).unwrap().people = value.clone();
        }
    }

    let count = targets.len() as i64;
    let existing_people = &targets.first().unwrap().people;
    for person in &value {
        if !existing_people.contains(person) && state_people.contains_key(person) {
            state_people.get_mut(person).unwrap().photo_count += count;
        }
    }
    for person in existing_people {
        if !value.contains(person) && state_people.contains_key(person) {
            state_people.get_mut(person).unwrap().photo_count -= count;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn set_photo_location(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();
    let mut state_places = state.places.lock().unwrap();

    let targets = get_photo_targets(&photo, &connection)?;
    let existing_location = &targets.first().unwrap().location;

    for target in &targets {
        connection.execute(format!(
            "UPDATE Photo SET location='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&target.id)
        ))?;
        if state_photos.contains_key(&target.id) {
            state_photos.get_mut(&target.id).unwrap().location = value.clone();
        } else {
            return Err(ApiError::NotFoundError(format!(
                "Photo {} not found",
                target.id
            )));
        }
    }

    let count = targets.len() as i64;
    if existing_location.len() > 0 && state_places.contains_key(existing_location) {
        state_places.get_mut(existing_location).unwrap().count -= count;
    }
    if value.len() > 0 && state_places.contains_key(&value) {
        state_places.get_mut(&value).unwrap().count += count;
    }

    Ok(())
}

#[tauri::command]
pub fn set_photo_tags(
    state: State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<ValidationResult, ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();

    let targets = get_photo_targets(&photo, &connection)?;
    let existing_tags = &targets.first().unwrap().tags;

    let validation = validate_tags(&state_tags, &value);
    for target in &mut targets.clone() {
        connection.execute(format!(
            "UPDATE Photo SET tags='{0}' WHERE Id='{1}'",
            esc(&value.join(",")),
            esc(&target.id)
        ))?;
        target.tags = value.clone();
        target.valid_tags = validation.is_valid;
        target.validation_msg = validation.message.clone();
        if state_photos.contains_key(&target.id) {
            *state_photos.get_mut(&target.id).unwrap() = target.clone();
        } else {
            return Err(ApiError::NotFoundError(format!(
                "Photo {} not found",
                target.id
            )));
        }
    }

    let count = targets.len() as i64;
    for tag in &value {
        if !state_tags.contains_key(tag) {
            state_tags.insert(tag.clone(), Tag::new(tag));
        }
        if !existing_tags.contains(tag) {
            state_tags.get_mut(tag).unwrap().count += count;
        }
    }
    for tag in existing_tags {
        if !value.contains(tag) && state_tags.contains_key(tag) {
            state_tags.get_mut(tag).unwrap().count -= count;
        }
    }

    Ok(validation)
}

#[tauri::command]
pub fn set_photo_date(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();

    for target in &get_photo_targets(&photo, &connection)? {
        connection.execute(format!(
            "UPDATE Photo SET date='{0}' WHERE Id='{1}'",
            esc(&value),
            esc(&target.id)
        ))?;

        if state_photos.contains_key(&target.id) {
            state_photos.get_mut(&target.id).unwrap().date =
                Some(NaiveDate::parse_from_str(&value, DATE_FORMAT)?);
        } else {
            return Err(ApiError::NotFoundError(format!(
                "Photo {} not found",
                target.id
            )));
        }
    }

    Ok(())
}

#[tauri::command]
pub fn set_photo_group(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();

    let targets = get_photo_targets(&photo, &connection)?;
    if value.len() == 0 {
        for target in &targets {
            connection
                .execute(format!(
                    "UPDATE Photo SET photoGroup='' WHERE Id='{0}'",
                    esc(&target.id)
                ))
                .unwrap();
        }
    } else {
        let mut collected_tags = HashSet::<String>::new();
        let mut collected_location: Option<String> = None;
        let mut collected_people = HashSet::<String>::new();
        let mut collected_photographer: Option<String> = None;
        let mut collected_date: Option<NaiveDate> = None;
        for row in &targets {
            for tag in &row.tags {
                collected_tags.insert(tag.clone());
            }
            if collected_location.clone().is_none()
                || (row.id == photo && row.location != collected_location.clone().unwrap())
            {
                collected_location = Some(row.location.clone());
            }
            for person in &row.people {
                collected_people.insert(person.clone());
            }
            if collected_photographer.clone().is_none()
                || (row.id == photo && row.photographer != collected_photographer.clone().unwrap())
            {
                collected_photographer = Some(row.photographer.clone());
            }
            if row.date.is_some()
                && (collected_date.is_none()
                    || (row.id == photo && row.date.unwrap() != collected_date.unwrap()))
            {
                collected_date = row.date;
            }
        }

        let tags_vec = collected_tags.into_iter().collect();
        let people_vec: Vec<String> = collected_people.into_iter().collect();
        let validation = validate_tags(&state.tags.lock().unwrap(), &tags_vec);
        let joined_tags = esc(&tags_vec.join(","));
        let joined_people = esc(&people_vec.join(","));
        let mut date_str = String::new();
        if collected_date.is_some() {
            date_str = collected_date.unwrap().format(DATE_FORMAT).to_string();
        }
        let location_str = collected_location.clone().unwrap_or(String::new());
        let photographer_str = collected_photographer.clone().unwrap_or(String::new());

        for row in &targets {
            if !state_photos.contains_key(&row.id) {
                state_photos.insert(row.id.clone(), row.clone());
            }

            let target_photo = state_photos.get_mut(&row.id).unwrap();
            connection.execute(format!(
                "UPDATE Photo SET photoGroup='{0}', tags='{joined_tags}', location='{1}', people='{joined_people}', photographer='{2}', date='{3}' WHERE Id='{4}'",
                esc(&value),
                esc(&location_str),
                esc(&photographer_str),
                esc(&date_str),
                esc(&row.id)
            )).unwrap();
            target_photo.photo_group = value.clone();
            target_photo.tags = tags_vec.clone();
            target_photo.valid_tags = validation.is_valid;
            target_photo.validation_msg = validation.message.clone();
            target_photo.location = location_str.clone();
            target_photo.people = people_vec.clone();
            target_photo.photographer = photographer_str.clone();
            if collected_date.is_some() {
                target_photo.date = Some(collected_date.unwrap());
            } else {
                target_photo.date = None;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn set_photo_rating(
    state: State<'_, PhotoState>,
    photo: String,
    rating: i64,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Photo SET rating={rating} WHERE Id='{0}'",
        esc(&photo)
    ))?;

    let mut state_photos = state.photos.lock().unwrap();
    if state_photos.contains_key(&photo) {
        state_photos.get_mut(&photo).unwrap().rating = rating;
    } else {
        return Err(ApiError::NotFoundError(format!(
            "Photo {} not found",
            photo
        )));
    }

    Ok(())
}

#[tauri::command]
pub fn set_photo_bool(
    state: State<'_, PhotoState>,
    photo: String,
    property: String,
    value: bool,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Photo SET {property}={value} WHERE Id='{0}'",
        esc(&photo)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn refresh<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PhotoState>,
    path: String,
) -> Result<Vec<String>, ApiError> {
    let conn = state.db.lock().unwrap();
    let mut tags = state.tags.lock().unwrap();
    let mut people = state.people.lock().unwrap();
    let mut places = state.places.lock().unwrap();

    let photo_load = load_photos(
        &conn,
        &mut tags,
        &mut people,
        &mut places,
        &path,
        &app.path().app_data_dir()?.join("thumbnails"),
        false,
    )?;
    *state.photos.lock().unwrap() = photo_load.photos;

    Ok(photo_load.removed)
}
