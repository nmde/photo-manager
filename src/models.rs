use diesel::{Insertable, Queryable, Selectable};

use crate::schema::{
    layers, people, people_categories, photo_groups, photos, places, settings, shapes, tags,
};

#[derive(Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = layers)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = people)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub photo: Option<String>,
    pub category: String,
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = people_categories)]
pub struct PersonCategory {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = photo_groups)]
pub struct PhotoGroup {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = photos)]
pub struct Photo {
    pub name: String,
    pub asset_path: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub is_duplicate: Option<i32>,
    pub rating: Option<i32>,
    pub location: Option<String>,
    pub thumbnail: Option<String>,
    pub photo_group: Option<String>,
    pub date: Option<String>,
    pub people: Option<String>,
    pub hide_thumbnail: Option<i32>,
    pub photographer: Option<String>,
    pub metadata_date: Option<String>,
    pub metadata_location: Option<String>,
}

#[derive(Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = places)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub lat: f32,
    pub lng: f32,
    pub layer: String,
    pub category: String,
    pub shape: Option<String>,
}

#[derive(Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = settings)]
pub struct Setting {
    pub setting: String,
    pub value: String,
}

#[derive(Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = shapes)]
pub struct Shape {
    pub id: String,
    pub shape_type: String,
    pub points: String,
    pub layer: String,
    pub name: String,
}

#[derive(Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub name: String,
    pub color: Option<String>,
    pub prereqs: Option<String>,
    pub coreqs: Option<String>,
    pub incompatible: Option<String>,
}
