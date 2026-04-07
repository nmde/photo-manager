use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::Result;
use chrono::NaiveDate;
use diesel::{
    dsl::{delete, insert_into, update},
    query_builder::AsChangeset,
    BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, SqliteConnection,
    TextExpressionMethods,
};
use diesel_async::{sync_connection_wrapper::SyncConnectionWrapper, AsyncConnection, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use lazy_static::lazy_static;
use log::{debug, error, info};
use regex::Regex;
use rusty_pool::{JoinHandle, ThreadPool};
use strum::EnumString;
use tokio::{fs, sync::Mutex};
use walkdir::WalkDir;

use crate::{
    models::{Layer, Person, Photo, PhotoGroup, Place, Setting, Tag},
    row_to_vec,
    schema::{layers, people, photo_groups, photos, places, settings, tags},
    tags::ValidationResult,
    PhotoManager, MIGRATIONS,
};

pub mod api;

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

    pub fn tags(&self) -> Vec<String> {
        row_to_vec(&self.tags)
    }

    pub fn people(&self) -> Vec<String> {
        row_to_vec(&self.people)
    }

    pub fn date(&self) -> Option<NaiveDate> {
        if self.date.is_none() {
            return None;
        }
        NaiveDate::parse_from_str(self.date.as_ref().unwrap(), DATE_FORMAT).ok()
    }

    pub fn is_raw(&self) -> bool {
        RAW.is_match(&self.name.to_uppercase())
    }

    pub fn is_video(&self) -> bool {
        VIDEO.is_match(&self.name.to_uppercase())
    }
}

pub struct PhotoState {
    pub app: Mutex<PhotoManager>,
}

impl Default for PhotoState {
    fn default() -> Self {
        Self {
            app: Mutex::new(PhotoManager::default()),
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

struct LoadedPhotos {
    removed: Vec<String>,
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

impl PhotoManager {
    async fn load_photos(
        &mut self,
        path: &String,
        thumbnail_dir: &PathBuf,
    ) -> Result<LoadedPhotos> {
        info!("Loading photos from {path}");
        // Photos stored in the database, which does not necessarily reflect photos actually present in the folder
        let mut existing = HashMap::<String, Photo>::new();

        for photo in photos::table.load::<Photo>(&mut self.db).await? {
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
                                photo.thumbnail =
                                    Some(thumbnail_path.to_str().unwrap().to_string());
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
                                photo.thumbnail =
                                    Some(thumbnail_path.to_str().unwrap().to_string());
                                photo
                            }));
                        }
                    } else {
                        let photo = Photo::new(filename);
                        insert_into(photos::table)
                            .values(&photo)
                            .execute(&mut self.db)
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
                .execute(&mut self.db)
                .await?;
            photos.insert(result.name.clone(), result);
        }

        Ok(LoadedPhotos {
            removed: existing.keys().cloned().map(String::from).collect(),
        })
    }

    /// Sets the working folder path & initializes the SQLite database connection.
    /// Returns initial information from the database.
    pub async fn initialize(&mut self, path: &String, app_dir: &PathBuf) -> Result<Vec<String>> {
        // Establish a sync connection to apply migrations
        let db_path = Path::new(path).join("photos.db");
        let db_path = db_path.to_str().unwrap();
        let mut conn = SqliteConnection::establish(db_path)?;
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run database migrations");

        let mut conn = SyncConnectionWrapper::<SqliteConnection>::establish(db_path).await?;

        let thumbnail_dir = app_dir.join("thumbnails");
        if !thumbnail_dir.exists() {
            fs::create_dir_all(&thumbnail_dir).await?;
        }

        let mut tags = HashMap::<String, Tag>::new();
        for tag in tags::table.load::<Tag>(&mut conn).await? {
            tags.insert(tag.name.clone(), tag);
        }

        let mut layers = HashMap::<String, Layer>::new();
        for layer in layers::table.load::<Layer>(&mut conn).await? {
            layers.insert(layer.id.clone(), layer);
        }

        let mut places = HashMap::<String, Place>::new();
        for place in places::table.load::<Place>(&mut conn).await? {
            places.insert(place.id.clone(), place);
        }

        let mut people = HashMap::<String, Person>::new();
        for person in people::table.load::<Person>(&mut conn).await? {
            people.insert(person.id.clone(), person);
        }

        let photo_load = self.load_photos(&path, &thumbnail_dir).await?;

        let mut groups = HashMap::<String, PhotoGroup>::new();
        for row in photo_groups::table.load::<PhotoGroup>(&mut conn).await? {
            groups.insert(row.id.clone(), row);
        }

        let mut settings = HashMap::<String, Setting>::new();
        for setting in settings::table.load::<Setting>(&mut conn).await? {
            settings.insert(setting.setting.clone(), setting);
        }

        self.db = conn;

        Ok(photo_load.removed)
    }

    /// Performs a search of the photos using the given query.
    pub async fn search_photos(&mut self, query: &Vec<String>, sort: Sort) -> Result<Vec<Photo>> {
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

        let mut results = Vec::<Photo>::new();
        let mut photo_records = Vec::<Photo>::new();
        for row in statement.load::<Photo>(&mut self.db).await? {
            photo_records.push(row);
        }

        // I *want* to use SQL ORDER BY to sort the results, but it seems the results lose their order somewhere in the above statement
        // TODO test again with sql ordering

        // Terms that require additional processing and iterating over the photos (date:..., of:..., any tags)
        let people = self
            .get_people()
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
                        meets_terms =
                            meets_terms && ((negated && !in_photo) || (!negated && in_photo));
                    }
                    if term_matches(&tmp_term, "DATE:") {
                        if photo.date.is_none() {
                            meets_terms = false;
                        } else {
                            let comp_date = NaiveDate::parse_from_str(
                                &try_get_term(&tmp_term, 5)?,
                                DATE_FORMAT,
                            )
                            .ok();
                            meets_terms = meets_terms
                                && ((negated && photo.date() != comp_date)
                                    || (!negated && photo.date() == comp_date));
                        }
                    } else if term_matches(&tmp_term, "DATE>=") {
                        let comp_date =
                            NaiveDate::parse_from_str(&try_get_term(&tmp_term, 6)?, DATE_FORMAT)
                                .ok();
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
                            let comp_date = NaiveDate::parse_from_str(
                                &try_get_term(&tmp_term, 6)?,
                                DATE_FORMAT,
                            )
                            .ok();
                            meets_terms = meets_terms
                                && ((negated && photo.date() > comp_date)
                                    || (!negated && photo.date() <= comp_date));
                        }
                    } else if term_matches(&tmp_term, "DATE>") {
                        if photo.date.is_none() {
                            meets_terms = false;
                        } else {
                            let comp_date = NaiveDate::parse_from_str(
                                &try_get_term(&tmp_term, 5)?,
                                DATE_FORMAT,
                            )
                            .ok();
                            meets_terms = meets_terms
                                && ((negated && photo.date() < comp_date)
                                    || (!negated && photo.date() > comp_date));
                        }
                    } else if term_matches(&tmp_term, "DATE<") {
                        if photo.date.is_none() {
                            meets_terms = false;
                        } else {
                            let comp_date = NaiveDate::parse_from_str(
                                &try_get_term(&tmp_term, 5)?,
                                DATE_FORMAT,
                            )
                            .ok();
                            meets_terms = meets_terms
                                && ((negated && photo.date() > comp_date)
                                    || (!negated && photo.date() < comp_date));
                        }
                    } else if term_matches(&tmp_term, "IS:") {
                        if try_get_term(&tmp_term, 4)?.to_uppercase() == "VIDEO" {
                            meets_terms = meets_terms
                                && ((negated && !photo.is_video())
                                    || (!negated && photo.is_video()));
                        } else if try_get_term(&tmp_term, 4)?.to_uppercase() == "RAW" {
                            meets_terms = meets_terms
                                && ((negated && !photo.is_raw()) || (!negated && photo.is_raw()));
                        }
                    } else {
                        // Treat all remaining terms as tags
                        let in_tags = photo.tags().contains(&tmp_term);
                        meets_terms =
                            meets_terms && ((negated && !in_tags) || (!negated && in_tags));
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

        Ok(results)
    }

    pub async fn remove_deleted(&mut self, deleted: &Vec<String>) -> Result<()> {
        debug!(
            "Removing {} moved or deleted photos from the database",
            deleted.len()
        );

        for name in deleted {
            delete(photos::table.filter(photos::name.eq(name)))
                .execute(&mut self.db)
                .await?;
        }
        Ok(())
    }

    async fn get_photo_targets(&mut self, id: &String) -> Result<Vec<Photo>> {
        let mut targets = Vec::<Photo>::new();
        targets.push(
            photos::table
                .filter(photos::name.eq(id))
                .first::<Photo>(&mut self.db)
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
                .load::<Photo>(&mut self.db)
                .await?
            {
                targets.push(row);
            }
        }
        Ok(targets)
    }

    pub async fn set_photo_title(&mut self, photo: &String, value: &String) -> Result<()> {
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::title.eq(value))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_photo_desc(&mut self, photo: &String, value: &String) -> Result<()> {
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::description.eq(value))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_photographer(&mut self, photo: &String, value: &String) -> Result<()> {
        for target in self.get_photo_targets(&photo).await? {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::photographer.eq(value))
                .execute(&mut self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_people(&mut self, photo: &String, value: &Vec<String>) -> Result<()> {
        for target in self.get_photo_targets(&photo).await? {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::people.eq(value.join(",")))
                .execute(&mut self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_location(&mut self, photo: &String, value: &String) -> Result<()> {
        for target in self.get_photo_targets(&photo).await? {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::location.eq(value))
                .execute(&mut self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_tags(
        &mut self,
        photo: &String,
        value: &Vec<String>,
    ) -> Result<ValidationResult> {
        let validation = self.validate_tags(&value).await?;
        for target in self.get_photo_targets(&photo).await? {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::tags.eq(value.join(",")))
                .execute(&mut self.db)
                .await?;
        }

        Ok(validation)
    }

    pub async fn set_photo_date(&mut self, photo: &String, value: &String) -> Result<()> {
        for target in self.get_photo_targets(&photo).await? {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::date.eq(value))
                .execute(&mut self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_group(&mut self, photo: &String, value: &String) -> Result<()> {
        let targets = self.get_photo_targets(&photo).await?;
        if value.len() == 0 {
            for target in &targets {
                update(photos::table.filter(photos::name.eq(target.name.clone())))
                    .set(photos::photo_group.eq::<Option<String>>(None))
                    .execute(&mut self.db)
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
                if collected_location.clone().is_none()
                    || (row.name == *photo && row.location.is_some())
                {
                    collected_location = row.location.clone();
                }
                for person in row_to_vec(&row.people) {
                    collected_people.insert(person.clone());
                }
                if collected_photographer.clone().is_none()
                    || (row.name == *photo && row.photographer.is_some())
                {
                    collected_photographer = row.photographer.clone();
                }
                if row.date.is_some()
                    && (collected_date.is_none() || (row.name == *photo && row.date.is_some()))
                {
                    collected_date = row.date.clone();
                }
            }

            let tags_vec = collected_tags.into_iter().collect::<Vec<String>>();
            let people_vec = collected_people.into_iter().collect::<Vec<String>>();

            for row in &targets {
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
                    .execute(&mut self.db)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn set_photo_rating(&mut self, photo: &String, rating: i32) -> Result<()> {
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::rating.eq(rating))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_photo_is_duplicate(&mut self, photo: &String, value: bool) -> Result<()> {
        let int_val = if value { 1 } else { 0 };
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::is_duplicate.eq(int_val))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_photo_hide_thumbnail(&mut self, photo: &String, value: bool) -> Result<()> {
        let int_val = if value { 1 } else { 0 };
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::hide_thumbnail.eq(int_val))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn refresh(&mut self, path: &String, thumbnail_dir: &PathBuf) -> Result<Vec<String>> {
        debug!("Refreshing photos from {path}");

        let photo_load = self.load_photos(&path, thumbnail_dir).await?;

        Ok(photo_load.removed)
    }
}
