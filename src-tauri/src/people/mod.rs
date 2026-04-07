use anyhow::Result;
use diesel::{
    dsl::{insert_into, update},
    ExpressionMethods, QueryDsl, TextExpressionMethods,
};
use diesel_async::RunQueryDsl;

use crate::{
    models::{Person, PersonCategory, Photo},
    schema::{people, people_categories, photos},
    PhotoManager,
};

pub mod api;

impl PhotoManager {
    pub async fn create_person(
        &mut self,
        id: &String,
        name: &String,
        category: &String,
    ) -> Result<()> {
        insert_into(people::table)
            .values(Person {
                id: id.clone(),
                name: name.clone(),
                photo: None,
                category: category.clone(),
            })
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn create_person_category(
        &mut self,
        id: &String,
        name: &String,
        color: &String,
    ) -> Result<()> {
        insert_into(people_categories::table)
            .values(PersonCategory {
                id: id.clone(),
                name: name.clone(),
                color: color.clone(),
            })
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_person_name(&mut self, person: &String, value: &String) -> Result<()> {
        update(people::table.filter(people::id.eq(person)))
            .set(people::name.eq(value))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_person_category(&mut self, person: &String, value: &String) -> Result<()> {
        update(people::table.filter(people::id.eq(person)))
            .set(people::category.eq(value))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_person_photo(&mut self, person: &String, value: &String) -> Result<()> {
        update(people::table.filter(people::id.eq(person)))
            .set(people::photo.eq(value))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn get_people(&mut self) -> Result<Vec<Person>> {
        Ok(people::table.load::<Person>(&mut self.db).await?)
    }

    pub async fn get_people_categories(&mut self) -> Result<Vec<PersonCategory>> {
        Ok(people_categories::table
            .load::<PersonCategory>(&mut self.db)
            .await?)
    }

    pub async fn get_photographer_count(&mut self, person_id: &String) -> Result<usize> {
        Ok(photos::table
            .filter(photos::photographer.eq(person_id))
            .load::<Photo>(&mut self.db)
            .await?
            .len())
    }

    pub async fn get_photo_count(&mut self, person_id: &String) -> Result<usize> {
        Ok(photos::table
            .filter(photos::people.like(person_id))
            .load::<Photo>(&mut self.db)
            .await?
            .iter()
            .filter(|p| p.people().contains(person_id))
            .count())
    }
}
