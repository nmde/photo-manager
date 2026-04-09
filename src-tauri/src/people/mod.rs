use std::{collections::HashMap, result, sync};

use anyhow::Result;
use diesel::{
    dsl::{insert_into, update},
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use tokio::sync::Mutex;

use crate::{
    app::{ensure_db, DB},
    models::{Person, PersonCategory},
    schema::{people, people_categories},
};

pub mod api;

lazy_static! {
    pub static ref PEOPLE: Mutex<HashMap<String, Person>> = Mutex::new(HashMap::new());
    pub static ref PEOPLE_COUNTS: sync::Mutex<HashMap<String, usize>> =
        sync::Mutex::new(HashMap::new());
}

pub async fn create_person(id: &String, name: &String, category: &String) -> Result<()> {
    ensure_db().await?;

    let new_person = Person {
        id: id.clone(),
        name: name.clone(),
        photo: None,
        category: category.clone(),
    };
    insert_into(people::table)
        .values(new_person.clone())
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;
    PEOPLE.lock().await.insert(id.clone(), new_person);

    Ok(())
}

pub async fn create_person_category(id: &String, name: &String, color: &String) -> Result<()> {
    ensure_db().await?;
    insert_into(people_categories::table)
        .values(PersonCategory {
            id: id.clone(),
            name: name.clone(),
            color: color.clone(),
        })
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

    Ok(())
}

pub async fn get_people() -> Result<HashMap<String, Person>> {
    Ok(PEOPLE.lock().await.to_owned())
}

pub async fn get_people_categories() -> Result<Vec<PersonCategory>> {
    ensure_db().await?;
    Ok(people_categories::table
        .load::<PersonCategory>(DB.lock().await.as_mut().unwrap())
        .await?)
}

impl Person {
    pub async fn set_person_name(&mut self, person: &String, value: &String) -> Result<()> {
        ensure_db().await?;
        update(people::table.filter(people::id.eq(person)))
            .set(people::name.eq(value))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.name = value.clone();

        Ok(())
    }

    pub async fn set_person_category(&mut self, person: &String, value: &String) -> Result<()> {
        ensure_db().await?;
        update(people::table.filter(people::id.eq(person)))
            .set(people::category.eq(value))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.category = value.clone();

        Ok(())
    }

    pub async fn set_person_photo(&mut self, person: &String, value: &Option<String>) -> Result<()> {
        ensure_db().await?;
        update(people::table.filter(people::id.eq(person)))
            .set(people::photo.eq(value))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.photo = value.clone();

        Ok(())
    }
}

impl Serialize for Person {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let counts_cache = PEOPLE_COUNTS.lock().unwrap();
        let mut state = serializer.serialize_struct("PersonDto", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("photo", &self.photo)?;
        state.serialize_field("category", &self.category)?;
        state.serialize_field("count", &counts_cache.get(&self.id).unwrap_or(&0))?;
        state.end()
    }
}
