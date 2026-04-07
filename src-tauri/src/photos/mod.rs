use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    fmt::{self, Display, Formatter},
    ops::Deref,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    time::Duration,
};

use chrono::NaiveDate;
use diesel::{
    dsl::{delete, insert_into, update},
    query_builder::AsChangeset,
    result, BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, SqliteConnection,
    TextExpressionMethods,
};
use diesel_async::{sync_connection_wrapper::SyncConnectionWrapper, AsyncConnection, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use lazy_static::lazy_static;
use log::{debug, error, info};
use regex::Regex;
use rusty_pool::{JoinHandle, ThreadPool};
use serde::Serialize;
use strum::EnumString;
use tauri::{AppHandle, Manager, Runtime, State};
use thiserror::Error;
use tokio::{fs, sync::Mutex};
use walkdir::WalkDir;

use crate::{
    models::{Layer, Person, Photo, PhotoGroup, Place, Setting, Tag},
    row_to_vec,
    schema::{layers, people, photo_groups, photos, places, settings, tags},
    tags::{validate_tags, ValidationResult},
    ApiError, MIGRATIONS,
};

const DATE_FORMAT: &str = "%F";

lazy_static! {
    static ref RAW: Regex = Regex::new(r"^.*\.(ORF|NRW|HEIC|TIFF|TIF)$").unwrap();
    static ref VIDEO: Regex =
        Regex::new(r"^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM|FLV)$").unwrap();
}

impl Photo {
    fn new(filename: String) -> Self {
        Self {
            name: filename.clone(),
            asset_path: format!(
                "https://asset.localhost/{0}",
                url_escape::encode_component(&filename)
            ),
            title: None,
            description: None,
            tags: None,
            is_duplicate: None,
            rating: None,
            location: None,
            thumbnail: None,
            photo_group: None,
            date: None,
            people: None,
            hide_thumbnail: None,
            photographer: None,
        }
    }
}

pub struct PhotoState {
    pub db: Mutex<Option<SyncConnectionWrapper<SqliteConnection>>>,
    pub photos: Mutex<HashMap<String, Photo>>,
    pub people: Mutex<HashMap<String, Person>>,
    pub places: Mutex<HashMap<String, Place>>,
    pub layers: Mutex<HashMap<String, Layer>>,
    pub tags: Mutex<HashMap<String, Tag>>,
    pub groups: Mutex<HashMap<String, PhotoGroup>>,
    pub settings: Mutex<HashMap<String, Setting>>,
}

impl Default for PhotoState {
    fn default() -> Self {
        Self {
            db: Mutex::new(None),
            photos: Mutex::new(HashMap::<String, Photo>::new()),
            people: Mutex::new(HashMap::<String, Person>::new()),
            places: Mutex::new(HashMap::<String, Place>::new()),
            layers: Mutex::new(HashMap::<String, Layer>::new()),
            tags: Mutex::new(HashMap::<String, Tag>::new()),
            groups: Mutex::new(HashMap::<String, PhotoGroup>::new()),
            settings: Mutex::new(HashMap::<String, Setting>::new()),
        }
    }
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

#[derive(Debug, Error)]
pub enum LoadPhotoError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] result::Error),
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

async fn load_photos(
    conn: &mut SyncConnectionWrapper<SqliteConnection>,
    tags: &mut HashMap<String, Tag>,
    people: &mut HashMap<String, Person>,
    places: &mut HashMap<String, Place>,
    path: &String,
    thumbnail_dir: &PathBuf,
    counts: bool,
) -> anyhow::Result<LoadedPhotos> {
    info!("Loading photos from {path}");
    // Photos stored in the database, which does not necessarily reflect photos actually present in the folder
    let mut existing = HashMap::<String, Photo>::new();

    for row in photos::table.load::<Photo>(conn).await? {
        let mut photo = row;
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

    // Read the files in the selected folder
    let mut file_queue = VecDeque::<PathBuf>::new();
    let mut dir = fs::read_dir(&path).await?;
    while let Some(entry) = dir.next_entry().await? {
        file_queue.push_back(entry.path());
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
                    if existing_photo.photographer.is_some() {
                        let photographer = existing_photo.photographer.as_ref().unwrap();
                        if people.contains_key(photographer) {
                            people.get_mut(photographer).unwrap().photographer_count += 1;
                        }
                    }
                    if existing_photo.location.is_some() {
                        let location = existing_photo.photo.location.as_ref().unwrap();
                        if places.contains_key(location) {
                            places.get_mut(location).unwrap().count += 1;
                        }
                    }
                    for tag in &existing_photo.tags {
                        if tags.contains_key(tag) {
                            tags.get_mut(tag).unwrap().count += 1;
                        }
                    }
                }
                photos.insert(existing_photo.name.clone(), existing_photo.clone());
                existing.remove(&filename.to_string());
            } else {
                let thumbnail_path = thumbnail_dir.join(clean_thumbnail_path(&filename));
                if RAW.is_match(&filename.to_uppercase()) {
                    if !thumbnail_path.exists() {
                        // Convert to jpg
                        threads.push(pool.complete(async move {
                            debug!("Generating thumbnail for {filename}");
                            let output = Command::new("magick")
                                .args([&filename, thumbnail_path.to_str().unwrap()])
                                .output();
                            if output.is_err() {
                                error!(
                                    "ERROR: Could not generate thumbnail for {0}: {1}",
                                    &filename.to_string(),
                                    &output.err().unwrap().to_string()
                                );
                            }
                            let mut photo = Photo::new(filename);
                            photo.thumbnail = Some(thumbnail_path.to_str().unwrap().to_string());
                            photo.is_raw = true;
                            photo
                        }));
                    }
                } else if VIDEO.is_match(&filename.to_uppercase()) {
                    if !thumbnail_path.exists() {
                        threads.push(pool.complete(async move {
                            debug!("Generating thumbnail for {filename}");
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
                                error!(
                                    "ERROR: Could not generate thumbnail for {0}: {1}",
                                    &filename.to_string(),
                                    &output.err().unwrap().to_string()
                                );
                            }
                            let mut photo = Photo::new(filename);
                            photo.thumbnail = Some(thumbnail_path.to_str().unwrap().to_string());
                            photo.is_video = true;
                            photo
                        }));
                    }
                } else {
                    let photo = Photo::new(filename);
                    insert_into(photos::table)
                        .values(&photo)
                        .execute(conn)
                        .await?;
                    photos.insert(photo.name.clone(), photo);
                }
            }
        }
    }

    for thread in threads {
        let result = thread.await_complete();
        insert_into(photos::table)
            .values(result.clone())
            .execute(conn)
            .await?;
        photos.insert(result.name.clone(), result);
    }

    Ok(LoadedPhotos {
        photos,
        removed: existing.keys().cloned().map(String::from).collect(),
    })
}

/// Sets the working folder path & initializes the SQLite database connection.
/// Returns initial information from the database.
#[tauri::command]
pub async fn initialize<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PhotoState>,
    path: String,
) -> Result<Vec<String>, ApiError> {
    debug!("Initializing with path {path}");
    // Establish a sync connection to apply migrations
    let db_path = Path::new(&path).join("photos.db");
    let db_path = db_path.to_str().unwrap();
    let mut conn = SqliteConnection::establish(db_path)?;
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    let mut conn = SyncConnectionWrapper::<SqliteConnection>::establish(db_path).await?;

    let thumbnail_dir = app.path().app_data_dir()?.join("thumbnails");
    if !thumbnail_dir.exists() {
        fs::create_dir_all(&thumbnail_dir).await?;
    }

    let mut tags = HashMap::<String, Tag>::new();
    for tag in tags::table.load::<Tag>(&mut conn).await? {
        tags.insert(tag.name.clone(), Tag::from(tag));
    }

    let mut layers = HashMap::<String, Layer>::new();
    for layer in layers::table.load::<Layer>(&mut conn).await? {
        layers.insert(layer.id.clone(), Layer::from(layer));
    }

    let mut places = HashMap::<String, Place>::new();
    for place in places::table.load::<Place>(&mut conn).await? {
        if layers.contains_key(&place.layer) {
            layers.get_mut(&place.layer).unwrap().count += 1;
        }
        places.insert(place.id.clone(), Place::from(place));
    }

    let mut people = HashMap::<String, Person>::new();
    for person in people::table.load::<Person>(&mut conn).await? {
        people.insert(person.id.clone(), Person::from(person));
    }

    let photo_load = load_photos(
        &mut conn,
        &mut tags,
        &mut people,
        &mut places,
        &path,
        &thumbnail_dir,
        true,
    )
    .await?;

    *state.photos.lock().await = photo_load.photos;
    *state.people.lock().await = people;
    *state.places.lock().await = places;
    *state.layers.lock().await = layers;
    *state.tags.lock().await = tags;

    let mut groups = HashMap::<String, PhotoGroup>::new();
    for row in photo_groups::table.load::<PhotoGroup>(&mut conn).await? {
        groups.insert(row.id.clone(), row);
    }

    *state.groups.lock().await = groups;

    let mut settings = HashMap::<String, Setting>::new();
    for setting in settings::table.load::<Setting>(&mut conn).await? {
        settings.insert(setting.setting.clone(), setting);
    }

    *state.settings.lock().await = settings.clone();

    *state.db.lock().await = Some(conn);

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
    DatabaseError(#[from] result::Error),
    #[error("Search term error: {0}")]
    TermError(#[from] SearchTermError),
    #[error("Could not parse provided date: {0}")]
    DateError(#[from] chrono::ParseError),
    #[error("Invalid sort option: {0}")]
    SortTermError(#[from] strum::ParseError),
    #[error("{0}")]
    AnyError(#[from] anyhow::Error),
}

impl Serialize for SearchError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(EnumString, PartialEq)]
enum Sort {
    #[strum(ascii_case_insensitive)]
    Date,
    #[strum(ascii_case_insensitive)]
    DateDesc,
    #[strum(ascii_case_insensitive)]
    Name,
    #[strum(ascii_case_insensitive)]
    NameDesc,
    #[strum(ascii_case_insensitive)]
    Rating,
    #[strum(ascii_case_insensitive)]
    RatingDesc,
}

impl Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Sort::Date => write!(f, "date"),
            Sort::DateDesc => write!(f, "date descending"),
            Sort::Name => write!(f, "name"),
            Sort::NameDesc => write!(f, "name descending"),
            Sort::Rating => write!(f, "rating"),
            Sort::RatingDesc => write!(f, "rating descending"),
        }
    }
}

/// Performs a search of the photos using the given query.
async fn search_photos(
    connection: &mut SyncConnectionWrapper<SqliteConnection>,
    query: Vec<String>,
    sort: Sort,
    tags: &HashMap<String, Tag>,
    people: &HashMap<String, Person>,
) -> anyhow::Result<Vec<Photo>> {
    debug!(
        "Searching photos with query \"{0}\", sorted by {1}",
        query.join(","),
        sort
    );
    let mut unmet_terms = Vec::<String>::new();

    // Construct a SQL statement using terms that require no additional processing (is:..., at:..., only:..., by:..., has:...)
    let mut statement = photos::table
        .filter(photos::is_duplicate.eq(0))
        .into_boxed();
    let mut has_date = sort == Sort::Date || sort == Sort::DateDesc;
    let mut has_date_negated = false;
    for term in query {
        let mut chars = term.chars();
        let negated = term.get(0..1).unwrap() == "-";
        if negated {
            chars.next();
        }
        let tmp_term = chars.as_str().to_string();
        if term_matches(&tmp_term, "AT:") {
            let location = try_get_term(&tmp_term, 3)?;
            if negated {
                statement = statement.filter(photos::location.ne(location));
            } else {
                statement = statement.filter(photos::location.eq(location));
            }
        } else if term_matches(&tmp_term, "ONLY:") {
            let person = try_get_term(&tmp_term, 6)?;
            if negated {
                statement = statement.filter(photos::people.ne(person));
            } else {
                statement = statement.filter(photos::people.eq(person));
            }
        } else if term_matches(&tmp_term, "BY:") {
            let photographer = try_get_term(&tmp_term, 4)?;
            if negated {
                statement = statement.filter(photos::photographer.ne(photographer));
            } else {
                statement = statement.filter(photos::photographer.ne(photographer));
            }
        } else if term_matches(&tmp_term, "HAS:") {
            if try_get_term(&tmp_term, 4)?.to_uppercase() == "RATING" {
                if negated {
                    statement = statement.filter(photos::rating.is_null());
                } else {
                    statement = statement.filter(photos::rating.is_not_null());
                }
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "PHOTOGRAPHER" {
                if negated {
                    statement = statement.filter(photos::photographer.is_null());
                } else {
                    statement = statement.filter(photos::photographer.is_not_null());
                }
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "DATE" {
                has_date = true;
                has_date_negated = negated;
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "LOCATION" {
                if negated {
                    statement = statement.filter(photos::location.is_null());
                } else {
                    statement = statement.filter(photos::location.is_not_null());
                }
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "PEOPLE" {
                if negated {
                    statement = statement.filter(photos::people.is_null());
                } else {
                    statement = statement.filter(photos::people.is_not_null());
                }
            } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "TAGS" {
                if negated {
                    statement = statement.filter(photos::tags.is_null());
                } else {
                    statement = statement.filter(photos::tags.is_not_null());
                }
            }
        } else if term_matches(&tmp_term, "NAME:") {
            let name = try_get_term(&tmp_term, 5)?;
            if negated {
                statement = statement.filter(photos::name.not_like(name));
            } else {
                statement = statement.filter(photos::name.like(name));
            }
        } else if term_matches(&tmp_term, "RATING<=") {
            let rating = try_get_term(&tmp_term, 8)?.parse::<i32>()?;
            if negated {
                statement = statement.filter(photos::rating.gt(rating));
            } else {
                statement = statement.filter(photos::rating.le(rating));
            }
        } else if term_matches(&tmp_term, "RATING>=") {
            let rating = try_get_term(&tmp_term, 8)?.parse::<i32>()?;
            if negated {
                statement = statement.filter(photos::rating.lt(rating));
            } else {
                statement = statement.filter(photos::rating.ge(rating));
            }
        } else if term_matches(&tmp_term, "RATING<") {
            let rating = try_get_term(&tmp_term, 8)?.parse::<i32>()?;
            if negated {
                statement = statement.filter(photos::rating.ge(rating));
            } else {
                statement = statement.filter(photos::rating.lt(rating));
            }
        } else if term_matches(&tmp_term, "RATING>") {
            let rating = try_get_term(&tmp_term, 8)?.parse::<i32>()?;
            if negated {
                statement = statement.filter(photos::rating.le(rating));
            } else {
                statement = statement.filter(photos::rating.gt(rating));
            }
        } else if term_matches(&tmp_term, "RATING=") {
            let rating = try_get_term(&tmp_term, 7)?.parse::<i32>()?;
            if negated {
                statement = statement.filter(photos::rating.ne(rating));
            } else {
                statement = statement.filter(photos::rating.eq(rating));
            }
        } else {
            unmet_terms.push(term);
        }
    }

    if has_date {
        if has_date_negated {
            statement = statement.filter(photos::date.is_null());
        } else {
            statement = statement.filter(photos::date.is_not_null());
        }
    }

    let mut results = Vec::<Photo>::new();
    let mut photo_records = Vec::<Photo>::new();
    for row in statement.load::<Photo>(connection).await? {
        let mut photo = Photo::from(row);
        let validation = validate_tags(tags, &photo.tags);
        photo.valid_tags = validation.is_valid;
        photo.validation_msg = validation.message;
        photo_records.push(photo);
    }

    // I *want* to use SQL ORDER BY to sort the results, but it seems the results lose their order somewhere in the above statement
    // TODO test again with sql ordering

    // Terms that require additional processing and iterating over the photos (date:..., of:..., any tags)
    if unmet_terms.len() > 0 {
        for photo in photo_records {
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
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, DATE_FORMAT)?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() != comp_date)
                                || (!negated && photo.date.unwrap() == comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE>=") {
                    let comp_date =
                        NaiveDate::parse_from_str(&try_get_term(&tmp_term, 6)?, DATE_FORMAT)?;
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
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 6)?, DATE_FORMAT)?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() > comp_date)
                                || (!negated && photo.date.unwrap() <= comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE>") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, DATE_FORMAT)?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() < comp_date)
                                || (!negated && photo.date.unwrap() > comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE<") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, DATE_FORMAT)?;
                        meets_terms = meets_terms
                            && ((negated && photo.date.unwrap() > comp_date)
                                || (!negated && photo.date.unwrap() < comp_date));
                    }
                } else if term_matches(&tmp_term, "IS:") {
                    if try_get_term(&tmp_term, 4)?.to_uppercase() == "VIDEO" {
                        meets_terms = meets_terms
                            && ((negated && !photo.is_video) || (!negated && photo.is_video));
                    } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "RAW" {
                        meets_terms = meets_terms
                            && ((negated && !photo.is_raw) || (!negated && photo.is_raw));
                    }
                } else {
                    // Treat all remaining terms as tags
                    let in_tags = photo.tags.contains(&tmp_term);
                    meets_terms = meets_terms && ((negated && !in_tags) || (!negated && in_tags));
                }
            }
            if meets_terms {
                if sort == Sort::Name || sort == Sort::NameDesc {
                    match results.binary_search_by_key(&photo.name, |p| p.name.clone()) {
                        Ok(_pos) => {}
                        Err(pos) => results.insert(pos, photo.clone()),
                    }
                } else if sort == Sort::Rating || sort == Sort::RatingDesc {
                    match results.binary_search_by_key(&photo.rating, |p| p.rating) {
                        Ok(pos) => results.insert(pos, photo.clone()),
                        Err(pos) => results.insert(pos, photo.clone()),
                    }
                } else if sort == Sort::Date || sort == Sort::DateDesc {
                    match results.binary_search_by_key(&photo.date, |p| p.date) {
                        Ok(pos) => results.insert(pos, photo.clone()),
                        Err(pos) => results.insert(pos, photo.clone()),
                    }
                }
            }
        }
    } else {
        results = photo_records.into_iter().collect::<Vec<Photo>>();
        if sort == Sort::Name || sort == Sort::NameDesc {
            results.sort_by_key(|p| p.name.clone());
        } else if sort == Sort::Rating || sort == Sort::RatingDesc {
            results.sort_by_key(|p| p.rating);
        } else if sort == Sort::Date || sort == Sort::DateDesc {
            results.sort_by_key(|p| p.date);
        }
    }

    if sort == Sort::NameDesc || sort == Sort::RatingDesc || sort == Sort::DateDesc {
        results.reverse();
    }

    Ok(results)
}

#[tauri::command]
pub async fn photo_grid(
    state: State<'_, PhotoState>,
    query: Vec<String>,
    sort: String,
) -> Result<Vec<Photo>, SearchError> {
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();

    Ok(search_photos(
        connection,
        query,
        Sort::from_str(&sort)?,
        state.tags.lock().await.deref(),
        state.people.lock().await.deref(),
    )
    .await?)
}

#[tauri::command]
pub async fn remove_deleted(
    state: State<'_, PhotoState>,
    deleted: Vec<String>,
) -> Result<(), ApiError> {
    debug!(
        "Removing {} moved or deleted photos from the database",
        deleted.len()
    );
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();
    let mut state_photos = state.photos.lock().await;
    let cloned = state_photos.clone();

    for name in deleted {
        delete(photos::table.filter(photos::name.eq(&name)))
            .execute(connection)
            .await?;

        let test = cloned.iter().find_map(
            |(key, value)| {
                if name == value.name {
                    Some(key)
                } else {
                    None
                }
            },
        );
        if test.is_some() {
            state_photos.remove(test.unwrap());
        }
    }
    Ok(())
}

pub async fn get_photo_targets(
    id: &String,
    connection: &mut SyncConnectionWrapper<SqliteConnection>,
) -> anyhow::Result<Vec<Photo>> {
    let mut targets = Vec::<Photo>::new();
    targets.push(
        photos::table
            .filter(photos::name.eq(id))
            .first::<Photo>(connection)
            .await?,
    );
    let existing_group = &targets[0].photo_group;
    if existing_group.is_some() {
        for row in photos::table
            .filter(
                photos::photo_group
                    .eq(existing_group.as_ref().unwrap())
                    .and(photos::name.ne(id)),
            )
            .load::<Photo>(connection)
            .await?
        {
            targets.push(row);
        }
    }
    Ok(targets)
}

#[tauri::command]
pub async fn set_photo_title(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} title to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();

    update(photos::table.filter(photos::name.eq(photo)))
        .set(photos::title.eq(value))
        .execute(connection)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_desc(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} description to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();

    update(photos::table.filter(photos::name.eq(photo)))
        .set(photos::description.eq(value))
        .execute(connection)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_photographer(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} photographer to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();
    let mut state_photos = state.photos.lock().await;
    let mut state_people = state.people.lock().await;

    let targets = get_photo_targets(&photo, connection).await?;
    for target in &targets {
        update(photos::table.filter(photos::name.eq(target.name.clone())))
            .set(photos::photographer.eq(value.clone()))
            .execute(connection)
            .await?;
        if state_photos.contains_key(&target.name) {
            state_photos.get_mut(&target.name).unwrap().photographer = Some(value.clone());
        }
    }

    let count = targets.len() as i64;
    let existing_photographer = &targets[0].photographer;
    if existing_photographer.is_some() {
        let existing_photographer = existing_photographer.as_ref().unwrap();
        if *existing_photographer != value {
            if state_people.contains_key(existing_photographer) {
                state_people
                    .get_mut(existing_photographer)
                    .unwrap()
                    .photographer_count -= count;
            }
            state_people.get_mut(&value).unwrap().photographer_count += count;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_people(
    state: State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} people to {}", value.join(","));
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();
    let mut state_photos = state.photos.lock().await;
    let mut state_people = state.people.lock().await;

    let targets = get_photo_targets(&photo, connection).await?;
    let joined_people = &value.join(",");
    for target in &targets {
        update(photos::table.filter(photos::name.eq(target.name.clone())))
            .set(photos::people.eq(joined_people))
            .execute(connection)
            .await?;
        if state_photos.contains_key(&target.name) {
            state_photos.get_mut(&target.name).unwrap().people = value.clone();
        }
    }

    let count = targets.len() as i64;
    let existing_people = row_to_vec(&targets[0].people);
    for person in &value {
        if !existing_people.contains(person) && state_people.contains_key(person) {
            state_people.get_mut(person).unwrap().photo_count += count;
        }
    }
    for person in existing_people {
        if !value.contains(&person) && state_people.contains_key(&person) {
            state_people.get_mut(&person).unwrap().photo_count -= count;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_location(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} location to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();
    let mut state_photos = state.photos.lock().await;
    let mut state_places = state.places.lock().await;

    let targets = get_photo_targets(&photo, connection).await?;
    let existing_location = &targets[0].location;

    for target in &targets {
        update(photos::table.filter(photos::name.eq(target.name.clone())))
            .set(photos::location.eq(value.clone()))
            .execute(connection)
            .await?;
        if state_photos.contains_key(&target.name) {
            state_photos.get_mut(&target.name).unwrap().location = Some(value.clone());
        } else {
            return Err(ApiError::NotFoundError(format!(
                "Photo {} not found",
                target.name
            )));
        }
    }

    let count = targets.len() as i64;
    if existing_location.is_some() && state_places.contains_key(existing_location.as_ref().unwrap())
    {
        state_places
            .get_mut(existing_location.as_ref().unwrap())
            .unwrap()
            .count -= count;
    }
    if value.len() > 0 && state_places.contains_key(&value) {
        state_places.get_mut(&value).unwrap().count += count;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_tags(
    state: State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<ValidationResult, ApiError> {
    debug!("Setting photo {photo} tags to {}", value.join(","));
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();
    let mut state_photos = state.photos.lock().await;
    let mut state_tags = state.tags.lock().await;

    let targets = get_photo_targets(&photo, connection).await?;
    let existing_tags = &targets[0].tags;

    let validation = validate_tags(&state_tags, &value);
    for target in &mut targets.clone() {
        update(photos::table.filter(photos::name.eq(target.name.clone())))
            .set(photos::tags.eq(value.join(",")))
            .execute(connection)
            .await?;
        target.tags = value.clone();
        target.valid_tags = validation.is_valid;
        target.validation_msg = validation.message.clone();
        if state_photos.contains_key(&target.name) {
            *state_photos.get_mut(&target.name).unwrap() = target.clone();
        } else {
            return Err(ApiError::NotFoundError(format!(
                "Photo {} not found",
                target.name
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
pub async fn set_photo_date(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} date to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();
    let mut state_photos = state.photos.lock().await;

    for target in &get_photo_targets(&photo, connection).await? {
        update(photos::table.filter(photos::name.eq(target.name.clone())))
            .set(photos::date.eq(value.clone()))
            .execute(connection)
            .await?;

        if state_photos.contains_key(&target.name) {
            state_photos.get_mut(&target.name).unwrap().date =
                Some(NaiveDate::parse_from_str(&value, DATE_FORMAT)?);
        } else {
            return Err(ApiError::NotFoundError(format!(
                "Photo {} not found",
                target.name
            )));
        }
    }

    Ok(())
}

#[derive(AsChangeset)]
#[diesel(table_name = photos)]
struct GroupFields {
    photo_group: String,
    tags: Option<String>,
    location: Option<String>,
    people: Option<String>,
    photographer: Option<String>,
    date: Option<String>,
}

#[tauri::command]
pub async fn set_photo_group(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} group to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();
    let mut state_photos = state.photos.lock().await;

    let targets = get_photo_targets(&photo, connection).await?;
    if value.len() == 0 {
        for target in &targets {
            update(photos::table.filter(photos::name.eq(target.name.clone())))
                .set(photos::photo_group.eq::<Option<String>>(None))
                .execute(connection)
                .await?;
        }
    } else {
        let mut collected_tags = HashSet::<String>::new();
        let mut collected_location: Option<String> = None;
        let mut collected_people = HashSet::<String>::new();
        let mut collected_photographer: Option<String> = None;
        let mut collected_date: Option<String> = None;
        for row in &targets {
            for tag in row_to_vec(&row.tags) {
                collected_tags.insert(tag.clone());
            }
            if collected_location.clone().is_none() || (row.name == photo && row.location.is_some())
            {
                collected_location = row.location.clone();
            }
            for person in row_to_vec(&row.people) {
                collected_people.insert(person.clone());
            }
            if collected_photographer.clone().is_none()
                || (row.name == photo && row.photographer.is_some())
            {
                collected_photographer = row.photographer.clone();
            }
            if row.date.is_some()
                && (collected_date.is_none() || (row.name == photo && row.date.is_some()))
            {
                collected_date = row.date.clone();
            }
        }

        let tags_vec = collected_tags.into_iter().collect();
        let people_vec: Vec<String> = collected_people.into_iter().collect();
        let state_tags = state.tags.lock().await;
        let validation = validate_tags(&state_tags, &tags_vec);
        let parsed_date = if collected_date.is_some() {
            Some(NaiveDate::parse_from_str(
                collected_date.as_ref().unwrap(),
                DATE_FORMAT,
            )?)
        } else {
            None
        };

        for row in &targets {
            if !state_photos.contains_key(&row.name) {
                state_photos.insert(row.name.clone(), row.clone());
            }

            let target_photo = state_photos.get_mut(&row.name).unwrap();
            update(photos::table.filter(photos::name.eq(row.name.clone())))
                .into_boxed()
                .set(GroupFields {
                    photo_group: value.clone(),
                    tags: if tags_vec.is_empty() {
                        None
                    } else {
                        Some(tags_vec.join(","))
                    },
                    location: collected_location.clone(),
                    people: if people_vec.is_empty() {
                        None
                    } else {
                        Some(people_vec.join(","))
                    },
                    photographer: collected_photographer.clone(),
                    date: collected_date.clone(),
                })
                .execute(connection)
                .await?;
            target_photo.photo_group = Some(value.clone());
            target_photo.tags = tags_vec.clone();
            target_photo.valid_tags = validation.is_valid;
            target_photo.validation_msg = validation.message.clone();
            target_photo.location = collected_location.clone();
            target_photo.people = people_vec.clone();
            target_photo.photographer = collected_photographer.clone();
            target_photo.date = parsed_date;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_rating(
    state: State<'_, PhotoState>,
    photo: String,
    rating: i32,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} rating to {rating}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();

    update(photos::table.filter(photos::name.eq(photo.clone())))
        .set(photos::rating.eq(rating))
        .execute(connection)
        .await?;

    let mut state_photos = state.photos.lock().await;
    if state_photos.contains_key(&photo) {
        state_photos.get_mut(&photo).unwrap().rating = Some(rating);
    } else {
        return Err(ApiError::NotFoundError(format!(
            "Photo {} not found",
            photo
        )));
    }

    Ok(())
}

#[tauri::command]
pub async fn set_photo_is_duplicate(
    state: State<'_, PhotoState>,
    photo: String,
    value: bool,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} duplicate to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();

    let int_val = if value { 1 } else { 0 };
    update(photos::table.filter(photos::name.eq(photo)))
        .set(photos::is_duplicate.eq(int_val))
        .execute(connection)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_hide_thumbnail(
    state: State<'_, PhotoState>,
    photo: String,
    value: bool,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} hide thumbnail to {value}");
    let mut connection = state.db.lock().await;
    let connection = connection.as_mut().unwrap();

    let int_val = if value { 1 } else { 0 };
    update(photos::table.filter(photos::name.eq(photo)))
        .set(photos::hide_thumbnail.eq(int_val))
        .execute(connection)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn refresh<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PhotoState>,
    path: String,
) -> Result<Vec<String>, ApiError> {
    debug!("Refreshing photos from {path}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();
    let mut tags = state.tags.lock().await;
    let mut people = state.people.lock().await;
    let mut places = state.places.lock().await;

    let photo_load = load_photos(
        conn,
        &mut tags,
        &mut people,
        &mut places,
        &path,
        &app.path().app_data_dir()?.join("thumbnails"),
        false,
    )
    .await?;
    *state.photos.lock().await = photo_load.photos;

    Ok(photo_load.removed)
}
