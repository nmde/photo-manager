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

#[derive(serde::Serialize, Clone)]
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

#[derive(serde::Serialize)]
pub struct WikiPage {
    pub id: String,
    pub name: String,
    pub content: String,
    pub iv: String,
}
