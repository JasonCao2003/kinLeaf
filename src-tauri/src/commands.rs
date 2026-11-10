use kinleaf::{
    model::{
        EditableMember, EditableRegistration, EntryFields, PersonChoice, PersonFields,
        PersonSummary,
    },
    store::Store,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::AppError;

fn backup_error(error: impl std::fmt::Display) -> AppError {
    AppError {
        code: "backup_operation_failed",
        message: error.to_string(),
    }
}

#[tauri::command]
pub async fn export_backup(app: AppHandle, family_id: String) -> Result<bool, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .dialog()
            .file()
            .set_title("备份族谱")
            .set_file_name("Kinleaf-backup.sqlite3")
            .add_filter("Kinleaf 族谱备份", &["sqlite3"])
            .blocking_save_file();
        let Some(path) = path else { return Ok(false) };
        let path = path.into_path().map_err(backup_error)?;
        kinleaf::backup::export_to(&app.state::<Store>(), &family_id, &path)
            .map_err(backup_error)?;
        Ok(true)
    })
    .await
    .map_err(backup_error)?
}

#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
) -> Result<Option<kinleaf::model::FamilySummary>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .dialog()
            .file()
            .set_title("从备份恢复族谱副本")
            .add_filter("Kinleaf 族谱备份", &["sqlite3"])
            .blocking_pick_file();
        let Some(path) = path else { return Ok(None) };
        let path = path.into_path().map_err(backup_error)?;
        kinleaf::backup::restore_from(&app.state::<Store>(), &path)
            .map(Some)
            .map_err(backup_error)
    })
    .await
    .map_err(backup_error)?
}

fn command_error(error: impl std::fmt::Display) -> AppError {
    AppError {
        code: "member_operation_failed",
        message: error.to_string(),
    }
}

#[tauri::command]
pub fn add_child(
    store: State<'_, Store>,
    family_id: String,
    parent_entry_id: String,
    person: PersonChoice,
    fields: EntryFields,
) -> Result<String, AppError> {
    store
        .add_child(&family_id, &parent_entry_id, person, &fields)
        .map_err(command_error)
}

#[tauri::command]
pub fn update_entry(
    store: State<'_, Store>,
    family_id: String,
    entry_id: String,
    parent_entry_id: Option<String>,
    fields: EntryFields,
) -> Result<(), AppError> {
    store
        .update_entry(&family_id, &entry_id, parent_entry_id.as_deref(), &fields)
        .map_err(command_error)
}

#[tauri::command]
pub fn delete_entry(
    store: State<'_, Store>,
    family_id: String,
    entry_id: String,
) -> Result<(), AppError> {
    store
        .delete_entry(&family_id, &entry_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn create_person(
    store: State<'_, Store>,
    family_id: String,
    fields: PersonFields,
) -> Result<String, AppError> {
    store
        .create_person(&family_id, fields)
        .map_err(command_error)
}

#[tauri::command]
pub fn update_person(
    store: State<'_, Store>,
    family_id: String,
    person_id: String,
    fields: PersonFields,
) -> Result<(), AppError> {
    store
        .update_person(&family_id, &person_id, fields)
        .map_err(command_error)
}

#[tauri::command]
pub fn find_people(
    store: State<'_, Store>,
    family_id: String,
    query: String,
) -> Result<Vec<PersonSummary>, AppError> {
    store.find_people(&family_id, &query).map_err(command_error)
}

#[tauri::command]
pub fn set_parents(
    store: State<'_, Store>,
    family_id: String,
    person_id: String,
    father_id: Option<String>,
    mother_id: Option<String>,
) -> Result<(), AppError> {
    store
        .set_parents(
            &family_id,
            &person_id,
            father_id.as_deref(),
            mother_id.as_deref(),
        )
        .map_err(command_error)
}

#[tauri::command]
pub fn add_new_spouse(
    store: State<'_, Store>,
    family_id: String,
    person_id: String,
    fields: PersonFields,
) -> Result<String, AppError> {
    store
        .add_new_spouse(&family_id, &person_id, fields)
        .map_err(command_error)
}

#[tauri::command]
pub fn set_spouse(
    store: State<'_, Store>,
    family_id: String,
    person_id: String,
    other_id: String,
    linked: bool,
) -> Result<(), AppError> {
    store
        .set_spouse(&family_id, &person_id, &other_id, linked)
        .map_err(command_error)
}

#[tauri::command]
pub fn list_registrations(
    store: State<'_, Store>,
    family_id: String,
) -> Result<Vec<EditableRegistration>, AppError> {
    store.list_registrations(&family_id).map_err(command_error)
}

#[tauri::command]
pub fn read_member(
    store: State<'_, Store>,
    family_id: String,
    person_id: String,
) -> Result<EditableMember, AppError> {
    store
        .read_member(&family_id, &person_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn list_children(
    store: State<'_, Store>,
    family_id: String,
    parent_entry_id: String,
) -> Result<Vec<kinleaf::model::EntrySummary>, AppError> {
    store
        .list_children(&family_id, &parent_entry_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn get_entry(
    store: State<'_, Store>,
    family_id: String,
    entry_id: String,
) -> Result<kinleaf::model::EntryDetail, AppError> {
    store
        .get_entry(&family_id, &entry_id)
        .map_err(command_error)
}

#[tauri::command]
pub fn search_entries(
    store: State<'_, Store>,
    family_id: String,
    query: String,
) -> Result<Vec<kinleaf::model::SearchHit>, AppError> {
    store
        .search_entries(&family_id, &query)
        .map_err(command_error)
}
