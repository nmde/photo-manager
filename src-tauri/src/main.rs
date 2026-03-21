// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io;

use serde::{Serialize, Serializer};
use sqlite::Row;
use thiserror::Error;

use crate::photos::LoadPhotoError;

mod group;
mod people;
mod photos;
mod places;
mod settings;
mod tags;

pub fn esc(value: &String) -> String {
    String::from(value).replace("\'", "\'\'").to_string()
}

pub fn row_to_vec(row: &Row, key: &str) -> Vec<String> {
    let row_text = row.read::<&str, _>(key).to_string();
    let mut re = Vec::<String>::new();
    if row_text.len() > 0 {
        re = row_text.split(",").map(str::to_string).collect();
    }
    re
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("Failed to parse date: {0}")]
    DateParseError(#[from] chrono::ParseError),
    #[error("Database error: {0}")]
    SqliteError(#[from] sqlite::Error),
    #[error("Item not found: {0}")]
    NotFoundError(String),
    #[error("Application error: {0}")]
    TauriError(#[from] tauri::Error),
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("Failed to load photos: {0}")]
    LoadPhotoError(#[from] LoadPhotoError),
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(photos::PhotoState::default())
        .invoke_handler(tauri::generate_handler![
            people::create_person,
            people::create_person_category,
            people::set_person_name,
            people::set_person_category,
            people::set_person_notes,
            people::set_person_photo,
            people::get_people,
            people::get_people_categories,
            photos::initialize,
            photos::photo_grid,
            photos::remove_deleted,
            photos::set_photo_str,
            photos::set_photographer,
            photos::set_photo_people,
            photos::set_photo_location,
            photos::set_photo_tags,
            photos::set_photo_date,
            photos::set_photo_group,
            photos::set_photo_rating,
            photos::set_photo_bool,
            photos::refresh,
            places::get_layers,
            places::get_shapes,
            places::get_places,
            places::create_layer,
            places::set_layer_str,
            places::delete_layer,
            places::create_place,
            places::set_place_str,
            places::set_place_layer,
            places::set_place_position,
            places::delete_place,
            places::create_shape,
            places::set_shape_str,
            places::delete_shape,
            tags::set_tag_color,
            tags::set_tag_prereqs,
            tags::set_tag_coreqs,
            tags::set_tag_incompatible,
            tags::get_tags,
            tags::validate_photo,
            settings::set_setting,
            settings::get_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
