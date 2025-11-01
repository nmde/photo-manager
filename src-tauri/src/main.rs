// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod photos;
mod types;
mod tags;

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
            database::create_wiki_page,
            database::set_wiki_str,
            photos::open_folder,
            photos::search_photos,
            photos::photo_grid,
            photos::remove_deleted,
            photos::set_photo_str,
            photos::set_photographer,
            photos::set_photo_people,
            photos::set_photo_camera,
            photos::set_photo_location,
            photos::set_photo_tags,
            photos::set_photo_date,
            photos::set_photo_group,
            photos::set_photo_rating,
            photos::set_photo_bool,
            tags::set_tag_color,
            tags::set_tag_prereqs,
            tags::get_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
