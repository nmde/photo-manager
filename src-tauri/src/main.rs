// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod photos;
mod types;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(photos::PhotoState::default())
        .invoke_handler(tauri::generate_handler![
            database::create_activity,
            database::create_camera,
            database::create_group,
            database::create_journal_entry,
            database::set_journal_str,
            database::set_journal_mood,
            database::create_layer,
            database::set_layer_color,
            database::create_person,
            database::set_person_str,
            database::create_person_category,
            database::create_place,
            database::set_place_str,
            database::set_place_position,
            database::delete_place,
            database::set_setting,
            database::create_shape,
            database::set_shape_str,
            database::delete_shape,
            database::create_tag,
            database::set_tag_str,
            database::create_wiki_page,
            database::set_wiki_str,
            database::set_photo_str,
            database::set_photo_rating,
            database::set_photo_bool,
            photos::open_folder,
            photos::search_photos
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
