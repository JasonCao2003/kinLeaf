use kinleaf::{
    model::{
        EditableMember, EditableRegistration, EntryFields, PersonChoice, PersonFields,
        PersonSummary,
    },
    store::Store,
};
use tauri::State;

use crate::AppError;

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
