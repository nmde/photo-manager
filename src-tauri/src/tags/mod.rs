use anyhow::Result;
use diesel::{dsl::update, ExpressionMethods, QueryDsl, TextExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::Serialize;

use crate::{
    models::{Photo, Tag},
    row_to_vec,
    schema::{photos, tags},
    PhotoManager,
};

pub mod api;

impl Tag {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            color: String::new(),
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
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub message: Option<String>,
}

pub enum TagRelationship {
    Prereqs,
    Coreqs,
    Incompatible,
}

impl PhotoManager {
    pub async fn validate_tags(&mut self, tags: &Vec<String>) -> Result<ValidationResult> {
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
                let tag_data = self.get_tag(tag).await;
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

    pub async fn set_tag_color(&mut self, tag: &String, value: &String) -> Result<()> {
        update(tags::table.filter(tags::name.eq(tag)))
            .set(tags::color.eq(value))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn modify_tag_relationships(
        &mut self,
        category: TagRelationship,
        tag: &String,
        value: &Vec<String>,
    ) -> Result<()> {
        let joined = value.join(",");
        match category {
            TagRelationship::Prereqs => {
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::prereqs.eq(joined))
                    .execute(&mut self.db)
                    .await?
            }
            TagRelationship::Coreqs => {
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::coreqs.eq(joined))
                    .execute(&mut self.db)
                    .await?
            }
            TagRelationship::Incompatible => {
                update(tags::table.filter(tags::name.eq(tag)))
                    .set(tags::incompatible.eq(joined))
                    .execute(&mut self.db)
                    .await?
            }
        };
        Ok(())
    }

    async fn get_tag(&mut self, tag: &String) -> Result<Tag> {
        Ok(tags::table
            .filter(tags::name.eq(tag))
            .first(&mut self.db)
            .await?)
    }

    pub async fn get_tags(&mut self) -> Result<Vec<Tag>> {
        Ok(tags::table.load(&mut self.db).await?)
    }

    pub async fn validate_photo(&mut self, photo: &String) -> Result<ValidationResult> {
        let target = photos::table
            .filter(photos::name.eq(photo))
            .first::<Photo>(&mut self.db)
            .await?;
        Ok(self.validate_tags(&target.tags()).await?)
    }

    pub async fn count_photos_by_tag(&mut self, tag: &String) -> Result<usize> {
        Ok(photos::table
            .filter(photos::tags.like(tag))
            .load(&mut self.db)
            .await?
            .into_iter()
            .filter(|photo: &Photo| photo.tags().contains(tag))
            .count())
    }
}
