use std::{collections::HashMap, result, sync};

use anyhow::Result;
use diesel::{dsl::update, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use tokio::sync::Mutex;

use crate::{
    app::{row_to_vec, DB},
    models::{Photo, Tag},
    schema::{photos, tags},
};

pub mod api;

lazy_static! {
    pub static ref TAGS: Mutex<HashMap<String, Tag>> = Mutex::new(HashMap::new());
    pub static ref TAG_COUNTS: sync::Mutex<HashMap<String, usize>> =
        sync::Mutex::new(HashMap::new());
}

#[derive(Clone, Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub message: Option<String>,
}

pub enum TagRelationship {
    Prereqs,
    Coreqs,
    Incompatible,
}

impl Tag {
    pub fn prereqs(&self) -> Vec<String> {
        row_to_vec(&self.prereqs)
    }

    pub fn coreqs(&self) -> Vec<String> {
        row_to_vec(&self.coreqs)
    }

    pub fn incompatible(&self) -> Vec<String> {
        row_to_vec(&self.incompatible)
    }

    pub async fn set_tag_color(&self, tag: &String, value: &String) -> Result<()> {
        update(tags::table.filter(tags::name.eq(tag)))
            .set(tags::color.eq(value))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn modify_tag_relationships(
        &self,
        category: TagRelationship,
        tag: &String,
        value: &Vec<String>,
    ) -> Result<()> {
        let joined = value.join(",");
        match category {
            TagRelationship::Prereqs => {
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::prereqs.eq(joined))
                    .execute(DB.lock().await.as_mut().unwrap())
                    .await?
            }
            TagRelationship::Coreqs => {
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::coreqs.eq(joined))
                    .execute(DB.lock().await.as_mut().unwrap())
                    .await?
            }
            TagRelationship::Incompatible => {
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::incompatible.eq(joined))
                    .execute(DB.lock().await.as_mut().unwrap())
                    .await?
            }
        };
        Ok(())
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let counts_cache = TAG_COUNTS.lock().unwrap();
        let mut state = serializer.serialize_struct("TagDto", 6)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("color", &self.color)?;
        state.serialize_field("prereqs", &self.prereqs())?;
        state.serialize_field("coreqs", &self.coreqs())?;
        state.serialize_field("incompatible", &self.incompatible())?;
        state.serialize_field("count", &counts_cache.get(&self.name).unwrap_or(&0))?;
        state.end()
    }
}

pub async fn validate_tags(tags: &Vec<String>) -> Result<ValidationResult> {
    if tags.len() == 0 {
        Ok(ValidationResult {
            is_valid: true,
            message: None,
        })
    } else {
        let mut valid = true;
        let mut message = String::new();
        let mut missing_prereqs = String::new();
        let mut missing_coreqs = String::new();
        let mut incompatibles = String::new();
        for tag in tags {
            let tag_data = get_tag(tag).await;
            if tag_data.is_ok() {
                let tag_data = tag_data.unwrap();
                for prereq in &tag_data.prereqs() {
                    if !tags.contains(prereq) {
                        valid = false;
                        missing_prereqs.push_str(&prereq);
                        missing_prereqs.push_str(", ");
                    }
                }
                for coreq in &tag_data.coreqs() {
                    if !tags.contains(coreq) {
                        valid = false;
                        missing_coreqs.push_str(&coreq);
                        missing_coreqs.push_str(", ");
                    }
                }
                for incompatible in &tag_data.incompatible() {
                    if tags.contains(incompatible) {
                        valid = false;
                        incompatibles.push_str(&incompatible);
                        incompatibles.push_str(", ");
                    }
                }
            }
        }

        if missing_prereqs.len() > 0 {
            message.push_str("Missing prerequisite tag(s): ");
            message.push_str(&missing_prereqs);
        }
        if missing_coreqs.len() > 0 {
            message.push_str("Missing corequisite tag(s): ");
            message.push_str(&missing_coreqs);
        }
        if incompatibles.len() > 0 {
            message.push_str("Incompatible tag(s) present: ");
            message.push_str(&incompatibles);
        }

        Ok(ValidationResult {
            is_valid: valid,
            message: if message.len() > 0 {
                Some(message)
            } else {
                None
            },
        })
    }
}

async fn get_tag(tag: &String) -> Result<Tag> {
    Ok(tags::table
        .filter(tags::name.eq(tag))
        .first(DB.lock().await.as_mut().unwrap())
        .await?)
}

pub async fn get_tags() -> Result<Vec<Tag>> {
    Ok(tags::table.load(DB.lock().await.as_mut().unwrap()).await?)
}

pub async fn validate_photo(photo: &String) -> Result<ValidationResult> {
    let target = photos::table
        .filter(photos::name.eq(photo))
        .first::<Photo>(DB.lock().await.as_mut().unwrap())
        .await?;
    Ok(validate_tags(&target.tags()).await?)
}
