use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display, Formatter},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use diesel::{
    debug_query, delete, insert_into, BoolExpressionMethods, Connection, ExpressionMethods,
    QueryDsl, SqliteConnection, TextExpressionMethods,
};
use diesel_async::{sync_connection_wrapper::SyncConnectionWrapper, AsyncConnection, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use exif::In;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use regex::Regex;
use rusty_pool::ThreadPool;
use serde::{Serialize, Serializer};
use strum::EnumString;
use thiserror::Error;
use tokio::{fs, sync::Mutex};
use walkdir::WalkDir;

use crate::{
    models::{Layer, Person, Photo, Place, Tag},
    people::{PEOPLE, PEOPLE_COUNTS},
    photos::{PHOTOS, RAW, VALIDATION_CACHE, VIDEO},
    places::{LAYERS, LAYER_COUNTS, PLACES, PLACE_COUNTS},
    schema::{layers, people, photos, places, tags},
    tags::{validate_tags, TAGS, TAG_COUNTS},
    MIGRATIONS,
};

pub mod api;

pub const DATE_FORMAT: &str = "%F";

lazy_static! {
    pub static ref DB: Mutex<Option<SyncConnectionWrapper<SqliteConnection>>> = Mutex::new(None);
    pub static ref OPEN_FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);
    pub static ref THUMBNAIL_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

pub fn row_to_vec(row_text: &Option<String>) -> Vec<String> {
    if row_text.is_none() {
        return Vec::new();
    }
    let row_text = row_text.as_ref().unwrap();
    let mut re = vec![];
    if row_text.len() > 0 {
        re = row_text.split(",").map(str::to_string).collect();
    }
    re
}

pub fn vec_to_row(row_vec: &Vec<String>) -> Option<String> {
    if row_vec.is_empty() {
        return None;
    }
    return Some(row_vec.join(","));
}

#[derive(Debug, Error)]
pub enum ApiError {
    TauriError(#[from] tauri::Error),
    Error(#[from] anyhow::Error),
    KeyError(#[from] strum::ParseError),
    NotFound(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self {
            ApiError::TauriError(e) => format!("Application error: {}", e),
            ApiError::Error(e) => format!("{}", e),
            ApiError::KeyError(e) => format!("Key error: {}", e),
            ApiError::NotFound(msg) => format!("Item not found: {}", msg),
        };
        error!("{message}");
        write!(f, "{message}")
    }
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize)]
pub struct LoadedPhotos {
    removed: Vec<String>,
    new_photos: Vec<String>,
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

fn try_get_term(term: &String, start: usize) -> Result<String> {
    let val = term.get(start..);
    if val.is_none() {
        return Err(anyhow!("Invalid search term: {term}"));
    }
    Ok(val.unwrap().to_string())
}

#[derive(EnumString, PartialEq)]
pub enum Sort {
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

pub async fn ensure_db() -> Result<()> {
    let db_lock = DB.lock().await;
    if db_lock.is_none() {
        return Err(anyhow!("No database connection established!"));
    }
    Ok(())
}

pub async fn get_photo_targets(id: &String) -> Result<Vec<Photo>> {
    ensure_db().await?;
    let mut targets = Vec::<Photo>::new();
    targets.push(
        photos::table
            .filter(photos::name.eq(id))
            .first::<Photo>(DB.lock().await.as_mut().unwrap())
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
            .load::<Photo>(DB.lock().await.as_mut().unwrap())
            .await?
        {
            targets.push(row);
        }
    }
    Ok(targets)
}

fn degrees_to_dec(input: &String) -> Result<f32> {
    let mut extracted = vec![];
    for digit in Regex::new(r"([0-9\.]+)").unwrap().find_iter(input) {
        extracted.push(digit.as_str());
    }
    let digits = extracted
        .into_iter()
        .map(|c| c.parse::<f32>().unwrap())
        .collect::<Vec<f32>>();
    if digits.len() != 3 {
        return Err(anyhow!("Invalid format for GPS degrees"));
    }

    Ok(digits[0] + (digits[1] / 60.0) + (digits[2] / 3600.0))
}

async fn create_photo(
    _photo: &Photo,
    conn: &mut SyncConnectionWrapper<SqliteConnection>,
    photo_state: &mut HashMap<String, Photo>,
) -> Result<String> {
    let mut photo = _photo.clone();
    let filename = &photo.name;
    let mut file_open = File::open(filename)?;
    let mut file_reader = BufReader::new(&mut file_open);
    let exif = exif::Reader::new().read_from_container(&mut file_reader);
    let mut file_date: Option<NaiveDateTime> = None;
    let mut file_location: Option<(f32, f32)> = None;
    if exif.is_err() {
        warn!(
            "Failed to read exif for {0}: {1}",
            filename,
            exif.as_ref().err().unwrap()
        );
        // If exif read fails, fall back to file metadata
        // Take the min between file created and file modified, often in my project the modified is more accurate than created
        let mut min_meta_time: Option<DateTime<Utc>> = None;
        let metadata = file_open.metadata()?;
        let created = metadata.created();
        if created.is_ok() {
            min_meta_time = Some(DateTime::<Utc>::from(created.as_ref().unwrap().clone()));
        }
        let modified = metadata.modified();
        if modified.is_ok() {
            let modified = DateTime::<Utc>::from(created.as_ref().unwrap().clone());
            if min_meta_time.is_none() {
                min_meta_time = Some(modified);
            } else if &modified < min_meta_time.as_ref().unwrap() {
                min_meta_time = Some(modified);
            }
        }
        if min_meta_time.is_some() {
            file_date = Some(min_meta_time.unwrap().naive_utc());
        } else {
            warn!(
                "Failed to read file metadata dates for {0}: {1}",
                filename,
                created.as_ref().err().unwrap()
            );
        }
    } else {
        let exif = exif.unwrap();
        let time_taken = exif.get_field(exif::Tag::DateTime, In::PRIMARY);
        if time_taken.is_some() {
            let value = &time_taken.unwrap().value;
            let parsed = NaiveDateTime::parse_from_str(
                &value.display_as(exif::Tag::DateTime).to_string(),
                "%F %T",
            );
            if parsed.is_ok() {
                file_date = Some(parsed.unwrap());
            } else {
                warn!(
                    "Exif date exists but failed to parse for {filename}: {}",
                    parsed.err().unwrap()
                );
            }
        }

        let lat = exif.get_field(exif::Tag::GPSLatitude, In::PRIMARY);
        let lng = exif.get_field(exif::Tag::GPSLongitude, In::PRIMARY);
        if lat.is_some() && lng.is_some() {
            // degress, min, sec format
            let lat_val = degrees_to_dec(
                &lat.unwrap()
                    .value
                    .display_as(exif::Tag::GPSLatitude)
                    .to_string(),
            );
            let lng_val = degrees_to_dec(
                &lng.unwrap()
                    .value
                    .display_as(exif::Tag::GPSLongitude)
                    .to_string(),
            );
            if lat_val.is_err() {
                error!(
                    "Could not parse exif lat for {filename}: {}",
                    lat_val.err().unwrap()
                );
            } else if lng_val.is_err() {
                error!(
                    "Could not parse exif lng for {filename}: {}",
                    lng_val.err().unwrap()
                );
            } else {
                let mut lat_val = lat_val.unwrap();
                let mut lng_val = lng_val.unwrap();
                if exif
                    .get_field(exif::Tag::GPSLatitudeRef, In::PRIMARY)
                    .unwrap()
                    .value
                    .display_as(exif::Tag::GPSLatitudeRef)
                    .to_string()
                    == "S"
                {
                    lat_val = -lat_val;
                }
                if exif
                    .get_field(exif::Tag::GPSLongitudeRef, In::PRIMARY)
                    .unwrap()
                    .value
                    .display_as(exif::Tag::GPSLongitudeRef)
                    .to_string()
                    == "W"
                {
                    lng_val = -lng_val;
                }
                file_location = Some((lat_val, lng_val));
            }
        } else {
            warn!("No GPS data in exif for {filename}");
        }
    }

    if file_date.is_some() {
        let date_str = file_date.as_ref().unwrap().format(DATE_FORMAT).to_string();
        info!("Resolved photo date from file data: {date_str}",);
        photo.metadata_date = Some(date_str.clone());
        photo.date = Some(date_str);
    }

    if file_location.is_some() {
        let file_location = file_location.unwrap();
        photo.metadata_location = Some(format!("{0},{1}", file_location.0, file_location.1));
    }

    if insert_into(photos::table)
        .values(photo.clone())
        .execute(conn)
        .await
        .is_err()
    {
        error!(
            "ERROR: Could not insert photo {} into database!",
            &photo.name,
        );
    }
    photo_state.insert(photo.name.clone(), photo.clone());
    Ok(filename.to_string())
}

async fn load_photos() -> Result<LoadedPhotos> {
    let path = OPEN_FOLDER.lock().await;
    if path.is_none() {
        return Err(anyhow!("No open folder found"));
    }
    let path = path.as_ref().unwrap();
    let thumbnail_dir = THUMBNAIL_DIR.lock().await;
    if thumbnail_dir.is_none() {
        return Err(anyhow!("No thumbnail dir found"));
    }
    let thumbnail_dir = thumbnail_dir.as_ref().unwrap();
    info!("Loading photos from {}", path.display());
    ensure_db().await?;
    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();
    // Photos stored in the database, which does not necessarily reflect photos actually present in the folder
    let mut existing = HashMap::new();
    let mut new_photos = vec![];

    let photo_load = photos::table.load::<Photo>(conn).await;
    if photo_load.is_err() {
        return Err(anyhow!(
            "Failed to load photos from database: {}",
            photo_load.err().unwrap()
        ));
    }
    for photo in photo_load.unwrap() {
        existing.insert(photo.name.clone(), photo);
    }
    debug!("Found {} existing photos", existing.len());

    // The processed list of extant photos in the folder, a combination of existing database entries and new empty objects for new files
    let mut photos = HashMap::new();

    // Read the files in the selected folder
    let mut file_queue = VecDeque::<PathBuf>::new();
    let dir = fs::read_dir(&path).await;
    if dir.is_err() {
        return Err(anyhow!(
            "Failed to read the selected folder: {}",
            dir.err().unwrap()
        ));
    }
    let mut dir = dir.unwrap();
    while let Some(entry) = dir.next_entry().await? {
        file_queue.push_back(entry.path());
    }
    let pool = ThreadPool::new(4, 4, Duration::from_millis(50));
    let mut threads = vec![];
    for file in WalkDir::new(path) {
        let file = file?;
        if file.metadata().unwrap().is_file() {
            let filename = file.path().display().to_string();
            let extension = file.path().extension();
            if extension.is_some() && extension.unwrap().to_str().unwrap() == "zip" {
                warn!("Skipping zip file: {filename}");
                continue;
            }
            if existing.contains_key(&filename) {
                let existing_photo = existing.get(&filename.to_string()).unwrap();
                photos.insert(existing_photo.name.clone(), existing_photo.clone());
                existing.remove(&filename.to_string());
            } else {
                let thumbnail_path = thumbnail_dir.join(clean_thumbnail_path(&filename));
                if RAW.is_match(&filename.to_uppercase()) {
                    if !thumbnail_path.exists() {
                        threads.push(pool.complete(async move {
                            debug!("Generating thumbnail for raw {filename}");
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
                            photo
                        }));
                    }
                } else if VIDEO.is_match(&filename.to_uppercase()) {
                    if !thumbnail_path.exists() {
                        threads.push(pool.complete(async move {
                            debug!("Generating thumbnail for video {filename}");
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
                            photo
                        }));
                    }
                } else {
                    new_photos.push(
                        create_photo(&Photo::new(filename.clone()), conn, &mut photos).await?,
                    );
                }
            }
        }
    }

    for thread in threads {
        new_photos.push(create_photo(&thread.await_complete(), conn, &mut photos).await?);
    }

    debug!("Validating photos");
    let mut validations = Vec::new();
    for photo in photos.values() {
        let validation = validate_tags(&photo.tags()).await?;
        validations.push((photo.name.clone(), validation));
    }

    {
        let mut validation_cache = VALIDATION_CACHE.lock().unwrap();
        for (photo_name, validation) in validations {
            validation_cache.insert(photo_name, validation);
        }
    }

    info!("Loaded {} photos", photos.len());
    *PHOTOS.lock().await = photos;

    Ok(LoadedPhotos {
        removed: existing.keys().cloned().map(String::from).collect(),
        new_photos,
    })
}

/// Sets the working folder path & initializes the SQLite database connection.
/// Returns initial information from the database.
pub async fn initialize(path: &String, app_dir: &PathBuf) -> Result<LoadedPhotos> {
    // Establish a sync connection just to apply migrations
    let path = Path::new(path);
    let db_path = path.join("photos.db");
    let db_path = db_path.to_str().unwrap();

    let conn = SqliteConnection::establish(db_path);
    if conn.is_err() {
        return Err(anyhow!(
            "Failed to establish database connection: {}",
            conn.err().unwrap()
        ));
    }
    let mut conn = conn.unwrap();
    let migrations = conn.run_pending_migrations(MIGRATIONS);
    if migrations.is_err() {
        return Err(anyhow!(
            "Failed to run database migrations: {}",
            migrations.err().unwrap()
        ));
    }
    debug!("Applied migrations to {db_path}");

    let conn = SyncConnectionWrapper::<SqliteConnection>::establish(db_path).await;
    if conn.is_err() {
        return Err(anyhow!(
            "Failed to establish async database connection: {}",
            conn.err().unwrap()
        ));
    }
    let mut conn = conn.unwrap();
    debug!("Loaded async database connection from {db_path}");

    let thumbnail_dir = app_dir.join("thumbnails");
    if !thumbnail_dir.exists() {
        if fs::create_dir_all(&thumbnail_dir).await.is_err() {
            return Err(anyhow!(
                "Failed to create thumbnail directory: {}",
                thumbnail_dir.to_str().unwrap()
            ));
        }
    }

    let layers_data = layers::table.load::<Layer>(&mut conn).await?;
    let places_data = places::table.load::<Place>(&mut conn).await?;
    let tags_data = tags::table.load::<Tag>(&mut conn).await?;
    let people_data = people::table.load::<Person>(&mut conn).await?;

    *DB.lock().await = Some(conn);
    *OPEN_FOLDER.lock().await = Some(path.to_path_buf());
    *THUMBNAIL_DIR.lock().await = Some(thumbnail_dir);

    let photo_load = load_photos().await?;

    let mut layers = LAYERS.lock().await;
    let mut places = PLACES.lock().await;
    let mut tags = TAGS.lock().await;
    let mut people = PEOPLE.lock().await;
    let loaded_photos = PHOTOS.lock().await;
    let mut layer_counts = LAYER_COUNTS.lock().unwrap();
    let mut place_counts = PLACE_COUNTS.lock().unwrap();
    let mut tag_counts = TAG_COUNTS.lock().unwrap();
    let mut people_counts = PEOPLE_COUNTS.lock().unwrap();

    for layer in layers_data {
        layer_counts.insert(layer.id.clone(), 0);
        layers.insert(layer.id.clone(), layer);
    }

    for place in places_data {
        place_counts.insert(place.id.clone(), 0);
        places.insert(place.id.clone(), place);
    }

    for tag in tags_data {
        tag_counts.insert(tag.name.clone(), 0);
        tags.insert(tag.name.clone(), tag);
    }

    for person in people_data {
        people_counts.insert(person.id.clone(), 0);
        people.insert(person.id.clone(), person);
    }

    for photo in loaded_photos.values() {
        for tag in photo.tags() {
            if !tags.contains_key(&tag) {
                tags.insert(tag.clone(), Tag::new(&tag));
            }
            if tag_counts.contains_key(&tag) {
                *tag_counts.get_mut(&tag).unwrap() += 1;
            } else {
                tag_counts.insert(tag.clone(), 1);
            }
        }
        for person in photo.people() {
            if people_counts.contains_key(&person) {
                *people_counts.get_mut(&person).unwrap() += 1;
            } else {
                people_counts.insert(person.clone(), 1);
            }
        }
        if photo.location.is_some() {
            let location = photo.location.as_ref().unwrap();
            if places.contains_key(location) {
                let place = places.get(location).unwrap();
                if place_counts.contains_key(location) {
                    *place_counts.get_mut(location).unwrap() += 1;
                } else {
                    place_counts.insert(location.clone(), 1);
                }
                if layer_counts.contains_key(&place.layer) {
                    *layer_counts.get_mut(&place.layer).unwrap() += 1;
                } else {
                    layer_counts.insert(place.layer.clone(), 1);
                }
            } else {
                warn!(
                    "Place referenced by photo {0} not found: {1}",
                    photo.name, location
                );
            }
        }
    }

    Ok(photo_load)
}

/// Performs a search of the photos using the given query.
pub async fn search_photos(query: &Vec<String>, sort: Sort) -> Result<Vec<Photo>> {
    debug!(
        "Searching photos with query \"{0}\", sorted by {1}",
        query.join(","),
        sort
    );
    let mut unmet_terms = vec![];

    // Construct a SQL statement using terms that require no additional processing (is:..., at:..., only:..., by:..., has:...)
    let mut statement = photos::table
        .filter(
            photos::is_duplicate
                .eq(0)
                .or(photos::is_duplicate.is_null()),
        )
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
            unmet_terms.push(term.to_string());
        }
    }

    if has_date {
        if has_date_negated {
            statement = statement.filter(photos::date.is_null());
        } else {
            statement = statement.filter(photos::date.is_not_null());
        }
    }
    debug!(
        "Constructed SQL query for search: {}",
        debug_query(&statement)
    );

    let mut results = Vec::<Photo>::new();
    ensure_db().await?;
    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();
    let photo_records = statement.load::<Photo>(conn).await?;
    debug!("Query returned {} photos", photo_records.len());

    // I *want* to use SQL ORDER BY to sort the results, but it seems the results lose their order somewhere in the above statement
    // TODO test again with sql ordering

    // Terms that require additional processing and iterating over the photos (date:..., of:..., any tags)
    let people = people::table
        .load::<Person>(conn)
        .await?
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect::<HashMap<String, Person>>();
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
                    let mut in_photo = photo.people().contains(q);
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
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, DATE_FORMAT)
                                .ok();
                        meets_terms = meets_terms
                            && ((negated && photo.date() != comp_date)
                                || (!negated && photo.date() == comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE>=") {
                    let comp_date =
                        NaiveDate::parse_from_str(&try_get_term(&tmp_term, 6)?, DATE_FORMAT).ok();
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        meets_terms = meets_terms
                            && ((negated && photo.date() < comp_date)
                                || (!negated && photo.date() >= comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE<=") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 6)?, DATE_FORMAT)
                                .ok();
                        meets_terms = meets_terms
                            && ((negated && photo.date() > comp_date)
                                || (!negated && photo.date() <= comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE>") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, DATE_FORMAT)
                                .ok();
                        meets_terms = meets_terms
                            && ((negated && photo.date() < comp_date)
                                || (!negated && photo.date() > comp_date));
                    }
                } else if term_matches(&tmp_term, "DATE<") {
                    if photo.date.is_none() {
                        meets_terms = false;
                    } else {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 5)?, DATE_FORMAT)
                                .ok();
                        meets_terms = meets_terms
                            && ((negated && photo.date() > comp_date)
                                || (!negated && photo.date() < comp_date));
                    }
                } else if term_matches(&tmp_term, "IS:") {
                    if try_get_term(&tmp_term, 4)?.to_uppercase() == "VIDEO" {
                        meets_terms = meets_terms
                            && ((negated && !photo.is_video()) || (!negated && photo.is_video()));
                    } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "RAW" {
                        meets_terms = meets_terms
                            && ((negated && !photo.is_raw()) || (!negated && photo.is_raw()));
                    }
                } else {
                    // Treat all remaining terms as tags
                    let in_tags = photo.tags().contains(&tmp_term);
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
                    match results.binary_search_by_key(&photo.date(), |p| p.date()) {
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
            results.sort_by_key(|p| p.date());
        }
    }

    if sort == Sort::NameDesc || sort == Sort::RatingDesc || sort == Sort::DateDesc {
        results.reverse();
    }

    debug!("Search returned {} photos", results.len());
    Ok(results)
}

pub async fn remove_deleted(deleted: &Vec<String>) -> Result<()> {
    ensure_db().await?;

    for name in deleted {
        delete(photos::table.filter(photos::name.eq(name)))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    }
    Ok(())
}

pub async fn refresh() -> Result<Vec<String>> {
    let photo_load = load_photos().await?;
    Ok(photo_load.removed)
}
