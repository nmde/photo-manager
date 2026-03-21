use serde::Serialize;


#[derive(Serialize)]
pub struct Group {
    pub id: String,
    pub name: String,
}
