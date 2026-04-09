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
    app::{get_photo_targets, row_to_vec, DATE_FORMAT, DB},
    models::Photo,
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

    pub async fn set_photo_title(&self, photo: &String, value: &String) -> Result<()> {
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::title.eq(value))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn set_photo_desc(&self, photo: &String, value: &String) -> Result<()> {
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::description.eq(value))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn set_photographer(&self, photo: &String, value: &String) -> Result<()> {
        let targets = get_photo_targets(&photo).await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in targets {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::photographer.eq(value))
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_people(&self, photo: &String, value: &Vec<String>) -> Result<()> {
        let targets = get_photo_targets(&photo).await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in targets {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::people.eq(value.join(",")))
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_location(&self, photo: &String, value: &String) -> Result<()> {
        let targets = get_photo_targets(&photo).await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in targets {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::location.eq(value))
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_date(&self, photo: &String, value: &String) -> Result<()> {
        let targets = get_photo_targets(&photo).await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in targets {
            update(photos::table.filter(photos::name.eq(target.name)))
                .set(photos::date.eq(value))
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    pub async fn set_photo_group(&self, photo: &String, value: &String) -> Result<()> {
        let targets = get_photo_targets(&photo).await?;
        if value.len() == 0 {
            let mut conn = DB.lock().await;
            let conn = conn.as_mut().unwrap();
            for target in &targets {
                update(photos::table.filter(photos::name.eq(target.name.clone())))
                    .set(photos::photo_group.eq::<Option<String>>(None))
                    .execute(conn)
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

            let mut conn = DB.lock().await;
            let conn = conn.as_mut().unwrap();
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
                    .execute(conn)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn set_photo_rating(&self, photo: &String, rating: i32) -> Result<()> {
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::rating.eq(rating))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn set_photo_is_duplicate(&self, photo: &String, value: bool) -> Result<()> {
        let int_val = if value { 1 } else { 0 };
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::is_duplicate.eq(int_val))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn set_photo_hide_thumbnail(&self, photo: &String, value: bool) -> Result<()> {
        let int_val = if value { 1 } else { 0 };
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        update(photos::table.filter(photos::name.eq(photo)))
            .set(photos::hide_thumbnail.eq(int_val))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn set_photo_tags(
        &self,
        photo: &String,
        value: &Vec<String>,
    ) -> Result<ValidationResult> {
        let validation = validate_tags(&value).await?;

        let targets = get_photo_targets(&photo).await?;
        let existing_tags = targets[0].tags();
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in &targets {
            update(photos::table.filter(photos::name.eq(target.name.clone())))
                .set(photos::tags.eq(value.join(",")))
                .execute(conn)
                .await?;
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
        for tag in existing_tags {
            if let Some(c) = tag_counts.get_mut(&tag) {
                *c -= count;
            }
        }
        for tag in value {
            if let Some(c) = tag_counts.get_mut(tag) {
                *c += count;
            } else {
                tag_counts.insert(tag.clone(), count);
            }
        }

        Ok(validation)
    }
}
