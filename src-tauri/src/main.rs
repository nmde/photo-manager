// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    io::{self, stdout},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use diesel::{ConnectionError, result};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use fern::{log_file, Dispatch};
use log::LevelFilter;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::photos::LoadPhotoError;

mod models;
mod people;
mod photos;
mod places;
mod schema;
mod settings;
mod tags;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn row_to_vec(row_text: &Option<String>) -> Vec<String> {
    if row_text.is_none() {
        return Vec::new();
    }
    let row_text = row_text.as_ref().unwrap();
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
    SqliteError(#[from] result::Error),
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("Item not found: {0}")]
    NotFoundError(String),
    #[error("Application error: {0}")]
    TauriError(#[from] tauri::Error),
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("Failed to load photos: {0}")]
    LoadPhotoError(#[from] LoadPhotoError),
    #[error("{0}")]
    AnyError(#[from] anyhow::Error),
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn setup_logger() -> anyhow::Result<()> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                DateTime::<Utc>::from(SystemTime::now()).format("%T"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(stdout())
        .chain(log_file("photo_manager.log")?)
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    setup_logger().expect("Failed to initialize logger");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(photos::PhotoState::default())
        .invoke_handler(tauri::generate_handler![
            people::create_person,
            people::create_person_category,
            people::set_person_name,
            people::set_person_category,
            people::set_person_photo,
            people::get_people,
            people::get_people_categories,
            photos::initialize,
            photos::photo_grid,
            photos::remove_deleted,
            photos::set_photo_title,
            photos::set_photo_desc,
            photos::set_photographer,
            photos::set_photo_people,
            photos::set_photo_location,
            photos::set_photo_tags,
            photos::set_photo_date,
            photos::set_photo_group,
            photos::set_photo_rating,
            photos::set_photo_is_duplicate,
            photos::set_photo_hide_thumbnail,
            photos::refresh,
            places::get_layers,
            places::get_shapes,
            places::get_places,
            places::create_layer,
            places::set_layer_color,
            places::delete_layer,
            places::create_place,
            places::set_place_name,
            places::set_place_category,
            places::set_place_shape,
            places::set_place_layer,
            places::set_place_position,
            places::delete_place,
            places::create_shape,
            places::set_shape_layer,
            places::set_shape_points,
            places::set_shape_name,
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
