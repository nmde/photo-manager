// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::stdout, path::Path, time::SystemTime};

use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use fern::{log_file, Dispatch};
use log::LevelFilter;
use tokio::fs;

use crate::{
    app::api::{initialize, photo_grid, refresh, remove_deleted},
    people::api::{
        create_person, create_person_category, get_people, get_people_categories,
        set_person_category, set_person_name, set_person_photo,
    },
    photos::api::{
        set_photo_date, set_photo_desc, set_photo_group, set_photo_hide_thumbnail,
        set_photo_is_duplicate, set_photo_location, set_photo_people, set_photo_rating,
        set_photo_tags, set_photo_title, set_photographer,
    },
    places::api::{
        create_layer, create_place, create_shape, delete_layer, delete_place, delete_shape,
        get_layers, get_places, get_shapes, set_layer_color, set_layer_name, set_place_category,
        set_place_layer, set_place_name, set_place_position, set_place_shape, set_shape_layer,
        set_shape_name, set_shape_points,
    },
    settings::api::{add_color, get_colors, get_theme, promote_color, set_theme},
    tags::api::{
        get_tags, set_tag_color, set_tag_coreqs, set_tag_incompatible, set_tag_prereqs,
        validate_photo,
    },
};

mod app;
mod models;
mod people;
mod photos;
mod places;
mod schema;
mod settings;
mod tags;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

async fn setup_logger() -> Result<()> {
    // Clear previous log file
    let file = Path::new("photo_manager.log");
    if file.exists() {
        fs::remove_file(file).await?;
    }
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
        .chain(log_file(file)?)
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    setup_logger().await.expect("Failed to initialize logger");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            set_layer_name,
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
            get_theme,
            set_theme,
            get_colors,
            promote_color,
            add_color,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
