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

#[derive(serde::Serialize, Clone)]
pub struct Setting {
    pub id: String,
    pub setting: String,
    pub value: i64,
}

#[derive(serde::Serialize)]
pub struct WikiPage {
    pub id: String,
    pub name: String,
    pub content: String,
    pub iv: String,
}
