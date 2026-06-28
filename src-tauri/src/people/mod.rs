use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use tokio::sync::Mutex as AsyncMutex;

use anyhow::Result;
use diesel::{
    dsl::{insert_into, update},
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::Serialize;

use crate::{
    app::{ensure_db, DB},
    models::{Person, PersonCategory},
    schema::{people, people_categories},
};

pub mod api;

pub static PEOPLE: LazyLock<AsyncMutex<HashMap<String, Person>>> =
    LazyLock::new(|| AsyncMutex::new(HashMap::new()));
pub static PEOPLE_COUNTS: LazyLock<Mutex<HashMap<String, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub static PHOTOGRAPHER_COUNTS: LazyLock<Mutex<HashMap<String, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn create_person(id: &str, name: &str, category: &str) -> Result<()> {
    ensure_db().await?;

    let new_person = Person {
        id: id.to_owned(),
        name: name.to_owned(),
        photo: None,
        category: category.to_owned(),
    };
    insert_into(people::table)
        .values(new_person.clone())
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;
    PEOPLE.lock().await.insert(id.to_string(), new_person);

    Ok(())
}

pub async fn create_person_category(id: &str, name: &str, color: &str) -> Result<()> {
    ensure_db().await?;
    insert_into(people_categories::table)
        .values(PersonCategory {
            id: id.to_owned(),
            name: name.to_owned(),
            color: color.to_owned(),
        })
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

    Ok(())
}

pub async fn get_people() -> Result<Vec<Person>> {
    Ok(PEOPLE
        .lock()
        .await
        .to_owned()
        .values()
        .cloned()
        .collect::<Vec<Person>>())
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

    pub async fn set_person_photo(
        &mut self,
        person: &String,
        value: &Option<String>,
    ) -> Result<()> {
        ensure_db().await?;
        update(people::table.filter(people::id.eq(person)))
            .set(people::photo.eq(value))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.photo = value.clone();

        Ok(())
    }
}

#[derive(Serialize)]
pub struct PersonDto {
    pub id: String,
    pub name: String,
    pub photo: Option<String>,
    pub category: String,
    pub count: usize,
    pub photographer_count: usize,
}

impl From<&Person> for PersonDto {
    fn from(value: &Person) -> Self {
        let counts_cache = PEOPLE_COUNTS.lock().unwrap();
        let photographer_cache = PHOTOGRAPHER_COUNTS.lock().unwrap();
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            photo: value.photo.clone(),
            category: value.category.clone(),
            count: counts_cache.get(&value.id).copied().unwrap_or(0),
            photographer_count: photographer_cache.get(&value.id).copied().unwrap_or(0),
        }
    }
}
