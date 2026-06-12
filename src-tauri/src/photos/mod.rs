use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync,
};

use anyhow::Result;
use chrono::NaiveDate;
use diesel::{dsl::update, query_builder::AsChangeset, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    app::{ensure_db, get_photo_targets, row_to_vec, vec_to_row, DATE_FORMAT, DB},
    models::{Photo, Tag},
    people::{PEOPLE_COUNTS, PHOTOGRAPHER_COUNTS},
    places::PLACE_COUNTS,
    schema::photos,
    tags::{validate_tags, ValidationResult, TAGS, TAG_COUNTS},
};

pub mod api;

lazy_static! {
    pub static ref RAW: Regex = Regex::new(r"^.*\.(ORF|NRW|HEIC|TIFF|TIF)$").unwrap();
    pub static ref VIDEO: Regex =
        Regex::new(r"^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM|FLV)$").unwrap();
    pub static ref PHOTOS: Mutex<HashMap<String, Photo>> =
        Mutex::new(HashMap::new());
    // Needs to be a sync mutex to be used in serialization
    pub static ref VALIDATION_CACHE: sync::Mutex<HashMap<String, ValidationResult>> =
        sync::Mutex::new(HashMap::new());
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

pub fn get_asset_path(filename: &String) -> String {
    format!(
        "https://asset.localhost/{0}",
        url_escape::encode_component(&filename)
    )
}

pub async fn get_group(group: &String) -> Result<Vec<Photo>> {
    ensure_db().await?;
    Ok(photos::table
        .filter(photos::photo_group.eq(group))
        .load(DB.lock().await.as_mut().unwrap())
        .await?)
}

impl Photo {
    pub fn new(filename: String) -> Self {
        Self {
            name: filename.clone(),
            asset_path: get_asset_path(&filename),
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
            metadata_date: None,
            metadata_location: None,
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

    /// Returns the filename of a paired RAW file if one exists on disk alongside this photo.
    pub fn grouped_raw(&self) -> Option<String> {
        if !self.is_raw() && !self.is_video() {
            let as_path = Path::new(&self.name);
            for ext in ["ORF", "NRW"] {
                let raw_path = as_path.with_extension(ext);
                if raw_path.exists() {
                    return Some(raw_path.to_str().unwrap().to_string());
                }
            }
        }
        None
    }

    pub fn is_video(&self) -> bool {
        VIDEO.is_match(&self.name.to_uppercase())
    }

    pub fn metadata_date(&self) -> Option<NaiveDate> {
        if self.metadata_date.is_none() {
            return None;
        }
        NaiveDate::parse_from_str(self.metadata_date.as_ref().unwrap(), DATE_FORMAT).ok()
    }

    pub fn metadata_location(&self) -> Option<(f32, f32)> {
        if self.metadata_location.is_none() {
            return None;
        }
        let split = self
            .metadata_location
            .as_ref()
            .unwrap()
            .split(",")
            .filter_map(|x| {
                let parsed = x.parse::<f32>();
                if parsed.is_ok() {
                    Some(parsed.unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<f32>>();
        if split.len() != 2 {
            warn!("Malformed photo metadata location for {}", self.name);
            return None;
        }
        Some((split[0], split[1]))
    }

    pub async fn set_photo_title(&mut self, photo: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::title.eq(value))
            .execute(conn)
            .await?;
        self.title = value.clone();

        Ok(())
    }

    pub async fn set_photo_desc(&mut self, photo: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::description.eq(value))
            .execute(conn)
            .await?;
        self.description = value.clone();

        Ok(())
    }

    pub async fn set_photographer(&mut self, photo: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        let targets = get_photo_targets(&photo).await?;
        let count = targets.len();
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in &targets {
            update(photos::table.filter(photos::name.eq(target.name.clone())))
                .set(photos::photographer.eq(value))
                .execute(conn)
                .await?;
        }

        let mut photographer_counts = PHOTOGRAPHER_COUNTS.lock().unwrap();
        if let Some(old) = &self.photographer {
            if photographer_counts.contains_key(old) {
                *photographer_counts.get_mut(old).unwrap() =
                    photographer_counts[old].saturating_sub(count);
            }
        }
        if let Some(new) = value {
            if photographer_counts.contains_key(new) {
                *photographer_counts.get_mut(new).unwrap() += count;
            } else {
                photographer_counts.insert(new.clone(), count);
            }
        }

        self.photographer = value.clone();
        Ok(())
    }

    pub async fn set_photo_people(&self, photo: &String, value: &Vec<String>) -> Result<()> {
        ensure_db().await?;
        let mut targets = get_photo_targets(&photo).await?;
        let existing_people = targets[0].people();
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        let joined = vec_to_row(&value);
        for target in &mut targets {
            update(photos::table.filter(photos::name.eq(target.name.clone())))
                .set(photos::people.eq(joined.clone()))
                .execute(conn)
                .await?;
            target.people = joined.clone();
        }

        // Acquire sync lock after all awaits
        let mut people_counts = PEOPLE_COUNTS.lock().unwrap();

        let count = targets.len();
        for person in &existing_people {
            if !value.contains(person) {
                if people_counts.contains_key(person) {
                    *people_counts.get_mut(person).unwrap() -= count;
                }
            }
        }
        for person in value {
            if !existing_people.contains(person) {
                if people_counts.contains_key(person) {
                    *people_counts.get_mut(person).unwrap() += count;
                } else {
                    people_counts.insert(person.clone(), count);
                }
            }
        }

        Ok(())
    }

    pub async fn set_photo_location(&self, photo: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        let mut targets = get_photo_targets(&photo).await?;
        let existing_place = targets[0].location.clone();
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in &mut targets {
            update(photos::table.filter(photos::name.eq(target.name.clone())))
                .set(photos::location.eq(value.clone()))
                .execute(conn)
                .await?;
            target.location = value.clone();
        }

        // Acquire sync lock after all awaits
        let mut place_counts = PLACE_COUNTS.lock().unwrap();

        let count = targets.len();
        if existing_place.is_some() {
            let existing_place = existing_place.unwrap();
            if place_counts.contains_key(&existing_place) {
                *place_counts.get_mut(&existing_place).unwrap() -= count;
            }
        }
        if value.is_some() {
            let value = value.as_ref().unwrap();
            if place_counts.contains_key(value) {
                *place_counts.get_mut(value).unwrap() += count;
            } else {
                place_counts.insert(value.clone(), count);
            }
        }

        Ok(())
    }

    pub async fn set_photo_date(&mut self, photo: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        let targets = get_photo_targets(&photo).await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in targets {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(
                    photos::date.eq(if value.is_some() && value.as_ref().unwrap().len() > 0 {
                        Some(value.clone().unwrap())
                    } else {
                        None
                    }),
                )
                .execute(conn)
                .await?;
        }
        self.date = value.clone();

        Ok(())
    }

    pub async fn set_photo_group(&self, photo: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        if value.is_none() {
            let mut conn = DB.lock().await;
            let conn = conn.as_mut().unwrap();
            let mut targets = get_photo_targets(&photo).await?;
            for target in &mut targets {
                update(photos::table.filter(photos::name.eq(target.name.clone())))
                    .set(photos::photo_group.eq::<Option<String>>(None))
                    .execute(conn)
                    .await?;
                target.photo_group = None;
            }
        } else {
            let value = value.as_ref().unwrap();
            let mut targets = get_photo_targets(&photo).await?;
            let existing_people = targets[0].people();
            let existing_tags = targets[0].tags();
            let existing_place = targets[0].location.clone();
            let mut collected_tags = HashSet::<String>::new();
            let mut collected_location: Option<String> = None;
            let mut collected_people = HashSet::<String>::new();
            let mut collected_photographer: Option<String> = None;
            let mut collected_date: Option<String> = None;
            for row in targets.as_mut_slice() {
                for tag in row.tags() {
                    collected_tags.insert(tag.clone());
                }
                if collected_location.clone().is_none()
                    || (row.name == *photo && row.location.is_some())
                {
                    collected_location = row.location.clone();
                }
                for person in row.people() {
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
            let tags_str = vec_to_row(&tags_vec);
            let people_str = vec_to_row(&people_vec);

            let mut conn = DB.lock().await;
            let conn = conn.as_mut().unwrap();
            for row in targets.as_mut_slice() {
                update(photos::table.filter(photos::name.eq(row.name.clone())))
                    .into_boxed()
                    .set(GroupFields {
                        photo_group: value.clone(),
                        tags: tags_str.clone(),
                        location: collected_location.clone(),
                        people: people_str.clone(),
                        photographer: collected_photographer.clone(),
                        date: collected_date.clone(),
                    })
                    .execute(conn)
                    .await?;
                row.photo_group = Some(value.clone());
                row.tags = tags_str.clone();
                row.location = collected_location.clone();
                row.people = people_str.clone();
                row.photographer = collected_photographer.clone();
                row.date = collected_date.clone();
            }

            let mut people_counts = PEOPLE_COUNTS.lock().unwrap();
            let mut place_counts = PLACE_COUNTS.lock().unwrap();
            let mut tag_counts = TAG_COUNTS.lock().unwrap();

            let count = targets.len();
            for person in &existing_people {
                if !people_vec.contains(person) {
                    if people_counts.contains_key(person) {
                        *people_counts.get_mut(person).unwrap() -= count;
                    }
                }
            }
            for person in &people_vec {
                if !existing_people.contains(person) {
                    if people_counts.contains_key(person) {
                        *people_counts.get_mut(person).unwrap() += count;
                    } else {
                        people_counts.insert(person.clone(), count);
                    }
                }
            }
            if existing_place.is_some() {
                let existing_place = existing_place.as_ref().unwrap();
                if place_counts.contains_key(existing_place) {
                    *place_counts.get_mut(existing_place).unwrap() -= count;
                }
            }
            if collected_location.is_some() {
                let collected_location = collected_location.unwrap();
                if place_counts.contains_key(&collected_location) {
                    *place_counts.get_mut(&collected_location).unwrap() += count;
                } else {
                    place_counts.insert(collected_location.clone(), count);
                }
            }
            for tag in &existing_tags {
                if !tags_vec.contains(tag) {
                    if tag_counts.contains_key(tag) {
                        *tag_counts.get_mut(tag).unwrap() -= count;
                    }
                }
            }
            for tag in &tags_vec {
                if !existing_tags.contains(tag) {
                    if tag_counts.contains_key(tag) {
                        *tag_counts.get_mut(tag).unwrap() += count;
                    } else {
                        tag_counts.insert(tag.clone(), count);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn set_photo_rating(&mut self, photo: &String, rating: Option<i32>) -> Result<()> {
        ensure_db().await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::rating.eq(rating))
            .execute(conn)
            .await?;
        self.rating = rating;

        Ok(())
    }

    pub async fn set_photo_is_duplicate(&mut self, photo: &String, value: bool) -> Result<()> {
        ensure_db().await?;
        let int_val = if value { 1 } else { 0 };
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::is_duplicate.eq(int_val))
            .execute(conn)
            .await?;
        self.is_duplicate = Some(int_val);

        Ok(())
    }

    pub async fn set_photo_hide_thumbnail(&mut self, photo: &String, value: bool) -> Result<()> {
        ensure_db().await?;
        let int_val = if value { 1 } else { 0 };
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::hide_thumbnail.eq(int_val))
            .execute(conn)
            .await?;
        self.hide_thumbnail = Some(int_val);

        Ok(())
    }

    pub async fn set_photo_tags(
        &self,
        photo: &String,
        value: &Vec<String>,
    ) -> Result<ValidationResult> {
        ensure_db().await?;
        let validation = validate_tags(&value).await?;
        let mut targets = get_photo_targets(&photo).await?;
        let existing_tags = targets[0].tags();
        let mut tags = TAGS.lock().await;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        let joined = vec_to_row(value);
        for target in &mut targets {
            update(photos::table.filter(photos::name.eq(target.name.clone())))
                .set(photos::tags.eq(joined.clone()))
                .execute(conn)
                .await?;
            target.tags = joined.clone();
        }

        // Update validation cache after DB operations
        let mut validation_cache = VALIDATION_CACHE.lock().unwrap();
        for target in &targets {
            if validation_cache.get(&target.name).is_none() {
                validation_cache.insert(target.name.clone(), validation.clone());
            } else {
                let vc = validation_cache.get_mut(&target.name).unwrap();
                vc.is_valid = validation.is_valid;
                vc.message = validation.message.clone();
            }
        }

        for tag in value {
            if !tags.contains_key(tag) {
                tags.insert(tag.clone(), Tag::new(tag));
            }
        }

        let mut tag_counts = TAG_COUNTS.lock().unwrap();
        let count = targets.len();
        for tag in &existing_tags {
            if !value.contains(tag) {
                if tag_counts.contains_key(tag) {
                    *tag_counts.get_mut(tag).unwrap() -= count;
                }
            }
        }
        for tag in value {
            if !existing_tags.contains(tag) {
                if tag_counts.contains_key(tag) {
                    *tag_counts.get_mut(tag).unwrap() += count;
                } else {
                    tag_counts.insert(tag.clone(), count);
                }
            }
        }

        Ok(validation)
    }
}

#[derive(Serialize)]
pub struct PhotoDto {
    pub name: String,
    pub asset_path: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_duplicate: bool,
    pub rating: Option<i32>,
    pub location: Option<String>,
    pub thumbnail: Option<String>,
    pub photo_group: Option<String>,
    pub date: Option<NaiveDate>,
    pub people: Vec<String>,
    pub hide_thumbnail: bool,
    pub photographer: Option<String>,
    pub is_video: bool,
    pub is_raw: bool,
    pub valid_tags: ValidationResult,
    pub metadata_date: Option<NaiveDate>,
    pub metadata_location: Option<(f32, f32)>,
    pub grouped_raw: Option<String>,
}

impl From<&Photo> for PhotoDto {
    fn from(value: &Photo) -> Self {
        let validation_cache = VALIDATION_CACHE.lock().unwrap();
        Self {
            name: value.name.clone(),
            asset_path: value.asset_path.clone(),
            title: value.title.clone(),
            description: value.description.clone(),
            tags: value.tags(),
            is_duplicate: value.is_duplicate.unwrap_or(0) == 1,
            rating: value.rating,
            location: value.location.clone(),
            thumbnail: value.thumbnail.clone(),
            photo_group: value.photo_group.clone(),
            date: value.date(),
            people: value.people(),
            hide_thumbnail: value.hide_thumbnail.unwrap_or(0) == 1,
            photographer: value.photographer.clone(),
            is_video: value.is_video(),
            is_raw: value.is_raw(),
            valid_tags: validation_cache
                .get(&value.name)
                .unwrap_or(&ValidationResult::default())
                .clone(),
            metadata_date: value.metadata_date(),
            metadata_location: value.metadata_location(),
            grouped_raw: value.grouped_raw(),
        }
    }
}
