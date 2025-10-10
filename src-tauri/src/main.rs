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
            database::get_activities,
            database::create_activity,
            database::get_cameras,
            database::create_camera,
            database::get_groups,
            database::create_group,
            database::get_journals,
            database::create_journal_entry,
            database::set_journal_str,
            database::set_journal_mood,
            database::get_layers,
            database::create_layer,
            database::set_layer_color,
            database::get_people,
            database::create_person,
            database::set_person_str,
            database::get_person_categories,
            database::create_person_category,
            database::get_places,
            database::create_place,
            database::set_place_str,
            database::set_place_position,
            database::delete_place,
            database::get_settings,
            database::set_setting,
            database::get_shapes,
            database::create_shape,
            database::set_shape_str,
            database::delete_shape,
            database::get_tags,
            database::create_tag,
            database::set_tag_str,
            database::get_wiki_pages,
            database::create_wiki_page,
            database::set_wiki_str,
            database::get_photos,
            database::set_photo_str,
            database::set_photo_rating,
            database::set_photo_bool,
            photos::open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
