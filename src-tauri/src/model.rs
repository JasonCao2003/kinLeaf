use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilySummary {
    pub id: String,
    pub name: String,
    pub root_entry_id: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyCatalog {
    pub families: Vec<FamilySummary>,
    pub unreadable_count: usize,
}

#[derive(Debug, Default, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonFields {
    pub name: String,
    pub spectrum_name: Option<String>,
    pub generation_name: Option<String>,
    pub birth_text: Option<String>,
    pub death_text: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Default, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryFields {
    pub rank: Option<i64>,
    pub branch_notes: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PersonChoice {
    New(PersonFields),
    Existing(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonSummary {
    pub id: String,
    pub name: String,
    pub spectrum_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableRegistration {
    pub id: String,
    pub person_id: String,
    pub parent_entry_id: Option<String>,
    pub name: String,
    pub fields: EntryFields,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableMember {
    pub fields: PersonFields,
    pub father_id: Option<String>,
    pub mother_id: Option<String>,
    pub spouse_ids: Vec<String>,
    pub registration_count: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySummary {
    pub entry_id: String,
    pub person_id: String,
    pub name: String,
    pub rank: Option<i64>,
    pub has_children: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub entry: EntrySummary,
    pub generation: usize,
    pub path: Vec<EntrySummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryDetail {
    pub hit: SearchHit,
    pub person: PersonFields,
    pub fields: EntryFields,
    pub father: Option<PersonSummary>,
    pub mother: Option<PersonSummary>,
    pub spouses: Vec<PersonSummary>,
    pub other_registrations: Vec<SearchHit>,
}
