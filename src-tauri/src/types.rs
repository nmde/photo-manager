#[derive(serde::Serialize)]
pub struct Activity {
    pub id: String,
    pub name: String,
    pub icon: String,
}

#[derive(serde::Serialize)]
pub struct Camera {
    pub id: String,
    pub name: String,
    pub count: i64,
}

#[derive(serde::Serialize)]
pub struct Group {
    pub id: String,
    pub name: String,
}

#[derive(serde::Serialize)]
pub struct Journal {
    pub id: String,
    pub date: String,
    pub mood: i64,
    pub text: String,
    pub activities: String,
    pub steps: i64,
    pub iv: String,
}

#[derive(serde::Serialize)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(serde::Serialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub photo: String,
    pub notes: String,
    pub category: String,
    pub photographer_count: i64,
    pub photo_count: i64
}

#[derive(serde::Serialize)]
pub struct PersonCategory {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(serde::Serialize, Clone)]
pub struct Photo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub is_duplicate: i64,
    pub rating: i64,
    pub location: String,
    pub thumbnail: String,
    pub video: i64,
    pub photo_group: String,
    pub date: String,
    pub raw: i64,
    pub people: Vec<String>,
    pub hide_thumbnail: i64,
    pub photographer: String,
    pub camera: String,
    pub valid_tags: bool,
    pub validation_msg: String
}

#[derive(serde::Serialize)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub layer: String,
    pub category: String,
    pub shape: String,
    pub tags: String,
    pub notes: String,
    pub count: i64
}

#[derive(serde::Serialize)]
pub struct Setting {
    pub id: String,
    pub setting: String,
    pub value: i64,
}

#[derive(serde::Serialize)]
pub struct Shape {
    pub id: String,
    pub shape_type: String,
    pub points: String,
    pub layer: String,
    pub name: String,
}

#[derive(serde::Serialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub prereqs: Vec::<String>,
    pub coreqs: Vec::<String>,
    pub incompatible: Vec::<String>,
    pub count: i64
}

#[derive(serde::Serialize)]
pub struct WikiPage {
    pub id: String,
    pub name: String,
    pub content: String,
    pub iv: String,
}
