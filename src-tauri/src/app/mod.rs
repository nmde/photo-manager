use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use diesel::{
    debug_query, delete, insert_into, BoolExpressionMethods, Connection, ExpressionMethods,
    QueryDsl, SqliteConnection, TextExpressionMethods,
};
use diesel_async::{sync_connection_wrapper::SyncConnectionWrapper, AsyncConnection, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use lazy_static::lazy_static;
use log::{debug, error, info};
use rusty_pool::ThreadPool;
use serde::{Serialize, Serializer};
use strum::EnumString;
use thiserror::Error;
use tokio::{fs, sync::Mutex};
use walkdir::WalkDir;

use crate::{
    models::{Person, Photo},
    photos::{PHOTOS, RAW, VIDEO},
    schema::{people, photos},
    MIGRATIONS,
};

pub mod api;

pub const DATE_FORMAT: &str = "%F";

pub fn row_to_vec(row_text: &Option<String>) -> Vec<String> {
    if row_text.is_none() {
        return Vec::new();
    }
    let row_text = row_text.as_ref().unwrap();
    let mut re = Vec::<String>::new();
    if row_text.len() > 0 {
        re = row_text.split(",").map(str::to_string).collect();
    }
    re
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

lazy_static! {
    pub static ref DB: Mutex<Option<SyncConnectionWrapper<SqliteConnection>>> = Mutex::new(None);
}

struct LoadedPhotos {
    removed: Vec<String>,
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

async fn load_photos(path: &String, thumbnail_dir: &PathBuf) -> Result<LoadedPhotos> {
    info!("Loading photos from {path}");
    ensure_db().await?;
    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();
    // Photos stored in the database, which does not necessarily reflect photos actually present in the folder
    let mut existing = HashMap::new();

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
    debug!("Loading {} files from {path}", file_queue.len());
    let pool = ThreadPool::new(4, 4, Duration::from_millis(50));
    let mut threads = vec![];
    for file in WalkDir::new(path) {
        let file = file?;
        if file.metadata().unwrap().is_file() {
            let filename = file.path().display().to_string();
            if existing.contains_key(&filename) {
                let existing_photo = existing.get(&filename.to_string()).unwrap();
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
                            photo
                        }));
                    }
                } else {
                    let photo = Photo::new(filename);
                    if insert_into(photos::table)
                        .values(&photo)
                        .execute(conn)
                        .await
                        .is_err()
                    {
                        error!(
                            "ERROR: Could not insert photo {} into database!",
                            &photo.name,
                        );
                    }
                    photos.insert(photo.name.clone(), photo);
                }
            }
        }
    }

    for thread in threads {
        let result = thread.await_complete();
        if insert_into(photos::table)
            .values(result.clone())
            .execute(conn)
            .await
            .is_err()
        {
            error!("ERROR: Could not create photo {}!", &result.name,);
        }
        photos.insert(result.name.clone(), result);
    }

    *PHOTOS.lock().await = photos;

    Ok(LoadedPhotos {
        removed: existing.keys().cloned().map(String::from).collect(),
    })
}

/// Sets the working folder path & initializes the SQLite database connection.
/// Returns initial information from the database.
pub async fn initialize(path: &String, app_dir: &PathBuf) -> Result<Vec<String>> {
    // Establish a sync connection to apply migrations
    let db_path = Path::new(path).join("photos.db");
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
    debug!("Loaded sync database connection from {db_path}");

    let conn = SyncConnectionWrapper::<SqliteConnection>::establish(db_path).await;
    if conn.is_err() {
        return Err(anyhow!(
            "Failed to establish async database connection: {}",
            conn.err().unwrap()
        ));
    }
    *DB.lock().await = Some(conn.unwrap());
    ensure_db().await?;
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

    let photo_load = load_photos(&path, &thumbnail_dir).await?;

    Ok(photo_load.removed)
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
    debug!("{} unmet terms need to be evaluted", unmet_terms.len());
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
    debug!(
        "Removing {} moved or deleted photos from the database",
        deleted.len()
    );
    ensure_db().await?;

    for name in deleted {
        delete(photos::table.filter(photos::name.eq(name)))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    }
    Ok(())
}

pub async fn refresh(path: &String, thumbnail_dir: &PathBuf) -> Result<Vec<String>> {
    debug!("Refreshing photos from {path}");
    let photo_load = load_photos(&path, thumbnail_dir).await?;
    Ok(photo_load.removed)
}
