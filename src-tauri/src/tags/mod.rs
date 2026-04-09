use std::{collections::HashMap, result, sync};

use anyhow::{anyhow, Result};
use diesel::{
    dsl::{insert_into, update},
    ExpressionMethods, QueryDsl, SqliteConnection,
};
use diesel_async::{sync_connection_wrapper::SyncConnectionWrapper, RunQueryDsl};
use lazy_static::lazy_static;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use tokio::sync::Mutex;

use crate::{
    app::{ensure_db, row_to_vec, DB},
    models::Tag,
    photos::PHOTOS,
    schema::tags,
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

async fn ensure_tag(
    tag: &String,
    conn: &mut SyncConnectionWrapper<SqliteConnection>,
) -> Result<()> {
    if tags::table
        .filter(tags::name.eq(tag))
        .load::<Tag>(conn)
        .await?
        .len()
        == 0
    {
        insert_into(tags::table)
            .values(Tag::new(tag))
            .execute(conn)
            .await?;
    }
    Ok(())
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
        let tags_repo = TAGS.lock().await;
        for tag in tags {
            if tags_repo.contains_key(tag) {
                let tag_data = tags_repo.get(tag).unwrap();
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

pub async fn get_tags() -> Result<HashMap<String, Tag>> {
    Ok(TAGS.lock().await.clone())
}

pub async fn validate_photo(photo: &String) -> Result<ValidationResult> {
    let photos = PHOTOS.lock().await;
    if !photos.contains_key(photo) {
        return Err(anyhow!("Photo {photo} not found!"));
    }
    let target = photos.get(photo).unwrap();

    Ok(validate_tags(&target.tags()).await?)
}

impl Tag {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            color: None,
            prereqs: None,
            coreqs: None,
            incompatible: None,
        }
    }

    pub fn prereqs(&self) -> Vec<String> {
        row_to_vec(&self.prereqs)
    }

    pub fn coreqs(&self) -> Vec<String> {
        row_to_vec(&self.coreqs)
    }

    pub fn incompatible(&self) -> Vec<String> {
        row_to_vec(&self.incompatible)
    }

    pub async fn set_tag_color(&mut self, tag: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        ensure_tag(tag, conn).await?;
        update(tags::table.filter(tags::name.eq(tag)))
            .set(tags::color.eq(value))
            .execute(conn)
            .await?;
        self.color = value.clone();

        Ok(())
    }

    pub async fn modify_tag_relationships(
        &mut self,
        category: TagRelationship,
        tag: &String,
        value: &Vec<String>,
    ) -> Result<()> {
        ensure_db().await?;
        let joined = value.join(",");
        let mut conn = DB.lock().await;
        let conn = conn.as_mut().unwrap();
        ensure_tag(tag, conn).await?;
        match category {
            TagRelationship::Prereqs => {
                self.prereqs = Some(joined.clone());
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::prereqs.eq(joined))
                    .execute(conn)
                    .await?
            }
            TagRelationship::Coreqs => {
                self.coreqs = Some(joined.clone());
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::coreqs.eq(joined))
                    .execute(conn)
                    .await?
            }
            TagRelationship::Incompatible => {
                self.incompatible = Some(joined.clone());
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::incompatible.eq(joined))
                    .execute(conn)
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
