#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kinleaf::{
    model::{FamilyCatalog, FamilySummary},
    store::Store,
};
mod commands;

use serde::Serialize;
use tauri::{Manager, State};

#[derive(Serialize)]
pub(crate) struct AppError {
    code: &'static str,
    message: String,
}

#[tauri::command]
fn list_families(store: State<'_, Store>) -> Result<FamilyCatalog, AppError> {
    store.read_catalog().map_err(|error| AppError {
        code: "list_failed",
        message: error.to_string(),
    })
}

#[tauri::command]
fn create_family(
    store: State<'_, Store>,
    name: String,
    founder_name: String,
) -> Result<FamilySummary, AppError> {
    store
        .create_family(name, founder_name)
        .map_err(|error| AppError {
            code: "create_failed",
            message: error.to_string(),
        })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let directory = app.path().app_data_dir()?.join("families");
            app.manage(Store::new(&directory));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_families,
            create_family,
            commands::add_child,
            commands::update_entry,
            commands::delete_entry,
            commands::create_person,
            commands::update_person,
            commands::find_people,
            commands::set_parents,
            commands::add_new_spouse,
            commands::set_spouse,
            commands::list_registrations,
            commands::read_member,
            commands::list_children,
            commands::get_entry,
            commands::search_entries,
            commands::export_backup,
            commands::restore_backup
        ])
        .run(tauri::generate_context!())
        .expect("start Kinleaf");
}
