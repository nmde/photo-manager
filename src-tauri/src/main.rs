// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::stdout, time::SystemTime};

use chrono::{DateTime, Utc};
use diesel::{Connection, SqliteConnection};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use fern::{log_file, Dispatch};
use log::LevelFilter;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::{
    people::api::{
        create_person, create_person_category, get_people, get_people_categories,
        set_person_category, set_person_name, set_person_photo,
    },
    photos::api::{
        initialize, photo_grid, refresh, remove_deleted, set_photo_date, set_photo_desc,
        set_photo_group, set_photo_hide_thumbnail, set_photo_is_duplicate, set_photo_location,
        set_photo_people, set_photo_rating, set_photo_tags, set_photo_title, set_photographer,
    },
    places::api::{
        create_layer, create_place, create_shape, delete_layer, delete_place, delete_shape,
        get_layers, get_places, get_shapes, set_layer_color, set_place_category, set_place_layer,
        set_place_name, set_place_position, set_place_shape, set_shape_layer, set_shape_name,
        set_shape_points,
    },
    settings::api::{get_setting, set_setting},
    tags::api::{
        get_tags, set_tag_color, set_tag_coreqs, set_tag_incompatible, set_tag_prereqs,
        validate_photo,
    },
};

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
    #[error("Application error: {0}")]
    TauriError(#[from] tauri::Error),
    #[error("{0}")]
    Error(#[from] anyhow::Error),
    #[error("Key error: {0}")]
    KeyError(#[from] strum::ParseError),
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub struct PhotoManager {
    db: SyncConnectionWrapper<SqliteConnection>,
}

impl Default for PhotoManager {
    fn default() -> Self {
        Self {
            db: SyncConnectionWrapper::new(SqliteConnection::establish(":memory:").unwrap()),
        }
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
            create_person,
            create_person_category,
            set_person_name,
            set_person_category,
            set_person_photo,
            get_people,
            get_people_categories,
            initialize,
            photo_grid,
            remove_deleted,
            set_photo_title,
            set_photo_desc,
            set_photographer,
            set_photo_people,
            set_photo_location,
            set_photo_tags,
            set_photo_date,
            set_photo_group,
            set_photo_rating,
            set_photo_is_duplicate,
            set_photo_hide_thumbnail,
            refresh,
            get_layers,
            get_shapes,
            get_places,
            create_layer,
            set_layer_color,
            delete_layer,
            create_place,
            set_place_name,
            set_place_category,
            set_place_shape,
            set_place_layer,
            set_place_position,
            delete_place,
            create_shape,
            set_shape_layer,
            set_shape_points,
            set_shape_name,
            delete_shape,
            set_tag_color,
            set_tag_prereqs,
            set_tag_coreqs,
            set_tag_incompatible,
            get_tags,
            validate_photo,
            set_setting,
            get_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
