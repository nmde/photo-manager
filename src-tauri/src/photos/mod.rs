use std::{
    collections::{HashMap, HashSet},
    result, sync,
};

use anyhow::Result;
use chrono::NaiveDate;
use diesel::{dsl::update, query_builder::AsChangeset, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use tokio::sync::Mutex;

use crate::{
    app::{ensure_db, get_photo_targets, row_to_vec, vec_to_row, DATE_FORMAT, DB},
    models::Photo,
    people::PEOPLE_COUNTS,
    places::PLACE_COUNTS,
    schema::photos,
    tags::{validate_tags, ValidationResult, TAG_COUNTS},
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

impl Serialize for Photo {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let validation_cache = VALIDATION_CACHE.lock().unwrap();
        let validation = validation_cache
            .get(&self.name)
            .unwrap_or(&ValidationResult {
                is_valid: true,
                message: None,
            });
        let mut state = serializer.serialize_struct("PhotoDto", 18)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("asset_path", &self.asset_path)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("tags", &self.tags())?;
        state.serialize_field("is_duplicate", &(self.is_duplicate.unwrap_or(0) == 1))?;
        state.serialize_field("rating", &self.rating)?;
        state.serialize_field("location", &self.location)?;
        state.serialize_field("thumbnail", &self.thumbnail)?;
        state.serialize_field("photo_group", &self.photo_group)?;
        state.serialize_field("date", &self.date())?;
        state.serialize_field("people", &self.people())?;
        state.serialize_field("hide_thumbnail", &(self.hide_thumbnail.unwrap_or(0) == 1))?;
        state.serialize_field("photographer", &self.photographer)?;
        state.serialize_field("is_video", &self.is_video())?;
        state.serialize_field("is_raw", &self.is_raw())?;
        state.serialize_field("valid_tags", &validation.is_valid)?;
        state.serialize_field("validation_msg", &validation.message)?;
        state.end()
    }
}

impl Photo {
    pub fn new(filename: String) -> Self {
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
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in targets {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::photographer.eq(value))
                .execute(conn)
                .await?;
        }
        self.photographer = value.clone();

        Ok(())
    }

    pub async fn set_photo_people(&self, photo: &String, value: &Vec<String>) -> Result<()> {
        ensure_db().await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();

        let mut targets = get_photo_targets(&photo).await?;
        let existing_people = targets[0].people();
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
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();

        let mut targets = get_photo_targets(&photo).await?;
        let existing_place = targets[0].location.clone();
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
                .set(photos::date.eq(value))
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
                if !value.contains(person) {
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
                    *tag_counts.get_mut(tag).unwrap() -= count;
                }
            }
            for tag in &tags_vec {
                if !existing_tags.contains(tag) {
                    if tag_counts.contains_key(tag) {
                        *tag_counts.get_mut(tag).unwrap() += 1;
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

        let mut tag_counts = TAG_COUNTS.lock().unwrap();
        let count = targets.len();
        for tag in &existing_tags {
            if !value.contains(tag) {
                *tag_counts.get_mut(tag).unwrap() -= count;
            }
        }
        for tag in value {
            if !existing_tags.contains(tag) {
                if tag_counts.contains_key(tag) {
                    *tag_counts.get_mut(tag).unwrap() += 1;
                } else {
                    tag_counts.insert(tag.clone(), count);
                }
            }
        }

        Ok(validation)
    }
}
