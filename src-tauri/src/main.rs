// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod people;
mod photos;
mod places;
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
            database::set_setting,
            database::create_wiki_page,
            database::set_wiki_str,
            people::create_person,
            people::set_person_str,
            people::create_person_category,
            people::get_people,
            people::get_people_categories,
            photos::open_folder,
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
            places::get_layers,
            places::get_shapes,
            places::get_places,
            places::create_layer,
            places::set_layer_color,
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
            tags::get_tag_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
